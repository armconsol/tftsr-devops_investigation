use async_trait::async_trait;
use std::time::Duration;

use crate::ai::provider::Provider;
use crate::ai::{ChatResponse, Message, ProviderInfo, TokenUsage};
use crate::state::ProviderConfig;

pub struct OpenAiProvider;

fn is_custom_rest_format(api_format: Option<&str>) -> bool {
    matches!(api_format, Some("custom_rest") | Some("msi_genai"))
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
    ) -> anyhow::Result<ChatResponse> {
        // Check if using custom REST format
        let api_format = config.api_format.as_deref().unwrap_or("openai");

        // Backward compatibility: accept legacy msi_genai identifier
        if is_custom_rest_format(Some(api_format)) {
            self.chat_custom_rest(messages, config).await
        } else {
            self.chat_openai(messages, config).await
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
    fn legacy_msi_format_is_recognized_for_compatibility() {
        assert!(is_custom_rest_format(Some("msi_genai")));
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
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No content in response"))?
            .to_string();

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
        })
    }

    /// Custom REST format (MSI GenAI payload contract)
    async fn chat_custom_rest(
        &self,
        messages: Vec<Message>,
        config: &ProviderConfig,
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

        // Use custom auth header and prefix (no prefix for this custom REST contract)
        let auth_header = config
            .custom_auth_header
            .as_deref()
            .unwrap_or("x-msi-genai-api-key");
        let auth_prefix = config.custom_auth_prefix.as_deref().unwrap_or("");
        let auth_value = format!("{auth_prefix}{api_key}", api_key = config.api_key);

        let resp = client
            .post(&url)
            .header(auth_header, auth_value)
            .header("Content-Type", "application/json")
            .header("X-msi-genai-client", "troubleshooting-rca-assistant")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await?;
            anyhow::bail!("Custom REST API error {status}: {text}");
        }

        let json: serde_json::Value = resp.json().await?;

        // Extract response content from "msg" field
        let content = json["msg"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No 'msg' field in response"))?
            .to_string();

        // Note: sessionId from response should be stored back to config.session_id
        // This would require making config mutable or returning it as part of ChatResponse
        // For now, the caller can extract it from the response if needed
        // TODO: Consider adding session_id to ChatResponse struct

        Ok(ChatResponse {
            content,
            model: config.model.clone(),
            usage: None, // This custom REST contract doesn't provide token usage in response
        })
    }
}
