use tauri::State;
use tracing::warn;

use crate::ai::provider::create_provider;
use crate::ai::{AnalysisResult, ChatResponse, Message, ProviderInfo};
use crate::db::models::{AiConversation, AiMessage, AuditEntry};
use crate::state::{AppState, ProviderConfig};

#[tauri::command]
pub async fn analyze_logs(
    issue_id: String,
    log_file_ids: Vec<String>,
    provider_config: ProviderConfig,
    state: State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    // Load log file contents
    let mut log_contents = String::new();
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for file_id in &log_file_ids {
            let mut stmt = db
                .prepare("SELECT file_name, file_path FROM log_files WHERE id = ?1")
                .map_err(|e| e.to_string())?;
            if let Ok((name, path)) = stmt.query_row([file_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            }) {
                log_contents.push_str(&format!("--- {name} ---\n"));
                if let Ok(content) = std::fs::read_to_string(&path) {
                    log_contents.push_str(&content);
                } else {
                    log_contents.push_str("[Could not read file]\n");
                }
                log_contents.push('\n');
            }
        }
    }

    let provider = create_provider(&provider_config);

    let messages = vec![
        Message {
            role: "system".into(),
            content: "You are an expert IT engineer. Analyze the following log content and \
                      identify key issues, errors, and anomalies. Structure your response as: \
                      SUMMARY: (one paragraph), KEY_FINDINGS: (bullet list), \
                      FIRST_WHY: (initial why question for 5-whys analysis), \
                      SEVERITY: (critical/high/medium/low)"
                .into(),
        },
        Message {
            role: "user".into(),
            content: format!("Analyze logs for issue {issue_id}:\n\n{log_contents}"),
        },
    ];

    let response = provider
        .chat(messages, &provider_config)
        .await
        .map_err(|e| {
            warn!(error = %e, "ai analyze_logs provider request failed");
            "AI analysis request failed".to_string()
        })?;

    let content = &response.content;
    let summary = extract_section(content, "SUMMARY:").unwrap_or_else(|| {
        content
            .lines()
            .next()
            .unwrap_or("Analysis complete")
            .to_string()
    });
    let key_findings = extract_list(content, "KEY_FINDINGS:");
    let suggested_why1 = extract_section(content, "FIRST_WHY:")
        .unwrap_or_else(|| "Why did this issue occur?".to_string());
    let severity_assessment =
        extract_section(content, "SEVERITY:").unwrap_or_else(|| "medium".to_string());

    // Write audit entry
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let entry = AuditEntry::new(
            "ai_analyze_logs".to_string(),
            "issue".to_string(),
            issue_id.clone(),
            serde_json::json!({ "log_file_ids": log_file_ids, "provider": provider_config.name })
                .to_string(),
        );
        crate::audit::log::write_audit_event(
            &db,
            &entry.action,
            &entry.entity_type,
            &entry.entity_id,
            &entry.details,
        )
        .map_err(|_| "Failed to write security audit entry".to_string())?;
    }

    Ok(AnalysisResult {
        summary,
        key_findings,
        suggested_why1,
        severity_assessment,
    })
}

fn extract_section(text: &str, header: &str) -> Option<String> {
    let start = text.find(header)?;
    let after = &text[start + header.len()..];
    let end = after.find('\n').unwrap_or(after.len());
    let section = after[..end].trim().to_string();
    if section.is_empty() {
        None
    } else {
        Some(section)
    }
}

fn extract_list(text: &str, header: &str) -> Vec<String> {
    let start = match text.find(header) {
        Some(s) => s + header.len(),
        None => return vec![],
    };
    let section = &text[start..];
    section
        .lines()
        .skip(1)
        .take_while(|l| {
            !l.chars()
                .next()
                .map(|c| c.is_uppercase() && l.contains(':'))
                .unwrap_or(false)
        })
        .filter(|l| l.trim_start().starts_with('-') || l.trim_start().starts_with('*'))
        .map(|l| {
            l.trim_start_matches(|c: char| c == '-' || c == '*' || c.is_whitespace())
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

#[tauri::command]
pub async fn chat_message(
    issue_id: String,
    message: String,
    provider_config: ProviderConfig,
    state: State<'_, AppState>,
) -> Result<ChatResponse, String> {
    // Find or create a conversation for this issue + provider
    let conversation_id = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let existing: Option<String> = db
            .prepare("SELECT id FROM ai_conversations WHERE issue_id = ?1 AND provider = ?2 AND model = ?3 ORDER BY created_at DESC LIMIT 1")
            .and_then(|mut stmt| {
                stmt.query_row(
                    rusqlite::params![issue_id, provider_config.name, provider_config.model],
                    |row| row.get(0),
                )
            })
            .ok();

        match existing {
            Some(id) => id,
            None => {
                let conv = AiConversation::new(
                    issue_id.clone(),
                    provider_config.name.clone(),
                    provider_config.model.clone(),
                );
                db.execute(
                    "INSERT INTO ai_conversations (id, issue_id, provider, model, created_at, title) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        conv.id, conv.issue_id, conv.provider, conv.model, conv.created_at, conv.title
                    ],
                )
                .map_err(|e| e.to_string())?;
                conv.id
            }
        }
    };

    // Load conversation history (use and_then to keep stmt lifetime within closure)
    let history: Vec<Message> = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let raw: Vec<(String, String)> = db
            .prepare(
                "SELECT role, content FROM ai_messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
            )
            .and_then(|mut stmt| {
                stmt.query_map([&conversation_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map(|rows| rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
            })
            .unwrap_or_default();
        drop(db);
        raw.into_iter()
            .map(|(role, content)| Message { role, content })
            .collect()
    };

    let provider = create_provider(&provider_config);

    let mut messages = history;
    messages.push(Message {
        role: "user".into(),
        content: message.clone(),
    });

    let response = provider
        .chat(messages, &provider_config)
        .await
        .map_err(|e| {
            warn!(error = %e, "ai chat provider request failed");
            "AI provider request failed".to_string()
        })?;

    // Save both user message and response to DB
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let user_msg = AiMessage::new(conversation_id.clone(), "user".to_string(), message);
        let asst_msg = AiMessage::new(
            conversation_id,
            "assistant".to_string(),
            response.content.clone(),
        );

        db.execute(
            "INSERT INTO ai_messages (id, conversation_id, role, content, token_count, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                user_msg.id, user_msg.conversation_id, user_msg.role,
                user_msg.content, user_msg.token_count, user_msg.created_at
            ],
        )
        .map_err(|e| e.to_string())?;

        db.execute(
            "INSERT INTO ai_messages (id, conversation_id, role, content, token_count, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                asst_msg.id, asst_msg.conversation_id, asst_msg.role,
                asst_msg.content, asst_msg.token_count, asst_msg.created_at
            ],
        )
        .map_err(|e| e.to_string())?;

        // Audit - capture full transmission details
        let audit_details = serde_json::json!({
            "provider": provider_config.name,
            "model": provider_config.model,
            "api_url": provider_config.api_url,
            "user_message": user_msg.content,
            "response_preview": if response.content.len() > 200 {
                format!("{preview}...", preview = &response.content[..200])
            } else {
                response.content.clone()
            },
            "token_count": user_msg.token_count,
        });
        let entry = AuditEntry::new(
            "ai_chat".to_string(),
            "issue".to_string(),
            issue_id,
            audit_details.to_string(),
        );
        if let Err(err) = crate::audit::log::write_audit_event(
            &db,
            &entry.action,
            &entry.entity_type,
            &entry.entity_id,
            &entry.details,
        ) {
            warn!(error = %err, "failed to write ai_chat audit entry");
        }
    }

    Ok(response)
}

