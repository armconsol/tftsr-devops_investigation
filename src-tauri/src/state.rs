use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

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
    /// Optional: Whether this provider supports tool/function calling
    /// If None, defaults to false (provider can only be used for chat)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_tool_calling: Option<bool>,
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

/// Approval response for shell command execution
#[derive(Debug, Clone)]
pub struct ApprovalResponse {
    pub approved: bool,
    pub decision: String, // "deny", "allow_once", "allow_session"
}

pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub settings: Arc<Mutex<AppSettings>>,
    pub app_data_dir: PathBuf,
    /// Track open integration webview windows by service name -> window label
    pub integration_webviews: Arc<Mutex<HashMap<String, String>>>,
    /// Live MCP server connections: server_id -> connection
    pub mcp_connections:
        Arc<TokioMutex<HashMap<String, Arc<TokioMutex<crate::mcp::client::McpConnection>>>>>,
    /// Pending shell command approvals: approval_id -> response channel
    pub pending_approvals:
        Arc<TokioMutex<HashMap<String, tokio::sync::oneshot::Sender<ApprovalResponse>>>>,
    /// Kubernetes cluster clients: cluster_id -> client
    pub clusters: Arc<TokioMutex<HashMap<String, crate::kube::ClusterClient>>>,
    /// Port forwarding sessions: session_id -> session
    pub port_forwards: Arc<TokioMutex<HashMap<String, crate::kube::PortForwardSession>>>,
    /// Refresh registry for domain-based data fetching
    pub refresh_registry: Arc<TokioMutex<crate::kube::RefreshRegistry>>,
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
            return Some(PathBuf::from(xdg).join("tftsr"));
        }
        if let Ok(home) = std::env::var("HOME") {
            return Some(
                PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("tftsr"),
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
                    .join("tftsr"),
            );
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return Some(PathBuf::from(appdata).join("tftsr"));
        }
    }

    // Fallback
    Some(PathBuf::from("./tftsr-data"))
}
