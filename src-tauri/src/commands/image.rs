use base64::Engine;
use sha2::Digest;
use std::path::Path;
use tauri::State;

use crate::audit::log::write_audit_event;
use crate::db::models::{AuditEntry, ImageAttachment, ImageAttachmentSummary};
use crate::state::AppState;

const MAX_IMAGE_FILE_BYTES: u64 = 10 * 1024 * 1024;
const SUPPORTED_IMAGE_MIME_TYPES: [&str; 6] = [
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/svg+xml",
    "image/bmp",
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
        "INSERT INTO image_attachments (id, issue_id, file_name, file_path, file_size, mime_type, upload_hash, uploaded_at, pii_warning_acknowledged, is_paste, image_data) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
            content,
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
pub async fn upload_image_attachment_by_content(
    issue_id: String,
    file_name: String,
    base64_content: String,
    state: State<'_, AppState>,
) -> Result<ImageAttachment, String> {
    let data_part = base64_content
        .split(',')
        .nth(1)
        .ok_or("Invalid image data format - missing base64 content")?;

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(data_part)
        .map_err(|_| "Failed to decode base64 image data")?;

    if decoded.len() as u64 > MAX_IMAGE_FILE_BYTES {
        return Err(format!(
            "Image content exceeds maximum supported size ({} MB)",
            MAX_IMAGE_FILE_BYTES / 1024 / 1024
        ));
    }

    let content_hash = format!("{:x}", sha2::Sha256::digest(&decoded));
    let file_size = decoded.len() as i64;

    let mime_type: String = infer::get(&decoded)
        .map(|m| m.mime_type().to_string())
        .unwrap_or_else(|| "image/png".to_string());

    if !is_supported_image_format(mime_type.as_str()) {
        return Err(format!(
            "Unsupported image format: {}. Supported formats: {}",
            mime_type,
            SUPPORTED_IMAGE_MIME_TYPES.join(", ")
        ));
    }

    // Use the file_name as file_path for DB storage
    let attachment = ImageAttachment::new(
        issue_id.clone(),
        file_name.clone(),
        file_name,
        file_size,
        mime_type,
        content_hash.clone(),
        true,
        false,
    );

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO image_attachments (id, issue_id, file_name, file_path, file_size, mime_type, upload_hash, uploaded_at, pii_warning_acknowledged, is_paste, image_data) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
            decoded,
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

    if decoded.len() as u64 > MAX_IMAGE_FILE_BYTES {
        return Err(format!(
            "Pasted image exceeds maximum supported size ({} MB)",
            MAX_IMAGE_FILE_BYTES / 1024 / 1024
        ));
    }

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
        "INSERT INTO image_attachments (id, issue_id, file_name, file_path, file_size, mime_type, upload_hash, uploaded_at, pii_warning_acknowledged, is_paste, image_data) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
            decoded,
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

#[tauri::command]
pub async fn upload_file_to_datastore(
    provider_config: serde_json::Value,
    file_path: String,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    use reqwest::multipart::Form;

    let canonical_path = validate_image_file_path(&file_path)?;
    let content =
        std::fs::read(&canonical_path).map_err(|_| "Failed to read file for datastore upload")?;

    let file_name = canonical_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let _file_size = content.len() as i64;

    // Extract API URL and auth header from provider config
    let api_url = provider_config
        .get("api_url")
        .and_then(|v| v.as_str())
        .ok_or("Provider config missing api_url")?
        .to_string();

    // Extract use_datastore_upload flag
    let use_datastore = provider_config
        .get("use_datastore_upload")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !use_datastore {
        return Err("use_datastore_upload is not enabled for this provider".to_string());
    }

    // Get datastore ID from custom_endpoint_path (stored as datastore ID)
    let datastore_id = provider_config
        .get("custom_endpoint_path")
        .and_then(|v| v.as_str())
        .ok_or("Provider config missing datastore ID in custom_endpoint_path")?
        .to_string();

    // Build upload endpoint: POST /api/v2/upload/<DATASTORE-ID>
    let api_url = api_url.trim_end_matches('/');
    let upload_url = format!("{api_url}/upload/{datastore_id}");

    // Read auth header and value
    let auth_header = provider_config
        .get("custom_auth_header")
        .and_then(|v| v.as_str())
        .unwrap_or("x-generic-api-key");

    let auth_prefix = provider_config
        .get("custom_auth_prefix")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let api_key = provider_config
        .get("api_key")
        .and_then(|v| v.as_str())
        .ok_or("Provider config missing api_key")?;

    let auth_value = format!("{auth_prefix}{api_key}");

    let client = reqwest::Client::new();

    // Create multipart form
    let part = reqwest::multipart::Part::bytes(content)
        .file_name(file_name)
        .mime_str("application/octet-stream")
        .map_err(|e| format!("Failed to create multipart part: {e}"))?;

    let form = Form::new().part("file", part);

    let resp = client
        .post(&upload_url)
        .header(auth_header, auth_value)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Upload request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp
            .text()
            .await
            .unwrap_or_else(|_| "unable to read response".to_string());
        return Err(format!("Datastore upload error {status}: {text}"));
    }

    // Parse response to get file ID
    let json = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse upload response: {e}"))?;

    // Response should have file_id or id field
    let file_id = json
        .get("file_id")
        .or_else(|| json.get("id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            format!(
                "Response missing file_id: {}",
                serde_json::to_string_pretty(&json).unwrap_or_default()
            )
        })?
        .to_string();

    Ok(file_id)
}

