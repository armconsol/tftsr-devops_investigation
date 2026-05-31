use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tauri::State;
use tracing::warn;

use crate::db::models::{AuditEntry, LogFile, PiiSpanRecord};
use crate::pii::{self, PiiDetectionResult, PiiDetector, RedactedLogFile};
use crate::state::AppState;

const MAX_LOG_FILE_BYTES: u64 = 50 * 1024 * 1024;

const SAFE_TEXT_EXTENSIONS: &[&str] = &[
    "log",
    "txt",
    "out",
    "err",
    "syslog",
    "journal",
    "yaml",
    "yml",
    "json",
    "toml",
    "xml",
    "ini",
    "cfg",
    "conf",
    "config",
    "env",
    "properties",
    "md",
    "markdown",
    "rst",
    "csv",
    "tsv",
    "ndjson",
    "jsonl",
    "sql",
    "sh",
    "bash",
    "zsh",
    "py",
    "js",
    "ts",
    "rb",
    "go",
    "rs",
    "java",
    "html",
    "htm",
    "css",
    "diff",
    "patch",
    "rtf",
];

const SAFE_BINARY_EXTENSIONS: &[&str] = &["pdf", "docx", "doc", "xlsx", "xls"];

pub fn is_safe_file(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    match ext.as_deref() {
        Some(e) => SAFE_TEXT_EXTENSIONS.contains(&e) || SAFE_BINARY_EXTENSIONS.contains(&e),
        None => false,
    }
}

pub fn extract_text_content(path: &Path) -> Result<String, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "pdf" => extract_pdf_text(path),
        "docx" | "doc" => extract_docx_text(path),
        "xlsx" | "xls" => Err(format!(
            "Spreadsheet format .{ext} is not yet supported for text extraction. \
             Export the sheet as CSV and upload that instead."
        )),
        _ => std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}")),
    }
}

fn extract_pdf_text(path: &Path) -> Result<String, String> {
    let doc = lopdf::Document::load(path).map_err(|e| format!("Failed to parse PDF: {e}"))?;
    let mut text = String::new();
    let mut pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    pages.sort_unstable();
    for page_num in pages {
        if let Ok(content) = doc.extract_text(&[page_num]) {
            text.push_str(&content);
            text.push('\n');
        }
    }
    if text.trim().is_empty() {
        return Err("PDF contains no extractable text (may be a scanned image)".to_string());
    }
    Ok(text)
}

