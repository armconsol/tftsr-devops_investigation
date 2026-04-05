use tauri::State;

use crate::db::models::{
    AiConversation, AiMessage, Issue, IssueDetail, IssueFilter, IssueSummary, IssueUpdate, LogFile,
    ResolutionStep,
};
use crate::state::AppState;

#[tauri::command]
pub async fn create_issue(
    title: String,
    description: String,
    severity: String,
    category: String,
    state: State<'_, AppState>,
) -> Result<Issue, String> {
    let issue = Issue::new(title, description, severity, category);

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO issues (id, title, description, severity, status, category, source, created_at, updated_at, resolved_at, assigned_to, tags) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            issue.id,
            issue.title,
            issue.description,
            issue.severity,
            issue.status,
            issue.category,
            issue.source,
            issue.created_at,
            issue.updated_at,
            issue.resolved_at,
            issue.assigned_to,
            issue.tags,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(issue)
}

#[tauri::command]
pub async fn get_issue(
    issue_id: String,
    state: State<'_, AppState>,
) -> Result<IssueDetail, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Load issue
    let mut stmt = db
        .prepare(
            "SELECT id, title, description, severity, status, category, source, \
             created_at, updated_at, resolved_at, assigned_to, tags \
             FROM issues WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let issue = stmt
        .query_row([&issue_id], |row| {
            Ok(Issue {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                severity: row.get(3)?,
                status: row.get(4)?,
                category: row.get(5)?,
                source: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                resolved_at: row.get(9)?,
                assigned_to: row.get(10)?,
                tags: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?;

    // Load log files
    let mut lf_stmt = db
        .prepare(
            "SELECT id, issue_id, file_name, file_path, file_size, mime_type, content_hash, uploaded_at, redacted \
             FROM log_files WHERE issue_id = ?1 ORDER BY uploaded_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let log_files: Vec<LogFile> = lf_stmt
        .query_map([&issue_id], |row| {
            Ok(LogFile {
                id: row.get(0)?,
                issue_id: row.get(1)?,
                file_name: row.get(2)?,
                file_path: row.get(3)?,
                file_size: row.get(4)?,
                mime_type: row.get(5)?,
                content_hash: row.get(6)?,
                uploaded_at: row.get(7)?,
                redacted: row.get::<_, i32>(8)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Load resolution steps (5-whys)
    let mut rs_stmt = db
        .prepare(
            "SELECT id, issue_id, step_order, why_question, answer, evidence, created_at \
             FROM resolution_steps WHERE issue_id = ?1 ORDER BY step_order ASC",
        )
        .map_err(|e| e.to_string())?;
    let resolution_steps: Vec<ResolutionStep> = rs_stmt
        .query_map([&issue_id], |row| {
            Ok(ResolutionStep {
                id: row.get(0)?,
                issue_id: row.get(1)?,
                step_order: row.get(2)?,
                why_question: row.get(3)?,
                answer: row.get(4)?,
                evidence: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Load conversations
    let mut conv_stmt = db
        .prepare(
            "SELECT id, issue_id, provider, model, created_at, title \
             FROM ai_conversations WHERE issue_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let conversations: Vec<AiConversation> = conv_stmt
        .query_map([&issue_id], |row| {
            Ok(AiConversation {
                id: row.get(0)?,
                issue_id: row.get(1)?,
                provider: row.get(2)?,
                model: row.get(3)?,
                created_at: row.get(4)?,
                title: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(IssueDetail {
        issue,
        log_files,
        resolution_steps,
        conversations,
    })
}

#[tauri::command]
pub async fn update_issue(
    issue_id: String,
    updates: IssueUpdate,
    state: State<'_, AppState>,
) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if let Some(ref title) = updates.title {
        db.execute(
            "UPDATE issues SET title = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![title, now, issue_id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(ref description) = updates.description {
        db.execute(
            "UPDATE issues SET description = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![description, now, issue_id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(ref severity) = updates.severity {
        db.execute(
            "UPDATE issues SET severity = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![severity, now, issue_id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(ref status) = updates.status {
        db.execute(
            "UPDATE issues SET status = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![status, now, issue_id],
        )
        .map_err(|e| e.to_string())?;
        if status == "resolved" {
            db.execute(
                "UPDATE issues SET resolved_at = ?1 WHERE id = ?2",
                rusqlite::params![now, issue_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    if let Some(ref category) = updates.category {
        db.execute(
            "UPDATE issues SET category = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![category, now, issue_id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(ref assigned_to) = updates.assigned_to {
        db.execute(
            "UPDATE issues SET assigned_to = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![assigned_to, now, issue_id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(ref tags) = updates.tags {
        db.execute(
            "UPDATE issues SET tags = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![tags, now, issue_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Fetch and return updated issue
    let mut stmt = db
        .prepare(
            "SELECT id, title, description, severity, status, category, source, \
             created_at, updated_at, resolved_at, assigned_to, tags \
             FROM issues WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_row([&issue_id], |row| {
        Ok(Issue {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            severity: row.get(3)?,
            status: row.get(4)?,
            category: row.get(5)?,
            source: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            resolved_at: row.get(9)?,
            assigned_to: row.get(10)?,
            tags: row.get(11)?,
        })
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_issue(issue_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Delete related records (CASCADE should handle this, but be explicit)
    db.execute("DELETE FROM ai_messages WHERE conversation_id IN (SELECT id FROM ai_conversations WHERE issue_id = ?1)", [&issue_id])
        .map_err(|e| e.to_string())?;
    db.execute(
        "DELETE FROM ai_conversations WHERE issue_id = ?1",
        [&issue_id],
    )
    .map_err(|e| e.to_string())?;
    db.execute(
        "DELETE FROM pii_spans WHERE log_file_id IN (SELECT id FROM log_files WHERE issue_id = ?1)",
        [&issue_id],
    )
    .map_err(|e| e.to_string())?;
    db.execute("DELETE FROM log_files WHERE issue_id = ?1", [&issue_id])
        .map_err(|e| e.to_string())?;
    db.execute(
        "DELETE FROM resolution_steps WHERE issue_id = ?1",
        [&issue_id],
    )
    .map_err(|e| e.to_string())?;
    db.execute("DELETE FROM issues WHERE id = ?1", [&issue_id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_issues(
    filter: IssueFilter,
    state: State<'_, AppState>,
) -> Result<Vec<IssueSummary>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let limit = filter.limit.unwrap_or(50);
    let offset = filter.offset.unwrap_or(0);

    let mut sql = String::from(
        "SELECT i.id, i.title, i.severity, i.status, i.category, i.created_at, i.updated_at, \
         (SELECT COUNT(*) FROM log_files lf WHERE lf.issue_id = i.id) as log_count, \
         (SELECT COUNT(*) FROM resolution_steps rs WHERE rs.issue_id = i.id) as step_count \
         FROM issues i WHERE 1=1",
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if let Some(ref status) = filter.status {
        sql.push_str(&format!(
            " AND i.status = ?{index}",
            index = params.len() + 1
        ));
        params.push(Box::new(status.clone()));
    }
    if let Some(ref severity) = filter.severity {
        sql.push_str(&format!(
            " AND i.severity = ?{index}",
            index = params.len() + 1
        ));
        params.push(Box::new(severity.clone()));
    }
    if let Some(ref category) = filter.category {
        sql.push_str(&format!(
            " AND i.category = ?{index}",
            index = params.len() + 1
        ));
        params.push(Box::new(category.clone()));
    }
    if let Some(ref domain) = filter.domain {
        sql.push_str(&format!(
            " AND i.category = ?{index}",
            index = params.len() + 1
        ));
        params.push(Box::new(domain.clone()));
    }
    if let Some(ref search) = filter.search {
        let pattern = format!("%{search}%");
        sql.push_str(&format!(
            " AND (i.title LIKE ?{0} OR i.description LIKE ?{0} OR i.category LIKE ?{0})",
            params.len() + 1
        ));
        params.push(Box::new(pattern));
    }

    sql.push_str(" ORDER BY i.updated_at DESC");
    sql.push_str(&format!(
        " LIMIT ?{limit_index} OFFSET ?{offset_index}",
        limit_index = params.len() + 1,
        offset_index = params.len() + 2
    ));
    params.push(Box::new(limit));
    params.push(Box::new(offset));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;
    let issues = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(IssueSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                severity: row.get(2)?,
                status: row.get(3)?,
                category: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                log_count: row.get(7)?,
                step_count: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(issues)
}

#[tauri::command]
pub async fn search_issues(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<IssueSummary>, String> {
    let filter = IssueFilter {
        search: Some(query),
        limit: Some(50),
        ..Default::default()
    };
    list_issues(filter, state).await
}

#[tauri::command]
pub async fn get_issue_messages(
    issue_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<AiMessage>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare(
            "SELECT am.id, am.conversation_id, am.role, am.content, am.token_count, am.created_at \
             FROM ai_messages am \
             JOIN ai_conversations ac ON ac.id = am.conversation_id \
             WHERE ac.issue_id = ?1 \
             ORDER BY am.created_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let messages = stmt
        .query_map([&issue_id], |row| {
            Ok(AiMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                token_count: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(messages)
}

#[tauri::command]
pub async fn add_five_why(
    issue_id: String,
    step_order: i64,
    why_question: String,
    answer: String,
    evidence: String,
    state: State<'_, AppState>,
) -> Result<ResolutionStep, String> {
    let step = ResolutionStep::new(issue_id.clone(), step_order, why_question, answer, evidence);

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO resolution_steps (id, issue_id, step_order, why_question, answer, evidence, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            step.id,
            step.issue_id,
            step.step_order,
            step.why_question,
            step.answer,
            step.evidence,
            step.created_at,
        ],
    )
    .map_err(|e| e.to_string())?;

    // Update issue timestamp
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    db.execute(
        "UPDATE issues SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, issue_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(step)
}

#[tauri::command]
pub async fn update_five_why(
    step_id: String,
    answer: String,
    evidence: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    if let Some(ref ev) = evidence {
        db.execute(
            "UPDATE resolution_steps SET answer = ?1, evidence = ?2 WHERE id = ?3",
            rusqlite::params![answer, ev, step_id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        db.execute(
            "UPDATE resolution_steps SET answer = ?1 WHERE id = ?2",
            rusqlite::params![answer, step_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn add_timeline_event(
    issue_id: String,
    event_type: String,
    description: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Use audit_log for timeline tracking
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let entry = crate::db::models::AuditEntry::new(
        event_type,
        "issue".to_string(),
        issue_id.clone(),
        serde_json::json!({ "description": description }).to_string(),
    );
    crate::audit::log::write_audit_event(
        &db,
        &entry.action,
        &entry.entity_type,
        &entry.entity_id,
        &entry.details,
    )
    .map_err(|_| "Failed to write security audit entry".to_string())?;

    // Update issue timestamp
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    db.execute(
        "UPDATE issues SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, issue_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
