use rusqlite::OptionalExtension;
use tauri::{Manager, State};
use tracing::warn;

use crate::ai::agents::create_agent_registry;
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
    // Load log file contents — only redacted files may be sent to an AI provider
    let mut log_contents = String::new();
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for file_id in &log_file_ids {
            let mut stmt = db
                .prepare("SELECT file_name, file_path, redacted FROM log_files WHERE id = ?1")
                .map_err(|e| e.to_string())?;
            if let Ok((name, path, redacted)) = stmt.query_row([file_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i32>(2)? != 0,
                ))
            }) {
                let redacted_path = redacted_path_for(&name, &path, redacted)?;
                log_contents.push_str(&format!("--- {name} ---\n"));
                if let Ok(content) = std::fs::read_to_string(&redacted_path) {
                    log_contents.push_str(&content);
                } else {
                    log_contents.push_str("[Could not read redacted file]\n");
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
            tool_call_id: None,
            tool_calls: None,
        },
        Message {
            role: "user".into(),
            content: format!("Analyze logs for issue {issue_id}:\n\n{log_contents}"),
            tool_call_id: None,
            tool_calls: None,
        },
    ];

    let response = provider
        .chat(messages, &provider_config, None)
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

/// Returns the path to the `.redacted` file, or an error if the file has not been redacted.
fn redacted_path_for(name: &str, path: &str, redacted: bool) -> Result<String, String> {
    if !redacted {
        return Err(format!(
            "Log file '{name}' has not been scanned and redacted. \
             Run PII detection and apply redactions before sending to AI."
        ));
    }
    Ok(format!("{path}.redacted"))
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

/// Sanitize messages for final call when tool iteration limit is reached.
/// Converts tool role messages to assistant role with clear labeling as untrusted data.
fn sanitize_messages_for_final_call(messages: Vec<Message>) -> Vec<Message> {
    messages
        .into_iter()
        .map(|mut msg| {
            if msg.role == "tool" {
                // Convert tool output to assistant role with clear labeling as untrusted data
                msg.role = "assistant".into();
                msg.content = format!("[UNTRUSTED TOOL OUTPUT]: {}", msg.content);
                msg.tool_call_id = None;
            }
            msg.tool_calls = None; // Strip tool_calls from all messages
            msg
        })
        .collect()
}

#[tauri::command]
pub async fn chat_message(
    issue_id: String,
    message: String,
    log_file_ids: Option<Vec<String>>,
    provider_config: ProviderConfig,
    system_prompt: Option<String>,
    app_handle: tauri::AppHandle,
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

    // Load conversation history across ALL conversations for this issue
    let history: Vec<Message> = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let raw: Vec<(String, String)> = db
            .prepare(
                "SELECT am.role, am.content \
                 FROM ai_messages am \
                 JOIN ai_conversations ac ON ac.id = am.conversation_id \
                 WHERE ac.issue_id = ?1 \
                 ORDER BY am.created_at ASC",
            )
            .and_then(|mut stmt| {
                stmt.query_map([&issue_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map(|rows| rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
            })
            .unwrap_or_default();
        drop(db);
        raw.into_iter()
            .map(|(role, content)| Message {
                role,
                content,
                tool_call_id: None,
                tool_calls: None,
            })
            .collect()
    };

    // Auto-redact PII in both the typed message and any file attachments.
    // The backend is the sole authority for redaction; the frontend sends original content.
    let mut was_pii_redacted = false;
    let mut redacted_pii_types: Vec<String> = Vec::new();

    let full_message = {
        // Step 1: redact the typed user message text.
        let base = {
            let spans = crate::pii::PiiDetector::new().detect(&message);
            if spans.is_empty() {
                message.clone()
            } else {
                let types: std::collections::HashSet<&str> =
                    spans.iter().map(|s| s.pii_type.as_str()).collect();
                let mut type_list: Vec<&str> = types.into_iter().collect();
                type_list.sort_unstable();
                warn!(
                    pii_types = ?type_list,
                    pii_count = spans.len(),
                    "PII detected in typed chat message — auto-redacting before AI send"
                );
                was_pii_redacted = true;
                redacted_pii_types.extend(type_list.iter().map(|s| s.to_string()));
                crate::pii::apply_redactions(&message, &spans)
            }
        };

        // Step 2: load attachment files from DB, scan, and embed clean content.
        let files: Vec<(String, String)> = if let Some(ref ids) = log_file_ids {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let mut v = Vec::new();
            for file_id in ids {
                if let Ok((name, path)) = db
                    .prepare("SELECT file_name, file_path FROM log_files WHERE id = ?1")
                    .and_then(|mut s| {
                        s.query_row([file_id], |row| {
                            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                        })
                    })
                {
                    v.push((name, path));
                }
            }
            v
        } else {
            vec![]
        };

        // 8 KB embed limit; detect + redact on full content so PII at the boundary is caught.
        const EMBED_LIMIT: usize = 8_000;

        let mut msg = base;
        for (file_name, file_path) in &files {
            let content = std::fs::read_to_string(file_path).unwrap_or_default();
            let spans = crate::pii::PiiDetector::new().detect(&content);
            let redacted = if spans.is_empty() {
                content
            } else {
                let types: std::collections::HashSet<&str> =
                    spans.iter().map(|s| s.pii_type.as_str()).collect();
                let mut type_list: Vec<&str> = types.into_iter().collect();
                type_list.sort_unstable();
                warn!(
                    file_name = %file_name,
                    pii_types = ?type_list,
                    pii_count = spans.len(),
                    "PII detected in chat attachment — auto-redacting before AI send"
                );
                was_pii_redacted = true;
                redacted_pii_types.extend(type_list.iter().map(|s| s.to_string()));
                crate::pii::apply_redactions(&content, &spans)
            };
            // Truncate after redaction so the cut never lands inside a PII span.
            let embed_end = if redacted.len() > EMBED_LIMIT {
                let mut e = EMBED_LIMIT;
                while !redacted.is_char_boundary(e) {
                    e -= 1;
                }
                e
            } else {
                redacted.len()
            };
            msg.push_str(&format!(
                "\n\n--- Attached: {file_name} ---\n{}",
                &redacted[..embed_end]
            ));
        }
        msg
    };

    let provider = create_provider(&provider_config);

    // Search integration sources for relevant context
    let integration_context = search_integration_sources(&message, &app_handle, &state).await;

    // Load agent system
    let agent_registry = create_agent_registry();
    let devops_agent = agent_registry.get("devops-incident-responder");

    let mut messages = Vec::new();

    // Inject devops-incident-responder as primary system prompt (always)
    if let Some(agent) = devops_agent {
        messages.push(Message {
            role: "system".into(),
            content: agent.system_prompt.clone(),
            tool_call_id: None,
            tool_calls: None,
        });
    }

    // Inject domain system prompt if provided
    if let Some(ref prompt) = system_prompt {
        if !prompt.is_empty() {
            messages.push(Message {
                role: "system".into(),
                content: prompt.clone(),
                tool_call_id: None,
                tool_calls: None,
            });
        }
    }

    messages.extend(history);

    // If we found integration content, add it to the conversation context
    if !integration_context.is_empty() {
        let context_message = Message {
            role: "system".into(),
            content: format!(
                "INTERNAL DOCUMENTATION SOURCES:\n\n{integration_context}\n\n\
                 Instructions: The above content is from internal company documentation systems \
                 (Confluence, ServiceNow, Azure DevOps). \
                 \n\n**IMPORTANT**: First determine if this documentation is RELEVANT to the user's question:\
                 \n- If the documentation directly addresses the question → Use it and cite sources with URLs\
                 \n- If the documentation is tangentially related but doesn't answer the question → Briefly mention what internal docs exist, then provide a complete answer using general knowledge\
                 \n- If the documentation is completely unrelated → Ignore it and answer using general knowledge\
                 \n\nDo NOT force irrelevant internal documentation into your answer. The user needs accurate information, not forced citations."
            ),
            tool_call_id: None,
            tool_calls: None,
        };
        messages.push(context_message);
    }

    // Tool execution configuration
    const MAX_TOOL_ITERATIONS: usize = 20; // Allow sufficient iterations for complex diagnostics

    // Get available tools — static + MCP
    let mut all_tools = crate::ai::tools::get_available_tools();
    let mcp_tools = crate::ai::tools::get_enabled_mcp_tools(&state).await;
    all_tools.extend(mcp_tools);
    let tools = if all_tools.is_empty() {
        None
    } else {
        Some(all_tools)
    };

    // If tools are available AND using OpenAI-compatible provider, add explicit JSON format instruction
    // Only OpenAI-compatible providers (default case in create_provider) actually support tool calling.
    // Others (anthropic, gemini, mistral, ollama) either ignore tools or use provider-specific formats.
    let is_openai_compatible = {
        let kind = if provider_config.provider_type.is_empty() {
            provider_config.name.as_str()
        } else {
            provider_config.provider_type.as_str()
        };
        !matches!(kind, "anthropic" | "gemini" | "mistral" | "ollama")
    };

    if tools.is_some() && is_openai_compatible {
        messages.push(Message {
            role: "system".into(),
            content: "CRITICAL: You have tools available. When calling tools, you MUST use the native JSON function calling format in your API response. DO NOT output XML tags like <tool_name>. DO NOT output text descriptions of tool calls. Use the structured tool_calls field in your response.".into(),
            tool_call_id: None,
            tool_calls: None,
        });

        // Add iteration budget awareness
        messages.push(Message {
            role: "system".into(),
            content: format!(
                "TOOL EXECUTION BUDGET: You have a maximum of {MAX_TOOL_ITERATIONS} rounds (each AI response counts as one round). \
                 You can call multiple tools in a single round. \
                 Plan your investigation efficiently:\n\
                 - Call multiple related tools in the same round when possible\n\
                 - Prioritize high-value diagnostic commands first\n\
                 - Use comprehensive output formats (e.g., kubectl --output=yaml) to gather more data per call\n\
                 - Reserve 1 round for your final summary/answer\n\
                 - If you exceed the budget, you'll be cut off mid-investigation\n\
                 Current round count is not visible to you, so plan conservatively."
            ),
            tool_call_id: None,
            tool_calls: None,
        });
    }

    messages.push(Message {
        role: "user".into(),
        content: full_message.clone(),
        tool_call_id: None,
        tool_calls: None,
    });

    // Tool-calling loop: keep calling until AI gives final answer
    let final_response;
    let mut iteration = 0;

    loop {
        iteration += 1;

        // Warn AI when approaching limit
        if iteration == MAX_TOOL_ITERATIONS - 2 {
            messages.push(Message {
                role: "system".into(),
                content: format!(
                    "WARNING: You are on iteration {iteration}/{MAX_TOOL_ITERATIONS} (2 rounds remaining). \
                     You MUST provide your final answer in the NEXT round. \
                     Do NOT call any more tools. \
                     Summarize your findings based on the data you've already gathered."
                ),
                tool_call_id: None,
                tool_calls: None,
            });
        }

        // Force stop at limit with collected data
        if iteration > MAX_TOOL_ITERATIONS {
            let sanitized_messages = sanitize_messages_for_final_call(messages);

            // Add final instruction
            let mut final_messages = sanitized_messages;
            final_messages.push(Message {
                role: "system".into(),
                content: format!(
                    "CRITICAL: Tool iteration limit reached ({iteration}/{MAX_TOOL_ITERATIONS}). \
                     TOOLS ARE NOW DISABLED. \
                     You MUST respond now with a natural language summary of your findings. \
                     DO NOT attempt to call any tools - they will not execute. \
                     DO NOT emit tool_calls JSON - it will be ignored. \
                     Ignore any earlier instructions about tool calling or JSON formatting. \
                     Provide your best answer in plain text based on the diagnostic data already collected."
                ),
                tool_call_id: None,
                tool_calls: None,
            });

            // Make one final call WITHOUT tools to force text response
            let final_attempt = provider
                .chat(final_messages, &provider_config, None) // No tools available
                .await
                .map_err(|e| {
                    format!("AI provider request failed after reaching iteration limit: {e}")
                })?;

            final_response = final_attempt;
            tracing::warn!(
                "Tool iteration limit exceeded, forced final response: {} chars",
                final_response.content.len()
            );
            break;
        }

        let response = provider
            .chat(messages.clone(), &provider_config, tools.clone())
            .await
            .map_err(|e| {
                let error_msg = format!("AI provider request failed: {e}");
                warn!("{}", error_msg);
                error_msg
            })?;

        // Check if AI wants to call tools
        if let Some(tool_calls) = &response.tool_calls {
            tracing::info!("AI requested {} tool call(s)", tool_calls.len());

            // Execute each tool call
            for tool_call in tool_calls {
                tracing::info!("Executing tool: {}", tool_call.name);

                let tool_result = execute_tool_call(tool_call, &app_handle, &state).await;

                // Format result
                let result_content = match tool_result {
                    Ok(result) => result,
                    Err(e) => format!("Error executing tool: {e}"),
                };

                // Add tool result as a message
                messages.push(Message {
                    role: "tool".into(),
                    content: result_content,
                    tool_call_id: Some(tool_call.id.clone()),
                    tool_calls: None,
                });
            }

            // Continue loop to get AI's next response
            continue;
        }

        // No tool calls - this is the final answer
        final_response = response;
        break;
    }

    // Save both user message and response to DB
    let stored_user_message;
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let user_msg = AiMessage::new(conversation_id.clone(), "user".to_string(), full_message);
        stored_user_message = user_msg.content.clone();
        let asst_msg = AiMessage::new(
            conversation_id,
            "assistant".to_string(),
            final_response.content.clone(),
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
        let pii_types_for_audit = {
            use std::collections::HashSet;
            let mut v: Vec<String> = redacted_pii_types
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            v.sort_unstable();
            v
        };
        let audit_details = serde_json::json!({
            "provider": provider_config.name,
            "model": provider_config.model,
            "api_url": provider_config.api_url,
            "user_message": user_msg.content,
            "was_pii_redacted": was_pii_redacted,
            "pii_types_redacted": pii_types_for_audit,
            "response_preview": if final_response.content.len() > 200 {
                format!("{preview}...", preview = &final_response.content[..200])
            } else {
                final_response.content.clone()
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

    Ok(crate::ai::ChatResponse {
        user_message: Some(stored_user_message),
        ..final_response
    })
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
        tool_call_id: None,
        tool_calls: None,
    }];
    provider
        .chat(messages, &provider_config, None)
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

/// Search integration sources (Confluence, ServiceNow, Azure DevOps) for relevant context
async fn search_integration_sources(
    query: &str,
    app_handle: &tauri::AppHandle,
    state: &State<'_, AppState>,
) -> String {
    let mut all_results = Vec::new();

    // Try to get integration configurations
    let configs: Vec<crate::commands::integrations::IntegrationConfig> = {
        let db = match state.db.lock() {
            Ok(db) => db,
            Err(e) => {
                tracing::warn!("Failed to lock database: {}", e);
                return String::new();
            }
        };

        let mut stmt = match db.prepare(
            "SELECT service, base_url, username, project_name, space_key FROM integration_config",
        ) {
            Ok(stmt) => stmt,
            Err(e) => {
                tracing::warn!("Failed to prepare statement: {}", e);
                return String::new();
            }
        };

        let rows = match stmt.query_map([], |row| {
            Ok(crate::commands::integrations::IntegrationConfig {
                service: row.get(0)?,
                base_url: row.get(1)?,
                username: row.get(2)?,
                project_name: row.get(3)?,
                space_key: row.get(4)?,
            })
        }) {
            Ok(rows) => rows,
            Err(e) => {
                tracing::warn!("Failed to query integration configs: {}", e);
                return String::new();
            }
        };

        rows.filter_map(|r| r.ok()).collect()
    };

    // Search each available integration in parallel
    let mut search_tasks = Vec::new();

    for config in configs {
        // Authentication priority:
        // 1. Try cookies from persistent browser (may fail for HttpOnly)
        // 2. Try stored credentials from database
        // 3. Fall back to webview-based search (uses browser's session directly)

        let cookies_opt = match crate::commands::integrations::get_fresh_cookies_from_webview(
            &config.service,
            app_handle,
            state,
        )
        .await
        {
            Ok(Some(cookies)) => {
                tracing::info!("Using extracted cookies for {}", config.service);
                Some(cookies)
            }
            _ => {
                // Fallback: check for stored credentials in database
                tracing::info!(
                    "Cookie extraction failed for {}, checking stored credentials",
                    config.service
                );
                let encrypted_token: Option<String> = {
                    let db = match state.db.lock() {
                        Ok(db) => db,
                        Err(_) => continue,
                    };
                    db.query_row(
                        "SELECT encrypted_token FROM credentials WHERE service = ?1",
                        [&config.service],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()
                    .ok()
                    .flatten()
                };

                if let Some(token) = encrypted_token {
                    if let Ok(decrypted) = crate::integrations::auth::decrypt_token(&token) {
                        // Try to parse as cookies JSON
                        if let Ok(cookie_list) = serde_json::from_str::<
                            Vec<crate::integrations::webview_auth::Cookie>,
                        >(&decrypted)
                        {
                            tracing::info!(
                                "Using stored cookies for {} (count: {})",
                                config.service,
                                cookie_list.len()
                            );
                            Some(cookie_list)
                        } else {
                            tracing::warn!(
                                "Stored credentials for {} not in cookie format",
                                config.service
                            );
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };

        // If we have cookies (from extraction or database), use standard API search
        if let Some(cookies) = cookies_opt {
            match config.service.as_str() {
                "confluence" => {
                    let base_url = config.base_url.clone();
                    let query = query.to_string();
                    let cookies_clone = cookies.clone();
                    search_tasks.push(tokio::spawn(async move {
                        crate::integrations::confluence_search::search_confluence(
                            &base_url,
                            &query,
                            &cookies_clone,
                        )
                        .await
                        .unwrap_or_default()
                    }));
                }
                "servicenow" => {
                    let instance_url = config.base_url.clone();
                    let query = query.to_string();
                    let cookies_clone = cookies.clone();
                    search_tasks.push(tokio::spawn(async move {
                        let mut results = Vec::new();
                        // Search knowledge base
                        if let Ok(kb_results) =
                            crate::integrations::servicenow_search::search_servicenow(
                                &instance_url,
                                &query,
                                &cookies_clone,
                            )
                            .await
                        {
                            results.extend(kb_results);
                        }
                        // Search incidents
                        if let Ok(incident_results) =
                            crate::integrations::servicenow_search::search_incidents(
                                &instance_url,
                                &query,
                                &cookies_clone,
                            )
                            .await
                        {
                            results.extend(incident_results);
                        }
                        results
                    }));
                }
                "azuredevops" => {
                    let org_url = config.base_url.clone();
                    let project = config.project_name.unwrap_or_default();
                    let query = query.to_string();
                    let cookies_clone = cookies.clone();
                    search_tasks.push(tokio::spawn(async move {
                        let mut results = Vec::new();
                        // Search wiki
                        if let Ok(wiki_results) =
                            crate::integrations::azuredevops_search::search_wiki(
                                &org_url,
                                &project,
                                &query,
                                &cookies_clone,
                            )
                            .await
                        {
                            results.extend(wiki_results);
                        }
                        // Search work items
                        if let Ok(wi_results) =
                            crate::integrations::azuredevops_search::search_work_items(
                                &org_url,
                                &project,
                                &query,
                                &cookies_clone,
                            )
                            .await
                        {
                            results.extend(wi_results);
                        }
                        results
                    }));
                }
                _ => {}
            }
        } else {
            // Final fallback: try webview-based fetch (includes HttpOnly cookies automatically)
            // This makes HTTP requests FROM the authenticated webview, which includes all cookies
            tracing::info!(
                "No extracted cookies for {}, trying webview-based fetch",
                config.service
            );

            // Check if webview exists for this service
            let webview_label = {
                let webviews = match state.integration_webviews.lock() {
                    Ok(w) => w,
                    Err(_) => continue,
                };
                webviews.get(&config.service).cloned()
            };

            if let Some(label) = webview_label {
                // Get window handle
                if let Some(webview_window) = app_handle.get_webview_window(&label) {
                    let base_url = config.base_url.clone();
                    let service = config.service.clone();
                    let query_str = query.to_string();

                    match service.as_str() {
                        "confluence" => {
                            search_tasks.push(tokio::spawn(async move {
                                tracing::info!("Executing Confluence search via webview fetch");
                                match crate::integrations::webview_fetch::search_confluence_webview(
                                    &webview_window,
                                    &base_url,
                                    &query_str,
                                )
                                .await
                                {
                                    Ok(results) => {
                                        tracing::info!(
                                            "Webview fetch for Confluence returned {} results",
                                            results.len()
                                        );
                                        results
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "Webview fetch failed for Confluence: {}",
                                            e
                                        );
                                        Vec::new()
                                    }
                                }
                            }));
                        }
                        "servicenow" => {
                            search_tasks.push(tokio::spawn(async move {
                                tracing::info!("Executing ServiceNow search via webview fetch");
                                match crate::integrations::webview_fetch::search_servicenow_webview(
                                    &webview_window,
                                    &base_url,
                                    &query_str,
                                )
                                .await
                                {
                                    Ok(results) => {
                                        tracing::info!(
                                            "Webview fetch for ServiceNow returned {} results",
                                            results.len()
                                        );
                                        results
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "Webview fetch failed for ServiceNow: {}",
                                            e
                                        );
                                        Vec::new()
                                    }
                                }
                            }));
                        }
                        "azuredevops" => {
                            let project = config.project_name.unwrap_or_default();
                            search_tasks.push(tokio::spawn(async move {
                                tracing::info!("Executing Azure DevOps search via webview fetch");
                                let mut results = Vec::new();

                                // Search wiki
                                match crate::integrations::webview_fetch::search_azuredevops_wiki_webview(
                                    &webview_window,
                                    &base_url,
                                    &project,
                                    &query_str
                                ).await {
                                    Ok(wiki_results) => {
                                        tracing::info!("Webview fetch for ADO wiki returned {} results", wiki_results.len());
                                        results.extend(wiki_results);
                                    }
                                    Err(e) => {
                                        tracing::warn!("Webview fetch failed for ADO wiki: {}", e);
                                    }
                                }

                                // Search work items
                                match crate::integrations::webview_fetch::search_azuredevops_workitems_webview(
                                    &webview_window,
                                    &base_url,
                                    &project,
                                    &query_str
                                ).await {
                                    Ok(wi_results) => {
                                        tracing::info!("Webview fetch for ADO work items returned {} results", wi_results.len());
                                        results.extend(wi_results);
                                    }
                                    Err(e) => {
                                        tracing::warn!("Webview fetch failed for ADO work items: {}", e);
                                    }
                                }

                                results
                            }));
                        }
                        _ => {}
                    }
                } else {
                    tracing::warn!("Webview window not found for {}", config.service);
                }
            } else {
                tracing::warn!(
                    "No webview open for {} - cannot search. Please open browser window in Settings → Integrations",
                    config.service
                );
            }
        }
    }

    // Wait for all searches to complete
    for task in search_tasks {
        if let Ok(results) = task.await {
            all_results.extend(results);
        }
    }

    // Format results for AI context
    if all_results.is_empty() {
        return String::new();
    }

    let mut context = String::new();
    for (idx, result) in all_results.iter().enumerate() {
        context.push_str(&format!("--- SOURCE {} ({}) ---\n", idx + 1, result.source));
        context.push_str(&format!("Title: {}\n", result.title));
        context.push_str(&format!("URL: {}\n", result.url));

        if let Some(content) = &result.content {
            context.push_str(&format!("Content:\n{content}\n\n"));
        } else {
            context.push_str(&format!("Excerpt: {}\n\n", result.excerpt));
        }
    }

    tracing::info!(
        "Found {} integration sources for AI context",
        all_results.len()
    );
    context
}

/// Execute a tool call made by the AI
async fn execute_tool_call(
    tool_call: &crate::ai::ToolCall,
    app_handle: &tauri::AppHandle,
    app_state: &State<'_, AppState>,
) -> Result<String, String> {
    match tool_call.name.as_str() {
        "add_ado_comment" => {
            // Parse arguments
            let args: serde_json::Value = serde_json::from_str(&tool_call.arguments)
                .map_err(|e| format!("Failed to parse tool arguments: {e}"))?;

            let work_item_id = args
                .get("work_item_id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| "Missing or invalid work_item_id parameter".to_string())?;

            let comment_text = args
                .get("comment_text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing or invalid comment_text parameter".to_string())?;

            // Execute the add_ado_comment command
            tracing::info!(
                "AI executing tool: add_ado_comment({}, \"{}\")",
                work_item_id,
                comment_text
            );
            crate::commands::integrations::add_ado_comment(
                work_item_id,
                comment_text.to_string(),
                app_handle.clone(),
                app_state.clone(),
            )
            .await
        }
        "execute_shell_command" => execute_shell_tool_call(tool_call, app_handle, app_state).await,
        name if name.starts_with("mcp_") => execute_mcp_tool_call(tool_call, app_state).await,
        _ => {
            let error = format!("Unknown tool: {}", tool_call.name);
            tracing::warn!("{error}");
            Err(error)
        }
    }
}

async fn execute_shell_tool_call(
    tool_call: &crate::ai::ToolCall,
    app_handle: &tauri::AppHandle,
    app_state: &State<'_, AppState>,
) -> Result<String, String> {
    // Parse arguments
    let args: serde_json::Value = serde_json::from_str(&tool_call.arguments)
        .map_err(|e| format!("Failed to parse tool arguments: {e}"))?;

    let command = args
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing command parameter".to_string())?;

    let working_directory = args.get("working_directory").and_then(|v| v.as_str());
    let kubeconfig_id = args.get("kubeconfig_id").and_then(|v| v.as_str());

    // PII detection
    {
        let detector = crate::pii::detector::PiiDetector::new();
        let spans = detector.detect(command);
        if !spans.is_empty() {
            tracing::warn!(
                tool = %tool_call.name,
                pii_spans = spans.len(),
                "PII detected in shell command"
            );
        }
    }

    // Audit log
    {
        let db = app_state.db.lock().map_err(|e| e.to_string())?;
        let details = serde_json::json!({
            "tool": tool_call.name,
            "command": command,
        });
        crate::audit::log::write_audit_event(
            &db,
            "shell_tool_call",
            "shell_command",
            command,
            &details.to_string(),
        )
        .map_err(|e| format!("Audit log failed: {e}"))?;
    }

    // Execute with approval flow
    let result = crate::shell::executor::execute_with_approval(
        command,
        app_handle,
        app_state.inner(),
        kubeconfig_id,
        working_directory,
    )
    .await?;

    // Format output for AI
    Ok(format!(
        "Exit Code: {}\n\nStdout:\n{}\n\nStderr:\n{}",
        result.exit_code, result.stdout, result.stderr
    ))
}

async fn execute_mcp_tool_call(
    tool_call: &crate::ai::ToolCall,
    app_state: &State<'_, AppState>,
) -> Result<String, String> {
    // PII scan — log warning if sensitive data detected (non-blocking)
    {
        let detector = crate::pii::detector::PiiDetector::new();
        let spans = detector.detect(&tool_call.arguments);
        if !spans.is_empty() {
            tracing::warn!(
                tool = %tool_call.name,
                pii_spans = spans.len(),
                "PII detected in MCP tool call arguments"
            );
        }
    }

    // Audit log — mandatory before any external call
    {
        let db = app_state.db.lock().map_err(|e| e.to_string())?;
        let details = serde_json::json!({
            "tool": tool_call.name,
            "args_preview": if tool_call.arguments.len() > 200 {
                format!("{}...", &tool_call.arguments[..200])
            } else {
                tool_call.arguments.clone()
            },
        });
        crate::audit::log::write_audit_event(
            &db,
            "mcp_tool_call",
            "mcp_tool",
            &tool_call.name,
            &details.to_string(),
        )
        .map_err(|e| format!("Audit log failed: {e}"))?;
    }

    // Look up the tool → server_id + raw tool name
    let (server_id, raw_tool_name) = {
        let db = app_state.db.lock().map_err(|e| e.to_string())?;
        let tool = crate::mcp::store::get_tool_by_key(&db, &tool_call.name)?
            .ok_or_else(|| format!("MCP tool not found: {}", tool_call.name))?;
        (tool.server_id, tool.name)
    };

    // Get connection from state — use tokio Mutex, never hold std::Mutex simultaneously
    let conn_arc = {
        let connections = app_state.mcp_connections.lock().await;
        connections
            .get(&server_id)
            .cloned()
            .ok_or_else(|| format!("No active connection for MCP server {server_id}"))?
    };

    let args: serde_json::Value = serde_json::from_str(&tool_call.arguments)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    let conn = conn_arc.lock().await;
    crate::mcp::client::call_tool(&conn, &raw_tool_name, &args).await
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
    fn test_redacted_path_rejects_unredacted_file() {
        let err = redacted_path_for("app.log", "/data/app.log", false).unwrap_err();
        assert!(err.contains("app.log"));
        assert!(err.contains("redacted"));
    }

    #[test]
    fn test_redacted_path_returns_dotredacted_suffix() {
        let path = redacted_path_for("app.log", "/data/app.log", true).unwrap();
        assert_eq!(path, "/data/app.log.redacted");
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

    #[test]
    fn test_history_query_same_conversation() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO issues (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["issue-1", "Test", &now, &now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_conversations (id, issue_id, provider, model, created_at, title) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["conv-1", "issue-1", "openai", "gpt-4o", &now, "Conv 1"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_messages (id, conversation_id, role, content, token_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["msg-1", "conv-1", "user", "Hello", 5, "2025-01-01 10:00:00"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_messages (id, conversation_id, role, content, token_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["msg-2", "conv-1", "assistant", "Hi there", 8, "2025-01-01 10:00:01"],
        )
        .unwrap();

        let issue_id = "issue-1";
        let raw: Vec<(String, String)> = conn
            .prepare(
                "SELECT am.role, am.content \
                 FROM ai_messages am \
                 JOIN ai_conversations ac ON ac.id = am.conversation_id \
                 WHERE ac.issue_id = ?1 \
                 ORDER BY am.created_at ASC",
            )
            .and_then(|mut stmt| {
                stmt.query_map([&issue_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map(|rows| rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
            })
            .unwrap();

        assert_eq!(raw.len(), 2);
        assert_eq!(raw[0], ("user".to_string(), "Hello".to_string()));
        assert_eq!(raw[1], ("assistant".to_string(), "Hi there".to_string()));
    }

    #[test]
    fn test_history_query_across_conversations() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO issues (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["issue-1", "Test", &now, &now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_conversations (id, issue_id, provider, model, created_at, title) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["conv-1", "issue-1", "openai", "gpt-4o", &now, "Conv 1"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_conversations (id, issue_id, provider, model, created_at, title) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["conv-2", "issue-1", "anthropic", "claude-3", &now, "Conv 2"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_messages (id, conversation_id, role, content, token_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["msg-1", "conv-1", "user", "From conv 1", 5, "2025-01-01 10:00:00"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_messages (id, conversation_id, role, content, token_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["msg-2", "conv-2", "user", "From conv 2", 5, "2025-01-01 11:00:00"],
        )
        .unwrap();

        let issue_id = "issue-1";
        let raw: Vec<(String, String)> = conn
            .prepare(
                "SELECT am.role, am.content \
                 FROM ai_messages am \
                 JOIN ai_conversations ac ON ac.id = am.conversation_id \
                 WHERE ac.issue_id = ?1 \
                 ORDER BY am.created_at ASC",
            )
            .and_then(|mut stmt| {
                stmt.query_map([&issue_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map(|rows| rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
            })
            .unwrap();

        assert_eq!(raw.len(), 2);
        assert_eq!(raw[0].1, "From conv 1");
        assert_eq!(raw[1].1, "From conv 2");
    }

    #[test]
    fn test_history_query_empty_for_new_issue() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO issues (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["issue-new", "Empty", &now, &now],
        )
        .unwrap();

        let issue_id = "issue-new";
        let raw: Vec<(String, String)> = conn
            .prepare(
                "SELECT am.role, am.content \
                 FROM ai_messages am \
                 JOIN ai_conversations ac ON ac.id = am.conversation_id \
                 WHERE ac.issue_id = ?1 \
                 ORDER BY am.created_at ASC",
            )
            .and_then(|mut stmt| {
                stmt.query_map([&issue_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map(|rows| rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
            })
            .unwrap();

        assert!(raw.is_empty());
    }

    #[test]
    fn test_history_query_ordered_by_created_at() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO issues (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["issue-1", "Test", &now, &now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_conversations (id, issue_id, provider, model, created_at, title) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["conv-1", "issue-1", "openai", "gpt-4o", &now, "Conv 1"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_conversations (id, issue_id, provider, model, created_at, title) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["conv-2", "issue-1", "anthropic", "claude-3", &now, "Conv 2"],
        )
        .unwrap();
        // Insert messages out of order: conv-2 message is earlier than conv-1 message
        conn.execute(
            "INSERT INTO ai_messages (id, conversation_id, role, content, token_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["msg-1", "conv-1", "user", "Second", 5, "2025-01-01 12:00:00"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_messages (id, conversation_id, role, content, token_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["msg-2", "conv-2", "user", "First", 5, "2025-01-01 09:00:00"],
        )
        .unwrap();

        let issue_id = "issue-1";
        let raw: Vec<(String, String)> = conn
            .prepare(
                "SELECT am.role, am.content \
                 FROM ai_messages am \
                 JOIN ai_conversations ac ON ac.id = am.conversation_id \
                 WHERE ac.issue_id = ?1 \
                 ORDER BY am.created_at ASC",
            )
            .and_then(|mut stmt| {
                stmt.query_map([&issue_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map(|rows| rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
            })
            .unwrap();

        assert_eq!(raw.len(), 2);
        assert_eq!(raw[0].1, "First");
        assert_eq!(raw[1].1, "Second");
    }

    #[test]
    fn test_message_sanitization_converts_tool_role_to_assistant() {
        // Messages with 'tool' role that would cause validation errors
        let messages = vec![
            Message {
                role: "user".into(),
                content: "What pods are running?".into(),
                tool_call_id: None,
                tool_calls: None,
            },
            Message {
                role: "assistant".into(),
                content: "".into(),
                tool_call_id: None,
                tool_calls: Some(vec![crate::ai::ToolCall {
                    id: "call_123".into(),
                    name: "execute_shell_command".into(),
                    arguments: r#"{"command":"kubectl get pods"}"#.into(),
                }]),
            },
            Message {
                role: "tool".into(),
                content: "pod1 Running\npod2 Running".into(),
                tool_call_id: Some("call_123".into()),
                tool_calls: None,
            },
        ];

        // Use production sanitization helper
        let sanitized = sanitize_messages_for_final_call(messages);

        // Verify sanitization
        assert_eq!(sanitized.len(), 3);
        assert_eq!(sanitized[0].role, "user");
        assert_eq!(sanitized[1].role, "assistant");
        assert_eq!(sanitized[1].tool_calls, None); // Stripped
        assert_eq!(sanitized[2].role, "assistant"); // Converted from 'tool' to assistant
        assert!(sanitized[2].content.starts_with("[UNTRUSTED TOOL OUTPUT]:")); // Labeled as untrusted
        assert_eq!(sanitized[2].tool_call_id, None); // Stripped
    }

    #[test]
    fn test_message_sanitization_preserves_non_tool_messages() {
        let messages = vec![
            Message {
                role: "system".into(),
                content: "You are a helpful assistant".into(),
                tool_call_id: None,
                tool_calls: None,
            },
            Message {
                role: "user".into(),
                content: "Hello".into(),
                tool_call_id: None,
                tool_calls: None,
            },
        ];

        // Use production sanitization helper
        let sanitized = sanitize_messages_for_final_call(messages);

        // Verify non-tool messages preserved
        assert_eq!(sanitized.len(), 2);
        assert_eq!(sanitized[0].role, "system");
        assert_eq!(sanitized[0].content, "You are a helpful assistant");
        assert_eq!(sanitized[1].role, "user");
        assert_eq!(sanitized[1].content, "Hello");
    }

    #[test]
    fn test_sanitize_messages_strips_tool_calls_from_all_messages() {
        let messages = vec![
            Message {
                role: "assistant".into(),
                content: "".into(),
                tool_call_id: None,
                tool_calls: Some(vec![crate::ai::ToolCall {
                    id: "call_1".into(),
                    name: "test".into(),
                    arguments: "{}".into(),
                }]),
            },
            Message {
                role: "tool".into(),
                content: "output".into(),
                tool_call_id: Some("call_1".into()),
                tool_calls: None,
            },
        ];

        let sanitized = sanitize_messages_for_final_call(messages);

        assert_eq!(sanitized.len(), 2);
        assert_eq!(sanitized[0].tool_calls, None); // Stripped from assistant
        assert_eq!(sanitized[1].tool_calls, None); // Already None, but verified
    }
}
