use async_trait::async_trait;

use crate::ai::provider::Provider;
use crate::ai::{ChatResponse, Message, ProviderInfo, TokenUsage};
use crate::state::ProviderConfig;

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
    ) -> anyhow::Result<ChatResponse> {
        let client = reqwest::Client::new();
        let base_url = if config.api_url.is_empty() {
            "http://localhost:11434".to_string()
        } else {
            config.api_url.trim_end_matches('/').to_string()
        };
        let url = format!("{base_url}/api/chat");

        // Ollama expects {model, messages: [{role, content}], stream: false}
        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": config.model,
            "messages": api_messages,
            "stream": false,
        });

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
            .ok_or_else(|| anyhow::anyhow!("No content in Ollama response"))?
            .to_string();

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
        })
    }
}
