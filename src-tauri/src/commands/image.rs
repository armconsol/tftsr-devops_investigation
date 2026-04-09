use base64::Engine;
use sha2::Digest;
use std::path::Path;
use tauri::State;

use crate::audit::log::write_audit_event;
use crate::db::models::{AuditEntry, ImageAttachment};
use crate::state::AppState;

const MAX_IMAGE_FILE_BYTES: u64 = 10 * 1024 * 1024;
const SUPPORTED_IMAGE_MIME_TYPES: [&str; 5] = [
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/svg+xml",
];

fn validate_image_file_path(file_path: &str) -> Result<std::path::PathBuf, String> {
    let path = Path::new(file_path);
    let canonical = std::fs::canonicalize(path).map_err(|_| "Unable to access selected file")?;
    let metadata = std::fs::metadata(&canonical).map_err(|_| "Unable to read file metadata")?;

    if !metadata.is_file() {
        return Err("Selected path is not a file".to_string());
    }

    if metadata.len() > MAX_IMAGE_FILE_BYTES {
        return Err(format!(
            "Image file exceeds maximum supported size ({} MB)",
            MAX_IMAGE_FILE_BYTES / 1024 / 1024
        ));
    }

    Ok(canonical)
}

fn is_supported_image_format(mime_type: &str) -> bool {
    SUPPORTED_IMAGE_MIME_TYPES.contains(&mime_type)
}

#[tauri::command]
pub async fn upload_image_attachment(
    issue_id: String,
    file_path: String,
    state: State<'_, AppState>,
) -> Result<ImageAttachment, String> {
    let canonical_path = validate_image_file_path(&file_path)?;
    let content =
        std::fs::read(&canonical_path).map_err(|_| "Failed to read selected image file")?;
    let content_hash = format!("{:x}", sha2::Sha256::digest(&content));
    let file_name = canonical_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let file_size = content.len() as i64;
    let mime_type: String = infer::get(&content)
        .map(|m| m.mime_type().to_string())
        .unwrap_or_else(|| "image/png".to_string());

    if !is_supported_image_format(mime_type.as_str()) {
        return Err(format!(
            "Unsupported image format: {}. Supported formats: {}",
            mime_type,
            SUPPORTED_IMAGE_MIME_TYPES.join(", ")
        ));
    }

    let canonical_file_path = canonical_path.to_string_lossy().to_string();
    let attachment = ImageAttachment::new(
        issue_id.clone(),
        file_name,
        canonical_file_path,
        file_size,
        mime_type,
        content_hash.clone(),
        true,
        false,
    );

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO image_attachments (id, issue_id, file_name, file_path, file_size, mime_type, upload_hash, uploaded_at, pii_warning_acknowledged, is_paste) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            attachment.id,
            attachment.issue_id,
            attachment.file_name,
            attachment.file_path,
            attachment.file_size,
            attachment.mime_type,
            attachment.upload_hash,
            attachment.uploaded_at,
            attachment.pii_warning_acknowledged as i32,
            attachment.is_paste as i32,
        ],
    )
    .map_err(|_| "Failed to store uploaded image metadata".to_string())?;

    let entry = AuditEntry::new(
        "upload_image_attachment".to_string(),
        "image_attachment".to_string(),
        attachment.id.clone(),
        serde_json::json!({
            "issue_id": issue_id,
            "file_name": attachment.file_name,
            "is_paste": false,
        })
        .to_string(),
    );
    if let Err(err) = write_audit_event(
        &db,
        &entry.action,
        &entry.entity_type,
        &entry.entity_id,
        &entry.details,
    ) {
        tracing::warn!(error = %err, "failed to write upload_image_attachment audit entry");
    }

    Ok(attachment)
}

#[tauri::command]
pub async fn upload_paste_image(
    issue_id: String,
    base64_image: String,
    mime_type: String,
    state: State<'_, AppState>,
) -> Result<ImageAttachment, String> {
    if !base64_image.starts_with("data:image/") {
        return Err("Invalid image data - must be a data URL".to_string());
    }

    let data_part = base64_image
        .split(',')
        .nth(1)
        .ok_or("Invalid image data format - missing base64 content")?;

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(data_part)
        .map_err(|_| "Failed to decode base64 image data")?;

    let content_hash = format!("{:x}", sha2::Sha256::digest(&decoded));
    let file_size = decoded.len() as i64;
    let file_name = format!("pasted-image-{}.png", uuid::Uuid::now_v7());

    if !is_supported_image_format(mime_type.as_str()) {
        return Err(format!(
            "Unsupported image format: {}. Supported formats: {}",
            mime_type,
            SUPPORTED_IMAGE_MIME_TYPES.join(", ")
        ));
    }

    let attachment = ImageAttachment::new(
        issue_id.clone(),
        file_name.clone(),
        String::new(),
        file_size,
        mime_type,
        content_hash,
        true,
        true,
    );

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO image_attachments (id, issue_id, file_name, file_path, file_size, mime_type, upload_hash, uploaded_at, pii_warning_acknowledged, is_paste) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            attachment.id,
            attachment.issue_id,
            attachment.file_name,
            attachment.file_path,
            attachment.file_size,
            attachment.mime_type,
            attachment.upload_hash,
            attachment.uploaded_at,
            attachment.pii_warning_acknowledged as i32,
            attachment.is_paste as i32,
        ],
    )
    .map_err(|_| "Failed to store pasted image metadata".to_string())?;

    let entry = AuditEntry::new(
        "upload_paste_image".to_string(),
        "image_attachment".to_string(),
        attachment.id.clone(),
        serde_json::json!({
            "issue_id": issue_id,
            "file_name": attachment.file_name,
            "is_paste": true,
        })
        .to_string(),
    );
    if let Err(err) = write_audit_event(
        &db,
        &entry.action,
        &entry.entity_type,
        &entry.entity_id,
        &entry.details,
    ) {
        tracing::warn!(error = %err, "failed to write upload_paste_image audit entry");
    }

    Ok(attachment)
}

#[tauri::command]
pub async fn list_image_attachments(
    issue_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ImageAttachment>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db
        .prepare(
            "SELECT id, issue_id, file_name, file_path, file_size, mime_type, upload_hash, uploaded_at, pii_warning_acknowledged, is_paste \
             FROM image_attachments WHERE issue_id = ?1 ORDER BY uploaded_at ASC",
        )
        .map_err(|e| e.to_string())?;

    let attachments = stmt
        .query_map([&issue_id], |row| {
            Ok(ImageAttachment {
                id: row.get(0)?,
                issue_id: row.get(1)?,
                file_name: row.get(2)?,
                file_path: row.get(3)?,
                file_size: row.get(4)?,
                mime_type: row.get(5)?,
                upload_hash: row.get(6)?,
                uploaded_at: row.get(7)?,
                pii_warning_acknowledged: row.get::<_, i32>(8)? != 0,
                is_paste: row.get::<_, i32>(9)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(attachments)
}

#[tauri::command]
pub async fn delete_image_attachment(
    attachment_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let affected = db
        .execute(
            "DELETE FROM image_attachments WHERE id = ?1",
            [&attachment_id],
        )
        .map_err(|e| e.to_string())?;

    if affected == 0 {
        return Err("Image attachment not found".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_image_format() {
        assert!(is_supported_image_format("image/png"));
        assert!(is_supported_image_format("image/jpeg"));
        assert!(is_supported_image_format("image/gif"));
        assert!(is_supported_image_format("image/webp"));
        assert!(is_supported_image_format("image/svg+xml"));
        assert!(!is_supported_image_format("image/bmp"));
        assert!(!is_supported_image_format("text/plain"));
    }
}