fn extract_docx_text(path: &Path) -> Result<String, String> {
    use std::io::Read as _;
    let file = std::fs::File::open(path).map_err(|e| format!("Failed to open file: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to open as ZIP/DOCX: {e}"))?;
    let mut xml_content = String::new();
    {
        // Safety: only one hardcoded entry is ever accessed; no arbitrary path extraction is
        // performed, so zip-slip path traversal attacks cannot apply here.
        let mut doc_xml = archive
            .by_name("word/document.xml")
            .map_err(|_| "Not a valid DOCX: missing word/document.xml".to_string())?;
        doc_xml
            .read_to_string(&mut xml_content)
            .map_err(|e| format!("Failed to read document.xml: {e}"))?;
    }
    let mut text = String::new();
    let mut reader = quick_xml::Reader::from_str(&xml_content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Text(e)) => {
                if let Ok(s) = e.unescape() {
                    let trimmed = s.trim().to_string();
                    if !trimmed.is_empty() {
                        text.push_str(&trimmed);
                        text.push(' ');
                    }
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }
    if text.trim().is_empty() {
        return Err("DOCX contains no extractable text".to_string());
    }
    Ok(text)
}

fn validate_log_file_path(file_path: &str) -> Result<PathBuf, String> {
    let path = Path::new(file_path);
    let canonical = std::fs::canonicalize(path).map_err(|_| "Unable to access selected file")?;
    let metadata = std::fs::metadata(&canonical).map_err(|_| "Unable to read file metadata")?;

    if !metadata.is_file() {
        return Err("Selected path is not a file".to_string());
    }

    if metadata.len() > MAX_LOG_FILE_BYTES {
        return Err(format!(
            "File exceeds maximum supported size ({} MB)",
            MAX_LOG_FILE_BYTES / 1024 / 1024
        ));
    }

    Ok(canonical)
}

#[tauri::command]
pub async fn upload_log_file(
    issue_id: String,
    file_path: String,
    state: State<'_, AppState>,
) -> Result<LogFile, String> {
    let canonical_path = validate_log_file_path(&file_path)?;

    if !is_safe_file(&canonical_path) {
        let ext = canonical_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("(none)");
        return Err(format!(
            "File type '.{ext}' is not supported. Supported formats include .log, .txt, .json, .pdf, .docx, .md, and many more."
        ));
    }

    let file_name = canonical_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let file_ext = canonical_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let extracted_text = extract_text_content(&canonical_path)
        .map_err(|e| format!("Failed to read file content: {e}"))?;
    let content_bytes = extracted_text.as_bytes();
    let content_hash = format!("{:x}", Sha256::digest(content_bytes));
    let file_size = content_bytes.len() as i64;

    let mime_type = match file_ext.as_str() {
        "json" => "application/json",
        "xml" => "application/xml",
        "yaml" | "yml" => "application/yaml",
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "doc" => "application/msword",
        "md" | "markdown" => "text/markdown",
        "csv" | "tsv" => "text/csv",
        "html" | "htm" => "text/html",
        _ => "text/plain",
    };

    let is_binary = SAFE_BINARY_EXTENSIONS.contains(&file_ext.as_str());
    let stored_path = if is_binary {
        let extracted_path = canonical_path.with_extension("extracted.txt");
        std::fs::write(&extracted_path, &extracted_text)
            .map_err(|e| format!("Failed to write extracted text: {e}"))?;
        extracted_path.to_string_lossy().to_string()
    } else {
        canonical_path.to_string_lossy().to_string()
    };

    let log_file = LogFile::new(issue_id.clone(), file_name, stored_path, file_size);
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
    .map_err(|_| "Failed to store uploaded log metadata".to_string())?;

    // Audit
    let entry = AuditEntry::new(
        "upload_log_file".to_string(),
        "log_file".to_string(),
        log_file.id.clone(),
        serde_json::json!({ "issue_id": issue_id, "file_name": log_file.file_name }).to_string(),
    );
    if let Err(err) = crate::audit::log::write_audit_event(
        &db,
        &entry.action,
        &entry.entity_type,
        &entry.entity_id,
        &entry.details,
    ) {
        warn!(error = %err, "failed to write upload_log_file audit entry");
    }

    Ok(log_file)
}

#[tauri::command]
pub async fn upload_log_file_by_content(
    issue_id: String,
    file_name: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<LogFile, String> {
    let fake_path = Path::new(&file_name);
    if !is_safe_file(fake_path) {
        let ext = fake_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("(none)");
        return Err(format!("File type '.{ext}' is not supported."));
    }

    let content_bytes = content.as_bytes();
    let content_hash = format!("{:x}", Sha256::digest(content_bytes));
    let file_size = content_bytes.len() as i64;

    let file_ext = fake_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let mime_type = match file_ext.as_str() {
        "json" => "application/json",
        "xml" => "application/xml",
        "yaml" | "yml" => "application/yaml",
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "doc" => "application/msword",
        "md" | "markdown" => "text/markdown",
        "csv" | "tsv" => "text/csv",
        "html" | "htm" => "text/html",
        _ => "text/plain",
    };

    // Use the file_name as the file_path for DB storage
    let log_file = LogFile::new(
        issue_id.clone(),
        file_name.clone(),
        file_name.clone(),
        file_size,
    );
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
    .map_err(|_| "Failed to store uploaded log metadata".to_string())?;

    // Audit
    let entry = AuditEntry::new(
        "upload_log_file".to_string(),
        "log_file".to_string(),
        log_file.id.clone(),
        serde_json::json!({ "issue_id": issue_id, "file_name": log_file.file_name }).to_string(),
    );
    if let Err(err) = crate::audit::log::write_audit_event(
        &db,
        &entry.action,
        &entry.entity_type,
        &entry.entity_id,
        &entry.details,
    ) {
        warn!(error = %err, "failed to write upload_log_file audit entry");
    }

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
            .map_err(|_| "Failed to load log file metadata".to_string())?
    };

    let content =
        std::fs::read_to_string(&file_path).map_err(|_| "Failed to read log file content")?;

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
                original_value: String::new(),
                replacement: span.replacement.clone(),
            };
            if let Err(err) = db.execute(
                "INSERT OR REPLACE INTO pii_spans (id, log_file_id, pii_type, start_offset, end_offset, original_value, replacement) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    record.id, record.log_file_id, record.pii_type,
                    record.start_offset, record.end_offset,
                    record.original_value, record.replacement
                ],
            ) {
                warn!(error = %err, span_id = %span.id, "failed to persist pii span");
            }
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
            .map_err(|_| "Failed to load log file metadata".to_string())?
    };

    let content =
        std::fs::read_to_string(&file_path).map_err(|_| "Failed to read log file content")?;

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
    let redacted_path = format!("{file_path}.redacted");
    std::fs::write(&redacted_path, &redacted_text)
        .map_err(|_| "Failed to write redacted output file".to_string())?;

    // Mark the log file as redacted in DB
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "UPDATE log_files SET redacted = 1 WHERE id = ?1",
            [&log_file_id],
        )
        .map_err(|_| "Failed to mark file as redacted".to_string())?;
        db.execute(
            "UPDATE pii_spans SET original_value = '' WHERE log_file_id = ?1",
            [&log_file_id],
        )
        .map_err(|_| "Failed to finalize redaction metadata".to_string())?;
    }

    Ok(RedactedLogFile {
        log_file_id,
        redacted_text,
        data_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_log_file_path_rejects_non_file() {
        let dir = std::env::temp_dir();
        let result = validate_log_file_path(dir.to_string_lossy().as_ref());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_log_file_path_accepts_small_file() {
        let file_path =
            std::env::temp_dir().join(format!("tftsr-analysis-test-{}.log", uuid::Uuid::now_v7()));
        std::fs::write(&file_path, "hello").unwrap();
        let result = validate_log_file_path(file_path.to_string_lossy().as_ref());
        assert!(result.is_ok());
        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_is_safe_file_allows_txt() {
        assert!(is_safe_file(Path::new("file.txt")));
    }

    #[test]
    fn test_is_safe_file_allows_md() {
        assert!(is_safe_file(Path::new("readme.md")));
    }

    #[test]
    fn test_is_safe_file_allows_pdf() {
        assert!(is_safe_file(Path::new("report.pdf")));
    }

    #[test]
    fn test_is_safe_file_allows_docx() {
        assert!(is_safe_file(Path::new("doc.docx")));
    }

    #[test]
    fn test_is_safe_file_rejects_exe() {
        assert!(!is_safe_file(Path::new("malware.exe")));
    }

    #[test]
    fn test_is_safe_file_rejects_dll() {
        assert!(!is_safe_file(Path::new("library.dll")));
    }

    #[test]
    fn test_is_safe_file_rejects_zip_directly() {
        assert!(!is_safe_file(Path::new("archive.zip")));
    }

    #[test]
    fn test_is_safe_file_case_insensitive() {
        assert!(is_safe_file(Path::new("file.TXT")));
        assert!(is_safe_file(Path::new("file.Log")));
    }

    #[test]
    fn test_is_safe_file_no_extension_rejected() {
        assert!(!is_safe_file(Path::new("Makefile")));
    }

    #[test]
    fn test_extract_text_plain_file() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("tftsr-test-extract-{}.txt", uuid::Uuid::now_v7()));
        std::fs::write(&path, "hello world").unwrap();
        let result = extract_text_content(&path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello world");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_extract_text_unsupported_binary_returns_error() {
        let result = extract_text_content(Path::new("data.xlsx"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not yet supported"));
    }
}
