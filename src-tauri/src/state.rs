use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    /// Optional: Maximum tokens for response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Optional: Temperature (0.0-2.0) - controls randomness
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Optional: Custom endpoint path (e.g., "" for no path, "/v1/chat" for custom path)
    /// If None, defaults to "/chat/completions" for OpenAI compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_endpoint_path: Option<String>,
    /// Optional: Custom auth header name (e.g., "x-custom-api-key")
    /// If None, defaults to "Authorization"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_auth_header: Option<String>,
    /// Optional: Custom auth value prefix (e.g., "" for no prefix, "Bearer " for OpenAI)
    /// If None, defaults to "Bearer "
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_auth_prefix: Option<String>,
    /// Optional: API format ("openai" or "custom_rest")
    /// If None, defaults to "openai"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_format: Option<String>,
    /// Optional: Session ID for stateful custom REST APIs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Optional: User ID for custom REST API cost tracking (CORE ID email)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Optional: When true, file uploads go to GenAI datastore instead of prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_datastore_upload: Option<bool>,
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
    /// Track open integration webview windows by service name -> window label
    /// These windows stay open for the user to browse and for fresh cookie extraction
    pub integration_webviews: Arc<Mutex<HashMap<String, String>>>,
}

/// Determine the application data directory.
/// Returns None if the directory cannot be determined.
pub fn get_app_data_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("TFTSR_DATA_DIR") {
        return Some(PathBuf::from(dir));
    }

    // Use platform-appropriate data directory
    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return Some(PathBuf::from(xdg).join("trcaa"));
        }
        if let Ok(home) = std::env::var("HOME") {
            return Some(
                PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("trcaa"),
            );
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Some(
                PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("trcaa"),
            );
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return Some(PathBuf::from(appdata).join("trcaa"));
        }
    }

    // Fallback
    Some(PathBuf::from("./trcaa-data"))
}
