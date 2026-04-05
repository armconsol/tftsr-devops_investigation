use async_trait::async_trait;
use std::time::Duration;

use crate::ai::provider::Provider;
use crate::ai::{ChatResponse, Message, ProviderInfo, TokenUsage};
use crate::state::ProviderConfig;

pub struct AnthropicProvider;

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "Anthropic".to_string(),
            supports_streaming: true,
            models: vec![
                "claude-sonnet-4-20250514".to_string(),
                "claude-haiku-4-20250414".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
            ],
        }
    }

    async fn chat(
        &self,
        messages: Vec<Message>,
        config: &ProviderConfig,
    ) -> anyhow::Result<ChatResponse> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;
        let url = format!(
            "{}/v1/messages",
            config
                .api_url
                .trim_end_matches('/')
                .trim_end_matches("/v1/messages")
        );

        // Extract system message if the first message has role "system"
        let (system_text, chat_messages): (Option<String>, Vec<&Message>) = {
            let mut sys = None;
            let mut msgs = Vec::new();
            for msg in &messages {
                if msg.role == "system" && sys.is_none() {
                    sys = Some(msg.content.clone());
                } else {
                    msgs.push(msg);
                }
            }
            (sys, msgs)
        };

        // Build Anthropic messages format
        let api_messages: Vec<serde_json::Value> = chat_messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": if m.role == "assistant" { "assistant" } else { "user" },
                    "content": m.content,
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": config.model,
            "messages": api_messages,
            "max_tokens": 4096,
        });

        if let Some(sys) = &system_text {
            body["system"] = serde_json::Value::String(sys.clone());
        }

        let resp = client
            .post(&url)
            .header("x-api-key", &config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await?;
            anyhow::bail!("Anthropic API error {status}: {text}");
        }

        let json: serde_json::Value = resp.json().await?;

        // Parse content from response.content[0].text
        let content = json["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|block| block["text"].as_str())
            .ok_or_else(|| anyhow::anyhow!("No text content in Anthropic response"))?
            .to_string();

        let usage = json.get("usage").and_then(|u| {
            Some(TokenUsage {
                prompt_tokens: u["input_tokens"].as_u64()? as u32,
                completion_tokens: u["output_tokens"].as_u64()? as u32,
                total_tokens: (u["input_tokens"].as_u64()? + u["output_tokens"].as_u64()?) as u32,
            })
        });

        let model = json["model"].as_str().unwrap_or(&config.model).to_string();

        Ok(ChatResponse {
            content,
            model,
            usage,
        })
    }
}
