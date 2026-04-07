use async_trait::async_trait;

use crate::ai::{ChatResponse, Message, ProviderInfo, Tool};
use crate::state::ProviderConfig;

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn info(&self) -> ProviderInfo;
    async fn chat(
        &self,
        messages: Vec<Message>,
        config: &ProviderConfig,
        tools: Option<Vec<Tool>>,
    ) -> anyhow::Result<ChatResponse>;
}

pub fn create_provider(config: &ProviderConfig) -> Box<dyn Provider> {
    // Match on provider_type (the kind), falling back to name for legacy configs
    let kind = if config.provider_type.is_empty() {
        config.name.as_str()
    } else {
        config.provider_type.as_str()
    };
    match kind {
        "anthropic" => Box::new(crate::ai::anthropic::AnthropicProvider),
        "gemini" => Box::new(crate::ai::gemini::GeminiProvider),
        "mistral" => Box::new(crate::ai::mistral::MistralProvider),
        "ollama" => Box::new(crate::ai::ollama::OllamaProvider),
        _ => Box::new(crate::ai::openai::OpenAiProvider), // default: OpenAI-compatible
    }
}
