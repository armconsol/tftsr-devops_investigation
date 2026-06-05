use async_trait::async_trait;
use std::time::Duration;

use crate::ai::provider::Provider;
use crate::ai::{ChatResponse, Message, ProviderInfo, TokenUsage};
use crate::state::ProviderConfig;

pub struct OpenAiProvider;

fn is_msi_genai_format(api_format: Option<&str>) -> bool {
    matches!(api_format, Some("msi-genai") | Some("custom_rest")) // custom_rest for backward compatibility
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
        // Check if using GenAI format (or legacy custom_rest)
        let api_format = config.api_format.as_deref().unwrap_or("openai");

        if is_msi_genai_format(Some(api_format)) {
            self.chat_msi_genai(messages, config, tools).await
        } else {
            self.chat_openai(messages, config, tools).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_msi_genai_format, OpenAiProvider};

    #[test]
    fn msi_genai_format_is_recognized() {
        assert!(is_msi_genai_format(Some("msi-genai")));
    }

    #[test]
    fn custom_rest_format_backward_compatible() {
        // Keep backward compatibility with old format name
        assert!(is_msi_genai_format(Some("custom_rest")));
    }

    #[test]
    fn openai_format_is_not_msi_genai() {
        assert!(!is_msi_genai_format(Some("openai")));
        assert!(!is_msi_genai_format(None));
    }

    #[test]
    fn parse_msigenai_chatgpt_tool_calls_from_json_text() {
        // GenAI ChatGPT format: returns tool calls as JSON object in msg
        let content = r#"{"tool_calls":[{"id":"call_1","type":"function","function":{"name":"execute_shell_command","arguments":{"command":"kubectl get namespaces"}}}]}"#;

        let result = OpenAiProvider::parse_tool_calls_from_text(content);
        assert!(result.is_some());

        let calls = result.unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_1");
        assert_eq!(calls[0].name, "execute_shell_command");
        assert!(calls[0].arguments.contains("kubectl get namespaces"));
    }

    #[test]
    fn parse_msigenai_claude_tool_calls_from_xml_wrapper() {
        // GenAI Claude format: XML wrapper around JSON array
        let content = r#"<tool_calls>
[{"id":"call_1","type":"function","function":{"name":"execute_shell_command","arguments":{"command":"kubectl get pods"}}}]
</tool_calls>"#;

        let result = OpenAiProvider::parse_tool_calls_from_text(content);
        assert!(result.is_some());

        let calls = result.unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_1");
        assert_eq!(calls[0].name, "execute_shell_command");
        assert!(calls[0].arguments.contains("kubectl get pods"));
    }

    #[test]
    fn parse_multiple_tool_calls_from_text() {
        let content = r#"{"tool_calls":[
            {"id":"call_1","function":{"name":"kubectl_get","arguments":{"resource":"pods"}}},
            {"id":"call_2","function":{"name":"kubectl_describe","arguments":{"resource":"svc/nginx"}}}
        ]}"#;

        let result = OpenAiProvider::parse_tool_calls_from_text(content);
        assert!(result.is_some());

        let calls = result.unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "kubectl_get");
        assert_eq!(calls[1].name, "kubectl_describe");
    }

    #[test]
    fn parse_tool_calls_returns_none_for_normal_text() {
        let content = "Hello, I found 5 pods running in the cluster.";
        let result = OpenAiProvider::parse_tool_calls_from_text(content);
        assert!(result.is_none());
    }

    #[test]
    fn parse_tool_calls_handles_arguments_as_string() {
        // Some providers return arguments as string, not object
        let content = r#"{"tool_calls":[{"id":"call_1","function":{"name":"test","arguments":"{\"key\":\"value\"}"}}]}"#;

        let result = OpenAiProvider::parse_tool_calls_from_text(content);
        assert!(result.is_some());

        let calls = result.unwrap();
        assert_eq!(calls[0].arguments, r#"{"key":"value"}"#);
    }

    #[test]
    fn parse_tool_calls_generates_fallback_id_when_missing() {
        // Some providers may omit id field - generate fallback to prevent silent drop
        let content = r#"{"tool_calls":[
            {"function":{"name":"kubectl_get","arguments":{"resource":"pods"}}},
            {"id":"call_2","function":{"name":"kubectl_describe","arguments":{"resource":"svc"}}}
        ]}"#;

        let result = OpenAiProvider::parse_tool_calls_from_text(content);
        assert!(result.is_some());

        let calls = result.unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].id, "tool_call_0"); // Fallback generated
        assert_eq!(calls[0].name, "kubectl_get");
        assert_eq!(calls[1].id, "call_2"); // Original preserved
        assert_eq!(calls[1].name, "kubectl_describe");
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

        tracing::debug!(
            url = %url,
            model = %config.model,
            max_tokens = ?config.max_tokens,
            temperature = ?config.temperature,
            "OpenAI API request"
        );

        let model = config.model.trim_end_matches('.');
        let mut body = serde_json::json!({
            "model": model,
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
            .await;

        let resp = match resp {
            Ok(response) => response,
            Err(e) => {
                tracing::error!(url = %url, error = %e, "OpenAI API request failed");
                anyhow::bail!("OpenAI API request failed: {e}");
            }
        };

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp
                .text()
                .await
                .unwrap_or_else(|_| "unable to read response body".to_string());
            tracing::error!(url = %url, status = %status, response = %text, "OpenAI API error response");
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
            user_message: None,
            tool_calls,
        })
    }

    /// GenAI format (non-OpenAI payload contract)
    ///
    /// GenAI uses a custom API format with 'prompt' field instead of 'messages',
    /// and has a known bug where tool calls are returned as JSON text in the 'msg'
    /// field instead of structured 'tool_calls' array. This implementation includes
    /// workaround parsing to extract tool calls from text.
    async fn chat_msi_genai(
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

            tracing::info!("GenAI: Sending {} tools in request", tool_count);
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
            anyhow::bail!("GenAI API error {status}: {text}");
        }

        let json: serde_json::Value = resp.json().await?;

        tracing::debug!(
            "GenAI response: {}",
            serde_json::to_string_pretty(&json).unwrap_or_else(|_| "invalid JSON".to_string())
        );

        // Extract response content from "msg" field
        let content = json["msg"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No 'msg' field in response"))?
            .to_string();

        // Parse tool_calls if present (check multiple possible field names)
        let mut tool_calls = json
            .get("tool_calls")
            .or_else(|| json.get("toolCalls"))
            .or_else(|| json.get("function_calls"))
            .and_then(|tc| {
                if let Some(arr) = tc.as_array() {
                    let calls: Vec<crate::ai::ToolCall> = arr
                        .iter()
                        .enumerate()
                        .filter_map(|(index, call)| {
                            // Try OpenAI format first
                            if let (Some(id), Some(name)) = (
                                call.get("id").and_then(|v| v.as_str()),
                                call.get("function")
                                    .and_then(|f| f.get("name"))
                                    .and_then(|n| n.as_str())
                                    .or_else(|| call.get("name").and_then(|n| n.as_str())),
                            ) {
                                // Accept arguments as either string or object (GenAI returns both)
                                let arguments = call
                                    .get("function")
                                    .and_then(|f| f.get("arguments"))
                                    .or_else(|| call.get("arguments"))
                                    .and_then(|args| {
                                        if let Some(s) = args.as_str() {
                                            Some(s.to_string())
                                        } else {
                                            // Serialize object to JSON string
                                            serde_json::to_string(args).ok()
                                        }
                                    });

                                if let Some(args) = arguments {
                                    tracing::info!(
                                        "GenAI: Parsed tool call: {} ({})",
                                        name,
                                        id
                                    );
                                    return Some(crate::ai::ToolCall {
                                        id: id.to_string(),
                                        name: name.to_string(),
                                        arguments: args,
                                    });
                                }
                            }

                            // Try simpler format
                            if let Some(name) = call.get("name").and_then(|n| n.as_str()) {
                                // Accept arguments as either string or object
                                let arguments = call.get("arguments").and_then(|args| {
                                    if let Some(s) = args.as_str() {
                                        Some(s.to_string())
                                    } else {
                                        // Serialize object to JSON string
                                        serde_json::to_string(args).ok()
                                    }
                                });

                                if let Some(args) = arguments {
                                    // Generate unique ID if missing (avoids duplicates)
                                    let id = call
                                        .get("id")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| format!("tool_call_{index}"));
                                    tracing::info!(
                                        "GenAI: Parsed tool call (simple format): {} ({})",
                                        name,
                                        id
                                    );
                                    return Some(crate::ai::ToolCall {
                                        id,
                                        name: name.to_string(),
                                        arguments: args,
                                    });
                                }
                            }

                            tracing::warn!("GenAI: Failed to parse tool call: {:?}", call);
                            None
                        })
                        .collect();
                    if calls.is_empty() {
                        None
                    } else {
                        tracing::info!("GenAI: Found {} tool calls", calls.len());
                        Some(calls)
                    }
                } else {
                    None
                }
            });

        // WORKAROUND: GenAI gateway bug - tool calls returned as JSON text in 'msg' field
        // Expected: {"tool_calls": [...]}
        // Actual: {"msg": '{"tool_calls":[...]}'}  or  {"msg": '<tool_calls>[...]</tool_calls>'}
        if tool_calls.is_none() {
            // Try parsing tool calls from msg content (GenAI workaround)
            if let Some(parsed_calls) = Self::parse_tool_calls_from_text(&content) {
                tracing::warn!(
                    "GenAI: GenAI workaround - parsed {} tool calls from msg text (gateway should return structured tool_calls field)",
                    parsed_calls.len()
                );
                tool_calls = Some(parsed_calls);
            }
        }

        // Note: sessionId from response should be stored back to config.session_id
        // This would require making config mutable or returning it as part of ChatResponse
        // For now, the caller can extract it from the response if needed
        // TODO: Consider adding session_id to ChatResponse struct

        Ok(ChatResponse {
            content,
            model: config.model.clone(),
            usage: None, // This custom REST contract doesn't provide token usage in response
            user_message: None,
            tool_calls,
        })
    }

    /// Parse tool calls from text content (GenAI gateway workaround)
    ///
    /// GenAI returns tool calls as JSON text in the 'msg' field instead of structured data:
    /// - ChatGPT models: `{"tool_calls":[...]}`
    /// - Claude models: `<tool_calls>[...]</tool_calls>`
    fn parse_tool_calls_from_text(content: &str) -> Option<Vec<crate::ai::ToolCall>> {
        // Try parsing as direct JSON object
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(calls) = parsed.get("tool_calls").and_then(|v| v.as_array()) {
                return Self::extract_tool_calls_from_array(calls);
            }
        }

        // Try finding JSON in text (handle Claude XML wrapper: <tool_calls>[...]</tool_calls>)
        if let Some(start) = content.find("<tool_calls>") {
            if let Some(end) = content.find("</tool_calls>") {
                let json_str = &content[start + 12..end].trim();
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(calls) = parsed.as_array() {
                        return Self::extract_tool_calls_from_array(calls);
                    }
                }
            }
        }

        // Try finding raw JSON array in text
        if let Some(start) = content.find("[{") {
            if let Some(end) = content.rfind("}]") {
                let json_str = &content[start..=end + 1];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(calls) = parsed.as_array() {
                        return Self::extract_tool_calls_from_array(calls);
                    }
                }
            }
        }

        None
    }

    /// Extract ToolCall structs from JSON array
    fn extract_tool_calls_from_array(
        calls: &[serde_json::Value],
    ) -> Option<Vec<crate::ai::ToolCall>> {
        let parsed: Vec<crate::ai::ToolCall> = calls
            .iter()
            .enumerate()
            .filter_map(|(index, call)| {
                // Generate fallback ID if missing (consistent with earlier parsing logic in this file)
                let id = call
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("tool_call_{index}"));

                // Try nested function.name format (OpenAI style)
                let name = call
                    .get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                    .or_else(|| call.get("name").and_then(|n| n.as_str()))?
                    .to_string();

                // Arguments can be string or object
                let arguments = call
                    .get("function")
                    .and_then(|f| f.get("arguments"))
                    .or_else(|| call.get("arguments"))
                    .and_then(|args| {
                        if let Some(s) = args.as_str() {
                            Some(s.to_string())
                        } else {
                            serde_json::to_string(args).ok()
                        }
                    })?;

                Some(crate::ai::ToolCall {
                    id,
                    name,
                    arguments,
                })
            })
            .collect();

        if parsed.is_empty() {
            None
        } else {
            Some(parsed)
        }
    }
}