/// Upload any file (not just images) to GenAI datastore
#[tauri::command]
pub async fn upload_file_to_datastore_any(
    provider_config: serde_json::Value,
    file_path: String,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    use reqwest::multipart::Form;

    // Validate file exists and is accessible
    let path = Path::new(&file_path);
    let canonical = std::fs::canonicalize(path).map_err(|_| "Unable to access selected file")?;
    let metadata = std::fs::metadata(&canonical).map_err(|_| "Unable to read file metadata")?;

    if !metadata.is_file() {
        return Err("Selected path is not a file".to_string());
    }

    let content =
        std::fs::read(&canonical).map_err(|_| "Failed to read file for datastore upload")?;

    let file_name = canonical
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let _file_size = content.len() as i64;

    // Extract API URL and auth header from provider config
    let api_url = provider_config
        .get("api_url")
        .and_then(|v| v.as_str())
        .ok_or("Provider config missing api_url")?
        .to_string();

    // Extract use_datastore_upload flag
    let use_datastore = provider_config
        .get("use_datastore_upload")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !use_datastore {
        return Err("use_datastore_upload is not enabled for this provider".to_string());
    }

    // Get datastore ID from custom_endpoint_path (stored as datastore ID)
    let datastore_id = provider_config
        .get("custom_endpoint_path")
        .and_then(|v| v.as_str())
        .ok_or("Provider config missing datastore ID in custom_endpoint_path")?
        .to_string();

    // Build upload endpoint: POST /api/v2/upload/<DATASTORE-ID>
    let api_url = api_url.trim_end_matches('/');
    let upload_url = format!("{api_url}/upload/{datastore_id}");

    // Read auth header and value
    let auth_header = provider_config
        .get("custom_auth_header")
        .and_then(|v| v.as_str())
        .unwrap_or("x-generic-api-key");

    let auth_prefix = provider_config
        .get("custom_auth_prefix")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let api_key = provider_config
        .get("api_key")
        .and_then(|v| v.as_str())
        .ok_or("Provider config missing api_key")?;

    let auth_value = format!("{auth_prefix}{api_key}");

    let client = reqwest::Client::new();

    // Create multipart form
    let part = reqwest::multipart::Part::bytes(content)
        .file_name(file_name)
        .mime_str("application/octet-stream")
        .map_err(|e| format!("Failed to create multipart part: {e}"))?;

    let form = Form::new().part("file", part);

    let resp = client
        .post(&upload_url)
        .header(auth_header, auth_value)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Upload request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp
            .text()
            .await
            .unwrap_or_else(|_| "unable to read response".to_string());
        return Err(format!("Datastore upload error {status}: {text}"));
    }

    // Parse response to get file ID
    let json = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse upload response: {e}"))?;

    // Response should have file_id or id field
    let file_id = json
        .get("file_id")
        .or_else(|| json.get("id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            format!(
                "Response missing file_id: {}",
                serde_json::to_string_pretty(&json).unwrap_or_default()
            )
        })?
        .to_string();

    Ok(file_id)
}

#[tauri::command]
pub async fn get_image_attachment_data(
    attachment_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let row: (Option<Vec<u8>>, String, String) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.prepare("SELECT image_data, file_path, mime_type FROM image_attachments WHERE id = ?1")
            .and_then(|mut stmt| {
                stmt.query_row([&attachment_id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })
            })
            .map_err(|_| "Image attachment not found".to_string())?
    };
    let (image_data, file_path, mime_type) = row;
    let bytes = if let Some(data) = image_data {
        data
    } else {
        std::fs::read(&file_path).map_err(|e| format!("Image file not found on disk: {e}"))?
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{mime_type};base64,{b64}"))
}

#[tauri::command]
pub async fn list_all_image_attachments(
    search: Option<String>,
    issue_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<ImageAttachmentSummary>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut query = "SELECT id, issue_id, file_name, file_path, file_size, mime_type, \
                            upload_hash, uploaded_at, pii_warning_acknowledged, is_paste, issue_title \
                     FROM v_image_attachments_with_issue WHERE 1=1"
        .to_string();
    let mut params: Vec<String> = Vec::new();

    if let Some(ref q) = search {
        query.push_str(" AND file_name LIKE ?");
        params.push(format!("%{q}%"));
    }
    if let Some(ref id) = issue_id {
        query.push_str(" AND issue_id = ?");
        params.push(id.clone());
    }
    query.push_str(" ORDER BY uploaded_at DESC");

    let mut stmt = db.prepare(&query).map_err(|e| e.to_string())?;
    let results = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(ImageAttachmentSummary {
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
                issue_title: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(results)
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
        assert!(is_supported_image_format("image/bmp"));
        assert!(!is_supported_image_format("text/plain"));
    }

    #[test]
    fn test_get_image_attachment_data_base64_format() {
        let bytes = b"\x89PNG\r\n\x1a\n";
        let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
        let result = format!("data:{};base64,{}", "image/png", b64);
        assert!(result.starts_with("data:image/png;base64,"));
        assert!(!result.is_empty());
    }
}
