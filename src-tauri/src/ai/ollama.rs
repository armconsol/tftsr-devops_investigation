use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::ai::provider::Provider;
use crate::ai::{ChatResponse, Message, ProviderInfo, TokenUsage, ToolCall};
use crate::state::ProviderConfig;

// Track if we've already attempted auto-start this session
static AUTO_START_ATTEMPTED: AtomicBool = AtomicBool::new(false);

pub struct OllamaProvider;

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "Ollama (Local)".to_string(),
            supports_streaming: true,
            models: vec![
                "llama3.1".to_string(),
                "llama3".to_string(),
                "mistral".to_string(),
                "codellama".to_string(),
                "phi3".to_string(),
            ],
        }
    }

    async fn chat(
        &self,
        messages: Vec<Message>,
        config: &ProviderConfig,
        tools: Option<Vec<crate::ai::Tool>>,
    ) -> anyhow::Result<ChatResponse> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;
        let base_url = if config.api_url.is_empty() {
            "http://localhost:11434".to_string()
        } else {
            config.api_url.trim_end_matches('/').to_string()
        };

        // Auto-start Ollama if using localhost and we haven't tried yet this session
        // Only attempt once to avoid recurring latency on every chat() call
        if base_url == "http://localhost:11434"
            && !AUTO_START_ATTEMPTED.swap(true, Ordering::Relaxed)
        {
            match crate::ollama::installer::start_ollama_service().await {
                Ok(started) => {
                    if started {
                        tracing::info!("Ollama service auto-started successfully");
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to auto-start Ollama: {}", e);
                    // Continue anyway - maybe it's already running or will start soon
                }
            }
        }

        let url = format!("{base_url}/api/chat");

        // Ollama expects {model, messages: [{role, content, tool_calls?, tool_call_id?}], stream: false}
        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                let mut msg = serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                });

                // Include tool_calls if present (for assistant messages with tool requests)
                if let Some(ref tool_calls) = m.tool_calls {
                    msg["tool_calls"] = serde_json::json!(tool_calls);
                }

                // Include tool_call_id if present (for tool result messages)
                if let Some(ref tool_call_id) = m.tool_call_id {
                    msg["tool_call_id"] = serde_json::json!(tool_call_id);
                }

                msg
            })
            .collect();

        let mut body = serde_json::json!({
            "model": config.model,
            "messages": api_messages,
            "stream": false,
        });

        // Add tools if provided (Ollama function calling format)
        if let Some(tools_list) = tools {
            let formatted_tools: Vec<serde_json::Value> = tools_list
                .iter()
                .map(|tool| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": tool.name,
                            "description": tool.description,
                            "parameters": tool.parameters
                        }
                    })
                })
                .collect();
            body["tools"] = serde_json::Value::from(formatted_tools);
        }

        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await?;
            anyhow::bail!("Ollama API error {status}: {text}");
        }

        let json: serde_json::Value = resp.json().await?;

        // Parse response.message.content
        let content = json["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Parse tool calls from Ollama response
        // Ollama returns tool_calls in message.tool_calls array
        let tool_calls = if let Some(calls_array) = json["message"]["tool_calls"].as_array() {
            let mut parsed_calls = Vec::new();
            for (idx, call) in calls_array.iter().enumerate() {
                // Generate fallback ID if not provided
                let id = call["id"]
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("tool_call_{idx}"));

                let function = &call["function"];

                // Skip malformed tool calls (missing name) instead of failing entire response
                let name = match function["name"].as_str() {
                    Some(n) => n.to_string(),
                    None => {
                        tracing::warn!("Skipping tool call with missing name at index {idx}");
                        continue;
                    }
                };

                // Arguments can be either an object or a string
                let arguments = if let Some(args_obj) = function["arguments"].as_object() {
                    match serde_json::to_string(args_obj) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::warn!(
                                "Failed to serialize tool call arguments at index {}: {}",
                                idx,
                                e
                            );
                            continue;
                        }
                    }
                } else if let Some(args_str) = function["arguments"].as_str() {
                    args_str.to_string()
                } else {
                    "{}".to_string()
                };

                parsed_calls.push(ToolCall {
                    id,
                    name,
                    arguments,
                });
            }
            if !parsed_calls.is_empty() {
                Some(parsed_calls)
            } else {
                None
            }
        } else {
            None
        };

        // Ollama provides eval_count / prompt_eval_count
        let usage = {
            let prompt_tokens = json["prompt_eval_count"].as_u64().unwrap_or(0) as u32;
            let completion_tokens = json["eval_count"].as_u64().unwrap_or(0) as u32;
            if prompt_tokens > 0 || completion_tokens > 0 {
                Some(TokenUsage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                })
            } else {
                None
            }
        };

        Ok(ChatResponse {
            content,
            model: config.model.clone(),
            usage,
            user_message: None,
            tool_calls,
        })
    }
}
