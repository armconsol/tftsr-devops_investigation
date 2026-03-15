use tauri::State;

use crate::db::models::AuditEntry;
use crate::docs::{exporter, generate_postmortem_markdown, generate_rca_markdown};
use crate::state::AppState;

use serde::{Deserialize, Serialize};

/// Document record returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub issue_id: String,
    pub doc_type: String,
    pub title: String,
    pub content_md: String,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn generate_rca(
    issue_id: String,
    state: State<'_, AppState>,
) -> Result<Document, String> {
    let issue_detail = super::db::get_issue(issue_id.clone(), state.clone()).await?;

    let content_md = generate_rca_markdown(&issue_detail);

    let doc_id = uuid::Uuid::now_v7().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let document = Document {
        id: doc_id.clone(),
        issue_id: issue_id.clone(),
        doc_type: "rca".to_string(),
        title: format!("RCA: {}", issue_detail.issue.title),
        content_md: content_md.clone(),
        created_at: now.clone(),
        updated_at: now,
    };

    // Audit
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let entry = AuditEntry::new(
        "generate_rca".to_string(),
        "document".to_string(),
        doc_id,
        serde_json::json!({ "issue_id": issue_id }).to_string(),
    );
    let _ = db.execute(
        "INSERT INTO audit_log (id, timestamp, action, entity_type, entity_id, user_id, details) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            entry.id, entry.timestamp, entry.action,
            entry.entity_type, entry.entity_id, entry.user_id, entry.details
        ],
    );

    Ok(document)
}

#[tauri::command]
pub async fn generate_postmortem(
    issue_id: String,
    state: State<'_, AppState>,
) -> Result<Document, String> {
    let issue_detail = super::db::get_issue(issue_id.clone(), state.clone()).await?;

    let content_md = generate_postmortem_markdown(&issue_detail);

    let doc_id = uuid::Uuid::now_v7().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let document = Document {
        id: doc_id.clone(),
        issue_id: issue_id.clone(),
        doc_type: "postmortem".to_string(),
        title: format!("Post-Mortem: {}", issue_detail.issue.title),
        content_md: content_md.clone(),
        created_at: now.clone(),
        updated_at: now,
    };

    // Audit
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let entry = AuditEntry::new(
        "generate_postmortem".to_string(),
        "document".to_string(),
        doc_id,
        serde_json::json!({ "issue_id": issue_id }).to_string(),
    );
    let _ = db.execute(
        "INSERT INTO audit_log (id, timestamp, action, entity_type, entity_id, user_id, details) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            entry.id, entry.timestamp, entry.action,
            entry.entity_type, entry.entity_id, entry.user_id, entry.details
        ],
    );

    Ok(document)
}

#[tauri::command]
pub async fn update_document(
    doc_id: String,
    content_md: String,
) -> Result<(), String> {
    // Documents are generated on-demand and held in memory / frontend state.
    // This is a no-op placeholder. In a future version with a documents table,
    // this would persist updates.
    if doc_id.is_empty() || content_md.is_empty() {
        return Err("doc_id and content_md are required".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn export_document(
    title: String,
    content_md: String,
    format: String,
    output_dir: String,
) -> Result<String, String> {
    let safe_title: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let output_path = match format.as_str() {
        "markdown" | "md" => {
            let path = format!("{}/{}.md", output_dir, safe_title);
            exporter::export_markdown(&content_md, &path).map_err(|e| e.to_string())?;
            path
        }
        "pdf" => {
            let path = format!("{}/{}.pdf", output_dir, safe_title);
            exporter::export_pdf(&content_md, &title, &path).map_err(|e| e.to_string())?;
            path
        }
        _ => return Err(format!("Unsupported export format: {}", format)),
    };

    Ok(output_path)
}
