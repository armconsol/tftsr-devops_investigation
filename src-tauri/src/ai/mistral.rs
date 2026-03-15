use async_trait::async_trait;

use crate::ai::provider::Provider;
use crate::ai::{ChatResponse, Message, ProviderInfo, TokenUsage};
use crate::state::ProviderConfig;

pub struct MistralProvider;

#[async_trait]
impl Provider for MistralProvider {
    fn name(&self) -> &str {
        "mistral"
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "Mistral AI".to_string(),
            supports_streaming: true,
            models: vec![
                "mistral-large-latest".to_string(),
                "mistral-medium-latest".to_string(),
                "mistral-small-latest".to_string(),
                "open-mistral-nemo".to_string(),
            ],
        }
    }

    async fn chat(
        &self,
        messages: Vec<Message>,
        config: &ProviderConfig,
    ) -> anyhow::Result<ChatResponse> {
        // Mistral uses OpenAI-compatible format
        let client = reqwest::Client::new();
        let base_url = if config.api_url.is_empty() {
            "https://api.mistral.ai/v1".to_string()
        } else {
            config.api_url.trim_end_matches('/').to_string()
        };
        let url = format!("{}/chat/completions", base_url);

        let body = serde_json::json!({
            "model": config.model,
            "messages": messages,
            "max_tokens": 4096,
        });

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await?;
            anyhow::bail!("Mistral API error {}: {}", status, text);
        }

        let json: serde_json::Value = resp.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No content in Mistral response"))?
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
}