#[tauri::command]
pub async fn test_provider_connection(
    provider_config: ProviderConfig,
) -> Result<ChatResponse, String> {
    let provider = create_provider(&provider_config);
    let messages = vec![Message {
        role: "user".into(),
        content:
            "Reply with exactly: Troubleshooting and RCA Assistant connection test successful."
                .into(),
    }];
    provider
        .chat(messages, &provider_config)
        .await
        .map_err(|e| {
            warn!(error = %e, "ai test_provider_connection failed");
            "Provider connection test failed".to_string()
        })
}

#[tauri::command]
pub async fn list_providers() -> Result<Vec<ProviderInfo>, String> {
    Ok(vec![
        ProviderInfo {
            name: "openai".to_string(),
            supports_streaming: true,
            models: vec!["gpt-4o".to_string(), "gpt-4o-mini".to_string()],
        },
        ProviderInfo {
            name: "anthropic".to_string(),
            supports_streaming: true,
            models: vec![
                "claude-3-5-sonnet-20241022".to_string(),
                "claude-3-haiku-20240307".to_string(),
            ],
        },
        ProviderInfo {
            name: "gemini".to_string(),
            supports_streaming: false,
            models: vec!["gemini-1.5-pro".to_string(), "gemini-1.5-flash".to_string()],
        },
        ProviderInfo {
            name: "mistral".to_string(),
            supports_streaming: true,
            models: vec![
                "mistral-large-latest".to_string(),
                "mistral-small-latest".to_string(),
            ],
        },
        ProviderInfo {
            name: "ollama".to_string(),
            supports_streaming: false,
            models: vec!["llama3.2:3b".to_string(), "llama3.1:8b".to_string()],
        },
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_section_basic() {
        let text = "SUMMARY: The server crashed due to OOM.\nKEY_FINDINGS:\n- item";
        assert_eq!(
            extract_section(text, "SUMMARY:"),
            Some("The server crashed due to OOM.".to_string())
        );
    }

    #[test]
    fn test_extract_section_missing() {
        let text = "No matching header here";
        assert_eq!(extract_section(text, "SUMMARY:"), None);
    }

    #[test]
    fn test_extract_section_empty_value() {
        let text = "SUMMARY:\nKEY_FINDINGS:";
        assert_eq!(extract_section(text, "SUMMARY:"), None);
    }

    #[test]
    fn test_extract_section_last_line() {
        let text = "SEVERITY: critical";
        assert_eq!(
            extract_section(text, "SEVERITY:"),
            Some("critical".to_string())
        );
    }

    #[test]
    fn test_extract_list_basic() {
        let text = "KEY_FINDINGS:\n- First issue\n- Second issue\nSEVERITY: high";
        let list = extract_list(text, "KEY_FINDINGS:");
        assert_eq!(list, vec!["First issue", "Second issue"]);
    }

    #[test]
    fn test_extract_list_with_asterisks() {
        let text = "KEY_FINDINGS:\n* Item one\n* Item two\nFIRST_WHY: Why?";
        let list = extract_list(text, "KEY_FINDINGS:");
        assert_eq!(list, vec!["Item one", "Item two"]);
    }

    #[test]
    fn test_extract_list_missing_header() {
        let text = "No findings here";
        let list = extract_list(text, "KEY_FINDINGS:");
        assert!(list.is_empty());
    }

    #[test]
    fn test_extract_list_empty_items_filtered() {
        let text = "KEY_FINDINGS:\n- \n- Actual item\n-  \nNEXT:";
        let list = extract_list(text, "KEY_FINDINGS:");
        assert_eq!(list, vec!["Actual item"]);
    }
}
