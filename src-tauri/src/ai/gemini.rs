use async_trait::async_trait;
use std::time::Duration;

use crate::ai::provider::Provider;
use crate::ai::{ChatResponse, Message, ProviderInfo, TokenUsage};
use crate::state::ProviderConfig;

pub struct GeminiProvider;

#[async_trait]
impl Provider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "Google Gemini".to_string(),
            supports_streaming: true,
            models: vec![
                "gemini-2.0-flash".to_string(),
                "gemini-2.0-pro".to_string(),
                "gemini-1.5-pro".to_string(),
                "gemini-1.5-flash".to_string(),
            ],
        }
    }

    async fn chat(
        &self,
        messages: Vec<Message>,
        config: &ProviderConfig,
        _tools: Option<Vec<crate::ai::Tool>>,
    ) -> anyhow::Result<ChatResponse> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            config.model
        );

        // Map OpenAI-style messages to Gemini format
        // Gemini uses "user" and "model" roles (not "assistant")
        // System messages are passed as a systemInstruction
        let mut system_text: Option<String> = None;
        let mut contents: Vec<serde_json::Value> = Vec::new();

        for msg in &messages {
            match msg.role.as_str() {
                "system" => {
                    system_text = Some(msg.content.clone());
                }
                "assistant" => {
                    contents.push(serde_json::json!({
                        "role": "model",
                        "parts": [{"text": msg.content}],
                    }));
                }
                _ => {
                    // "user" and anything else maps to "user"
                    contents.push(serde_json::json!({
                        "role": "user",
                        "parts": [{"text": msg.content}],
                    }));
                }
            }
        }

        let mut body = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": 4096,
            },
        });

        if let Some(sys) = &system_text {
            body["systemInstruction"] = serde_json::json!({
                "parts": [{"text": sys}],
            });
        }

        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-goog-api-key", &config.api_key)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await?;
            anyhow::bail!("Gemini API error {status}: {text}");
        }

        let json: serde_json::Value = resp.json().await?;

        // Parse candidates[0].content.parts[0].text
        let content = json["candidates"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|candidate| candidate["content"]["parts"].as_array())
            .and_then(|parts| parts.first())
            .and_then(|part| part["text"].as_str())
            .ok_or_else(|| anyhow::anyhow!("No text content in Gemini response"))?
            .to_string();

        // Parse token usage from usageMetadata
        let usage = json.get("usageMetadata").and_then(|u| {
            Some(TokenUsage {
                prompt_tokens: u["promptTokenCount"].as_u64()? as u32,
                completion_tokens: u["candidatesTokenCount"].as_u64()? as u32,
                total_tokens: u["totalTokenCount"].as_u64()? as u32,
            })
        });

        Ok(ChatResponse {
            content,
            model: config.model.clone(),
            usage,
            user_message: None,
            tool_calls: None,
        })
    }
}
