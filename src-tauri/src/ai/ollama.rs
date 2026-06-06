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
                "llama3.2:3b".to_string(),
                "phi3.5:3.8b".to_string(),
                "llama3.1:8b".to_string(),
                "qwen2.5:14b".to_string(),
                "gemma2:9b".to_string(),
            ],
        }
    }

    async fn chat(
        &self,
        messages: Vec<Message>,
        config: &ProviderConfig,
        tools: Option<Vec<crate::ai::Tool>>,
    ) -> anyhow::Result<ChatResponse> {
        // Longer timeout for tool calling - models need time to generate structured output
        let timeout_secs = if tools.is_some() { 180 } else { 60 };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .connect_timeout(Duration::from_secs(10))
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
            // Check if already running before attempting start
            let pre_status = crate::ollama::installer::check_ollama().await;
            let already_running = pre_status.map(|s| s.running).unwrap_or(false);

            if !already_running {
                match crate::ollama::installer::start_ollama_service().await {
                    Ok(true) => {
                        tracing::info!("Ollama service auto-started successfully");
                        // Give it a moment to fully initialize
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                    Ok(false) => {
                        tracing::debug!("Ollama not started (not installed or already running)");
                    }
                    Err(e) => {
                        tracing::warn!("Failed to auto-start Ollama: {}", e);
                        // Continue anyway - maybe it's already running or will start soon
                    }
                }
            } else {
                tracing::debug!("Ollama already running, skipping auto-start");
            }
        }

        // Quick health check before attempting chat (short timeout for fast failure)
        let health_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()?;
        let health_check_result = health_client
            .get(format!("{base_url}/api/tags"))
            .send()
            .await;

        match health_check_result {
            Ok(resp) if resp.status().is_success() => {
                tracing::debug!("Ollama health check passed");
            }
            Ok(resp) => {
                let status = resp.status();
                tracing::warn!("Ollama health check returned status {status}");
                anyhow::bail!(
                    "Ollama is not ready (status {status}). Please ensure Ollama is running."
                );
            }
            Err(e) => {
                tracing::error!("Cannot connect to Ollama at {base_url}: {e}");
                anyhow::bail!("Cannot connect to Ollama at {base_url}. Please ensure Ollama is running and accessible.");
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

        // Retry logic for transient connection issues
        let max_retries = 2;
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                tracing::warn!(
                    "Ollama request failed, retrying (attempt {}/{})...",
                    attempt + 1,
                    max_retries + 1
                );
                tokio::time::sleep(Duration::from_secs(2)).await;
            }

            let resp_result = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await;

            let resp = match resp_result {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(format!("Connection error: {e}"));
                    if attempt < max_retries {
                        continue; // Retry
                    } else {
                        anyhow::bail!(
                            "Failed to connect to Ollama after {} attempts. Last error: {e}",
                            max_retries + 1
                        );
                    }
                }
            };

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await?;
                last_error = Some(format!("API error {status}: {text}"));
                if attempt < max_retries && status.is_server_error() {
                    continue; // Retry on 5xx errors
                } else {
                    anyhow::bail!("Ollama API error {status}: {text}");
                }
            }

            // Success - parse response and return
            let json: serde_json::Value = match resp.json().await {
                Ok(j) => j,
                Err(e) => {
                    last_error = Some(format!("JSON parse error: {e}"));
                    if attempt < max_retries {
                        continue; // Retry
                    } else {
                        anyhow::bail!("Failed to parse Ollama response: {e}");
                    }
                }
            };

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

            return Ok(ChatResponse {
                content,
                model: config.model.clone(),
                usage,
                user_message: None,
                tool_calls,
            });
        }

        // If we get here, all retries failed
        anyhow::bail!(
            "Failed to get response from Ollama after {} attempts. Last error: {:?}",
            max_retries + 1,
            last_error
        )
    }
}
