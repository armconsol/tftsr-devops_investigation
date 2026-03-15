use sha2::{Digest, Sha256};
use tauri::State;

use crate::db::models::{AuditEntry, LogFile, PiiSpanRecord};
use crate::pii::{self, PiiDetectionResult, PiiDetector, RedactedLogFile};
use crate::state::AppState;

#[tauri::command]
pub async fn upload_log_file(
    issue_id: String,
    file_path: String,
    state: State<'_, AppState>,
) -> Result<LogFile, String> {
    let path = std::path::Path::new(&file_path);
    let content = std::fs::read(path).map_err(|e| e.to_string())?;
    let content_hash = format!("{:x}", Sha256::digest(&content));
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let file_size = content.len() as i64;
    let mime_type = if file_name.ends_with(".json") {
        "application/json"
    } else if file_name.ends_with(".xml") {
        "application/xml"
    } else {
        "text/plain"
    };

    let log_file = LogFile::new(issue_id.clone(), file_name, file_path.clone(), file_size);
    let log_file = LogFile {
        content_hash: content_hash.clone(),
        mime_type: mime_type.to_string(),
        ..log_file
    };

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO log_files (id, issue_id, file_name, file_path, file_size, mime_type, content_hash, uploaded_at, redacted) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            log_file.id,
            log_file.issue_id,
            log_file.file_name,
            log_file.file_path,
            log_file.file_size,
            log_file.mime_type,
            log_file.content_hash,
            log_file.uploaded_at,
            log_file.redacted as i32,
        ],
    )
    .map_err(|e| e.to_string())?;

    // Audit
    let entry = AuditEntry::new(
        "upload_log_file".to_string(),
        "log_file".to_string(),
        log_file.id.clone(),
        serde_json::json!({ "issue_id": issue_id, "file_name": log_file.file_name }).to_string(),
    );
    let _ = db.execute(
        "INSERT INTO audit_log (id, timestamp, action, entity_type, entity_id, user_id, details) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            entry.id,
            entry.timestamp,
            entry.action,
            entry.entity_type,
            entry.entity_id,
            entry.user_id,
            entry.details
        ],
    );

    Ok(log_file)
}

#[tauri::command]
pub async fn detect_pii(
    log_file_id: String,
    state: State<'_, AppState>,
) -> Result<PiiDetectionResult, String> {
    // Load file path from DB
    let file_path: String = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.prepare("SELECT file_path FROM log_files WHERE id = ?1")
            .and_then(|mut stmt| stmt.query_row([&log_file_id], |row| row.get(0)))
            .map_err(|e| e.to_string())?
    };

    let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;

    let detector = PiiDetector::new();
    let spans = detector.detect(&content);

    // Store PII spans in the database for later use
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for span in &spans {
            let record = PiiSpanRecord {
                id: span.id.clone(),
                log_file_id: log_file_id.clone(),
                pii_type: span.pii_type.clone(),
                start_offset: span.start as i64,
                end_offset: span.end as i64,
                original_value: span.original.clone(),
                replacement: span.replacement.clone(),
            };
            let _ = db.execute(
                "INSERT OR REPLACE INTO pii_spans (id, log_file_id, pii_type, start_offset, end_offset, original_value, replacement) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    record.id, record.log_file_id, record.pii_type,
                    record.start_offset, record.end_offset,
                    record.original_value, record.replacement
                ],
            );
        }
    }

    Ok(PiiDetectionResult {
        log_file_id,
        spans,
        original_text: content,
    })
}

#[tauri::command]
pub async fn apply_redactions(
    log_file_id: String,
    approved_span_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<RedactedLogFile, String> {
    // Load file path
    let file_path: String = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.prepare("SELECT file_path FROM log_files WHERE id = ?1")
            .and_then(|mut stmt| stmt.query_row([&log_file_id], |row| row.get(0)))
            .map_err(|e| e.to_string())?
    };

    let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;

    // Load PII spans from DB, filtering to only approved ones
    let spans: Vec<pii::PiiSpan> = {
        type Row = (String, String, i64, i64, String, String);
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let raw: Vec<Row> = db
            .prepare(
                "SELECT id, pii_type, start_offset, end_offset, original_value, replacement \
                 FROM pii_spans WHERE log_file_id = ?1 ORDER BY start_offset ASC",
            )
            .and_then(|mut stmt| {
                stmt.query_map([&log_file_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                })
                .map(|rows| rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
            })
            .unwrap_or_default();
        drop(db);
        raw.into_iter()
            .map(
                |(id, pii_type, start, end, original, replacement)| pii::PiiSpan {
                    id,
                    pii_type,
                    start: start as usize,
                    end: end as usize,
                    original,
                    replacement,
                },
            )
            .filter(|span| approved_span_ids.contains(&span.id))
            .collect()
    };

    // Apply redactions using the redactor module
    let redacted_text = pii::apply_redactions(&content, &spans);
    let data_hash = pii::hash_content(&redacted_text);

    // Save redacted file alongside original
    let redacted_path = format!("{}.redacted", file_path);
    std::fs::write(&redacted_path, &redacted_text).map_err(|e| e.to_string())?;

    // Mark the log file as redacted in DB
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "UPDATE log_files SET redacted = 1 WHERE id = ?1",
            [&log_file_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(RedactedLogFile {
        log_file_id,
        redacted_text,
        data_hash,
    })
}
