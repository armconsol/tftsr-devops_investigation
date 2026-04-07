use async_trait::async_trait;
use std::time::Duration;

use crate::ai::provider::Provider;
use crate::ai::{ChatResponse, Message, ProviderInfo, TokenUsage};
use crate::state::ProviderConfig;

pub struct OpenAiProvider;

fn is_custom_rest_format(api_format: Option<&str>) -> bool {
    matches!(api_format, Some("custom_rest"))
}

#[async_trait]
impl Provider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "OpenAI Compatible".to_string(),
            supports_streaming: true,
            models: vec![
                "gpt-4o".to_string(),
                "gpt-4o-mini".to_string(),
                "gpt-4-turbo".to_string(),
            ],
        }
    }

    async fn chat(
        &self,
        messages: Vec<Message>,
        config: &ProviderConfig,
        tools: Option<Vec<crate::ai::Tool>>,
    ) -> anyhow::Result<ChatResponse> {
        // Check if using custom REST format
        let api_format = config.api_format.as_deref().unwrap_or("openai");

        if is_custom_rest_format(Some(api_format)) {
            self.chat_custom_rest(messages, config, tools).await
        } else {
            self.chat_openai(messages, config, tools).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_custom_rest_format;

    #[test]
    fn custom_rest_format_is_recognized() {
        assert!(is_custom_rest_format(Some("custom_rest")));
    }

    #[test]
    fn openai_format_is_not_custom_rest() {
        assert!(!is_custom_rest_format(Some("openai")));
        assert!(!is_custom_rest_format(None));
    }
}

impl OpenAiProvider {
    /// OpenAI-compatible API format (default)
    async fn chat_openai(
        &self,
        messages: Vec<Message>,
        config: &ProviderConfig,
        tools: Option<Vec<crate::ai::Tool>>,
    ) -> anyhow::Result<ChatResponse> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        // Use custom endpoint path if provided, otherwise default to /chat/completions
        let endpoint_path = config
            .custom_endpoint_path
            .as_deref()
            .unwrap_or("/chat/completions");
        let api_url = config.api_url.trim_end_matches('/');
        let url = format!("{api_url}{endpoint_path}");

        let mut body = serde_json::json!({
            "model": config.model,
            "messages": messages,
        });

        // Add max_tokens if provided, otherwise use default 4096
        body["max_tokens"] = serde_json::Value::from(config.max_tokens.unwrap_or(4096));

        // Add temperature if provided
        if let Some(temp) = config.temperature {
            body["temperature"] = serde_json::Value::from(temp);
        }

        // Add tools if provided (OpenAI function calling format)
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
            body["tool_choice"] = serde_json::Value::from("auto");
        }

        // Use custom auth header and prefix if provided
        let auth_header = config
            .custom_auth_header
            .as_deref()
            .unwrap_or("Authorization");
        let auth_prefix = config.custom_auth_prefix.as_deref().unwrap_or("Bearer ");
        let auth_value = format!("{auth_prefix}{api_key}", api_key = config.api_key);

        let resp = client
            .post(&url)
            .header(auth_header, auth_value)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await?;
            anyhow::bail!("OpenAI API error {status}: {text}");
        }

        let json: serde_json::Value = resp.json().await?;
        let message = &json["choices"][0]["message"];

        let content = message["content"].as_str().unwrap_or("").to_string();

        // Parse tool_calls if present
        let tool_calls = message.get("tool_calls").and_then(|tc| {
            if let Some(arr) = tc.as_array() {
                let calls: Vec<crate::ai::ToolCall> = arr
                    .iter()
                    .filter_map(|call| {
                        Some(crate::ai::ToolCall {
                            id: call["id"].as_str()?.to_string(),
                            name: call["function"]["name"].as_str()?.to_string(),
                            arguments: call["function"]["arguments"].as_str()?.to_string(),
                        })
                    })
                    .collect();
                if calls.is_empty() {
                    None
                } else {
                    Some(calls)
                }
            } else {
                None
            }
        });

        let usage = json.get("usage").and_then(|u| {
            Some(TokenUsage {
                prompt_tokens: u["prompt_tokens"].as_u64()? as u32,
                completion_tokens: u["completion_tokens"].as_u64()? as u32,
                total_tokens: u["total_tokens"].as_u64()? as u32,
            })
        });

        Ok(ChatResponse {
            content,
            model: config.model.clone(),
            usage,
            tool_calls,
        })
    }

    /// Custom REST format (non-OpenAI payload contract)
    async fn chat_custom_rest(
        &self,
        messages: Vec<Message>,
        config: &ProviderConfig,
        tools: Option<Vec<crate::ai::Tool>>,
    ) -> anyhow::Result<ChatResponse> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        // Use custom endpoint path, default to empty (API URL already includes /api/v2/chat)
        let endpoint_path = config.custom_endpoint_path.as_deref().unwrap_or("");
        let api_url = config.api_url.trim_end_matches('/');
        let url = format!("{api_url}{endpoint_path}");

        // Extract system message if present
        let system_message = messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone());

        // Get last user message as prompt
        let prompt = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No user message found"))?;

        // Build request body
        let mut body = serde_json::json!({
            "model": config.model,
            "prompt": prompt,
        });

        // Add userId if provided (CORE ID email)
        if let Some(user_id) = &config.user_id {
            body["userId"] = serde_json::Value::String(user_id.clone());
        }

        // Add optional system message
        if let Some(system) = system_message {
            body["system"] = serde_json::Value::String(system);
        }

        // Add session ID if available (for conversation continuity)
        if let Some(session_id) = &config.session_id {
            body["sessionId"] = serde_json::Value::String(session_id.clone());
        }

        // Add modelConfig with temperature and max_tokens if provided
        let mut model_config = serde_json::json!({});
        if let Some(temp) = config.temperature {
            model_config["temperature"] = serde_json::Value::from(temp);
        }
        if let Some(max_tokens) = config.max_tokens {
            model_config["max_tokens"] = serde_json::Value::from(max_tokens);
        }
        if !model_config.is_null() && model_config.as_object().is_some_and(|obj| !obj.is_empty()) {
            body["modelConfig"] = model_config;
        }

        // Add tools if provided (OpenAI-style format, most common standard)
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
            let tool_count = formatted_tools.len();
            body["tools"] = serde_json::Value::from(formatted_tools);
            body["tool_choice"] = serde_json::Value::from("auto");

            tracing::info!("Custom REST: Sending {} tools in request", tool_count);
        }

        // Use custom auth header and prefix (no default prefix for custom REST)
        let auth_header = config
            .custom_auth_header
            .as_deref()
            .unwrap_or("Authorization");
        let auth_prefix = config.custom_auth_prefix.as_deref().unwrap_or("");
        let auth_value = format!("{auth_prefix}{api_key}", api_key = config.api_key);

        let resp = client
            .post(&url)
            .header(auth_header, auth_value)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await?;
            anyhow::bail!("Custom REST API error {status}: {text}");
        }

        let json: serde_json::Value = resp.json().await?;

        tracing::debug!(
            "Custom REST response: {}",
            serde_json::to_string_pretty(&json).unwrap_or_else(|_| "invalid JSON".to_string())
        );

        // Extract response content from "msg" field
        let content = json["msg"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No 'msg' field in response"))?
            .to_string();

        // Parse tool_calls if present (check multiple possible field names)
        let tool_calls = json
            .get("tool_calls")
            .or_else(|| json.get("toolCalls"))
            .or_else(|| json.get("function_calls"))
            .and_then(|tc| {
                if let Some(arr) = tc.as_array() {
                    let calls: Vec<crate::ai::ToolCall> = arr
                        .iter()
                        .filter_map(|call| {
                            // Try OpenAI format first
                            if let (Some(id), Some(name), Some(args)) = (
                                call.get("id").and_then(|v| v.as_str()),
                                call.get("function")
                                    .and_then(|f| f.get("name"))
                                    .and_then(|n| n.as_str())
                                    .or_else(|| call.get("name").and_then(|n| n.as_str())),
                                call.get("function")
                                    .and_then(|f| f.get("arguments"))
                                    .and_then(|a| a.as_str())
                                    .or_else(|| call.get("arguments").and_then(|a| a.as_str())),
                            ) {
                                tracing::info!("Custom REST: Parsed tool call: {} ({})", name, id);
                                return Some(crate::ai::ToolCall {
                                    id: id.to_string(),
                                    name: name.to_string(),
                                    arguments: args.to_string(),
                                });
                            }

                            // Try simpler format
                            if let (Some(name), Some(args)) = (
                                call.get("name").and_then(|n| n.as_str()),
                                call.get("arguments").and_then(|a| a.as_str()),
                            ) {
                                let id = call
                                    .get("id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("tool_call_0")
                                    .to_string();
                                tracing::info!(
                                    "Custom REST: Parsed tool call (simple format): {} ({})",
                                    name,
                                    id
                                );
                                return Some(crate::ai::ToolCall {
                                    id,
                                    name: name.to_string(),
                                    arguments: args.to_string(),
                                });
                            }

                            tracing::warn!("Custom REST: Failed to parse tool call: {:?}", call);
                            None
                        })
                        .collect();
                    if calls.is_empty() {
                        None
                    } else {
                        tracing::info!("Custom REST: Found {} tool calls", calls.len());
                        Some(calls)
                    }
                } else {
                    None
                }
            });

        // Note: sessionId from response should be stored back to config.session_id
        // This would require making config mutable or returning it as part of ChatResponse
        // For now, the caller can extract it from the response if needed
        // TODO: Consider adding session_id to ChatResponse struct

        Ok(ChatResponse {
            content,
            model: config.model.clone(),
            usage: None, // This custom REST contract doesn't provide token usage in response
            tool_calls,
        })
    }
}
