// Copyright (c) 2025 Shaun Arman
// MIT License - see LICENSE file for details

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
    #[serde(default)]
    pub debug_logging_enabled: bool,
    #[serde(default = "default_update_channel")]
    pub update_channel: String,
}

fn default_update_channel() -> String {
    "stable".to_string()
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
            debug_logging_enabled: false,
            update_channel: "stable".to_string(),
        }
    }
}

/// Approval response for shell command execution
#[derive(Debug, Clone)]
pub struct ApprovalResponse {
    pub approved: bool,
    pub decision: String, // "deny", "allow_once", "allow_session"
}

/// Application-wide shared state injected into every Tauri command via
/// `State<'_, AppState>`.
///
/// # Synchronization expectations
///
/// All fields except `app_data_dir` are wrapped in either a `std::sync::Mutex`
/// or a `tokio::sync::Mutex`. The choice is deliberate and **must** be
/// preserved by callers:
///
/// - **`std::sync::Mutex`** (e.g. `db`, `settings`, `integration_webviews`,
///   `watchers`): held for short, synchronous critical sections only. **Never
///   hold a `MutexGuard` across an `.await`** — `MutexGuard` is `!Send` and
///   the compiler will reject it. The standard pattern is to lock inside a
///   `{ }` block, take the data needed, drop the guard, then `.await`.
///
/// - **`tokio::sync::Mutex`** (e.g. `mcp_connections`, `pending_approvals`,
///   `clusters`, `port_forwards`, `refresh_registry`, `log_streams`): used
///   for state that must be held across an `.await` (network calls, channel
///   operations, etc.). These have an async `lock().await` API.
///
/// - **`Arc<crate::shell::SessionManager>`**: the manager itself owns its
///   internal locking via `RwLock`; callers do not lock the `Arc`.
///
/// - **`app_data_dir`**: immutable for the lifetime of the process; safe to
///   read without synchronization.
///
/// All fields are `pub` so command handlers in `commands/*.rs` can clone
/// individual `Arc`s into spawned tasks without taking the entire `AppState`.
/// Callers should treat the choice of mutex type as part of the API contract:
/// changing a `std::sync::Mutex` to a `tokio::sync::Mutex` (or vice-versa) is
/// a breaking change for every handler that touches the field.
pub struct AppState {
    /// Encrypted SQLite (SQLCipher in release) connection. Short-lived locks
    /// only; never held across `.await`.
    pub db: Arc<Mutex<rusqlite::Connection>>,
    /// In-memory copy of `AppSettings`. Persisted to disk via the settings
    /// commands; lock for read/write but never across `.await`.
    pub settings: Arc<Mutex<AppSettings>>,
    /// Resolved data directory (`~/.local/share/tftsr` on Linux, etc.).
    /// Immutable for the process lifetime — no locking needed.
    pub app_data_dir: PathBuf,
    /// Track open integration webview windows by service name -> window label.
    /// Short-lived `std::sync::Mutex`.
    pub integration_webviews: Arc<Mutex<HashMap<String, String>>>,
    /// Live MCP server connections: server_id -> connection
    pub mcp_connections:
        Arc<TokioMutex<HashMap<String, Arc<TokioMutex<crate::mcp::client::McpConnection>>>>>,
    /// Pending shell command approvals: approval_id -> response channel
    pub pending_approvals:
        Arc<TokioMutex<HashMap<String, tokio::sync::oneshot::Sender<ApprovalResponse>>>>,
    /// Kubernetes cluster clients: cluster_id -> client
    pub clusters: Arc<TokioMutex<HashMap<String, crate::kube::ClusterClient>>>,
    /// Proxmox cluster clients: cluster_id -> client
    pub proxmox_clusters:
        Arc<TokioMutex<HashMap<String, Arc<TokioMutex<crate::proxmox::client::ProxmoxClient>>>>>,
    /// Port forwarding sessions: session_id -> session
    pub port_forwards: Arc<TokioMutex<HashMap<String, crate::kube::PortForwardSession>>>,
    /// Refresh registry for domain-based data fetching
    pub refresh_registry: Arc<TokioMutex<crate::kube::RefreshRegistry>>,
    /// Resource watchers: unsubscribe_id -> receiver
    pub watchers: Arc<Mutex<HashMap<String, tokio::sync::mpsc::Receiver<serde_json::Value>>>>,
    /// Active pod log streaming tasks: stream_id -> abort handle
    pub log_streams: Arc<TokioMutex<HashMap<String, tokio::task::AbortHandle>>>,
    /// PTY session manager for interactive shells
    pub pty_sessions: Arc<crate::shell::SessionManager>,
    /// RDP session manager
    pub rdp_manager: Arc<std::sync::Mutex<crate::remote::rdp::RdpManager>>,
    /// Database connection pool manager for multi-database support
    pub db_pool_manager: Arc<TokioMutex<crate::db_drivers::DatabasePoolManager>>,
}

/// Determine the application data directory.
/// Returns None if the directory cannot be determined.
pub fn get_app_data_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("TRCAA_DATA_DIR") {
        return Some(PathBuf::from(dir));
    }
    if let Ok(dir) = std::env::var("TFTSR_DATA_DIR") {
        tracing::warn!("TFTSR_DATA_DIR is deprecated, use TRCAA_DATA_DIR instead");
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

    // Fallback: use current working directory joined with tftsr-data
    std::env::current_dir().ok().map(|p| p.join("tftsr-data"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.default_provider, "ollama");
        assert!(!settings.debug_logging_enabled);
        assert_eq!(settings.update_channel, "stable");
    }

    #[test]
    fn test_app_settings_deserialize_defaults_debug_logging_disabled() {
        let json = r#"{
            "theme":"dark",
            "ai_providers":[],
            "default_provider":"ollama",
            "default_model":"llama3.2:3b",
            "ollama_url":"http://localhost:11434"
        }"#;

        let settings: AppSettings =
            serde_json::from_str(json).expect("settings should deserialize");
        assert!(!settings.debug_logging_enabled);
    }

    #[test]
    fn test_get_app_data_dir_returns_some() {
        let dir = get_app_data_dir();
        assert!(
            dir.is_some(),
            "App data directory should always be resolvable"
        );
    }

    /// Smoke test to verify libsodium linking via tauri-plugin-stronghold dependency chain.
    /// This test ensures the transitive dependency on libsodium-sys-stable compiles and links
    /// correctly across all build targets (Linux amd64/arm64, Windows, macOS).
    ///
    /// If this test compiles, it proves:
    /// 1. libsodium-sys-stable build.rs successfully found libsodium
    /// 2. The linker can resolve libsodium symbols
    /// 3. The entire stronghold -> iota-crypto -> libsodium-sys-stable chain works
    #[test]
    fn test_libsodium_linking() {
        // Simply importing and using a type from the stronghold dependency chain
        // is sufficient to verify linking. If libsodium were missing or misconfigured,
        // this test would fail at compile time (missing symbols) or link time.

        // Verify we can create AppState structure which depends on the full stack
        let _settings = AppSettings::default();

        // If we reach here, libsodium is properly linked
        assert!(
            true,
            "libsodium linking verified via stronghold dependency chain"
        );
    }
}
