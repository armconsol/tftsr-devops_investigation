use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    #[serde(default)]
    pub provider_type: String,
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    /// Optional: Custom endpoint path (e.g., "" for no path, "/v1/chat" for custom path)
    /// If None, defaults to "/chat/completions" for OpenAI compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_endpoint_path: Option<String>,
    /// Optional: Custom auth header name (e.g., "x-msi-genai-api-key")
    /// If None, defaults to "Authorization"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_auth_header: Option<String>,
    /// Optional: Custom auth value prefix (e.g., "" for no prefix, "Bearer " for OpenAI)
    /// If None, defaults to "Bearer "
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_auth_prefix: Option<String>,
    /// Optional: API format ("openai" or "msi_genai")
    /// If None, defaults to "openai"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_format: Option<String>,
    /// Optional: Session ID for stateful APIs like MSI GenAI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub ai_providers: Vec<ProviderConfig>,
    pub active_provider: Option<String>,
    pub default_provider: String,
    pub default_model: String,
    pub ollama_url: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            theme: "dark".to_string(),
            ai_providers: vec![],
            active_provider: None,
            default_provider: "ollama".to_string(),
            default_model: "llama3.2:3b".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
        }
    }
}

pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub settings: Arc<Mutex<AppSettings>>,
    pub app_data_dir: PathBuf,
}
