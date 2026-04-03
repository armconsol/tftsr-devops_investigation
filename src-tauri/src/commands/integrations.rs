use crate::integrations::{ConnectionResult, PublishResult, TicketResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};
use tokio::sync::oneshot;

// Global OAuth state storage (verifier + service per state key)
lazy_static::lazy_static! {
    static ref OAUTH_STATE: Arc<Mutex<HashMap<String, (String, String)>>> = Arc::new(Mutex::new(HashMap::new()));
}

// Global callback server shutdown channel
lazy_static::lazy_static! {
    static ref CALLBACK_SERVER_SHUTDOWN: Arc<Mutex<Option<oneshot::Sender<()>>>> = Arc::new(Mutex::new(None));
}

#[tauri::command]
pub async fn test_confluence_connection(
    _base_url: String,
    _credentials: serde_json::Value,
) -> Result<ConnectionResult, String> {
    Err("Integrations available in v0.2. Please update to the latest version.".to_string())
}

#[tauri::command]
pub async fn publish_to_confluence(
    _doc_id: String,
    _space_key: String,
    _parent_page_id: Option<String>,
) -> Result<PublishResult, String> {
    Err("Integrations available in v0.2. Please update to the latest version.".to_string())
}

#[tauri::command]
pub async fn test_servicenow_connection(
    _instance_url: String,
    _credentials: serde_json::Value,
) -> Result<ConnectionResult, String> {
    Err("Integrations available in v0.2. Please update to the latest version.".to_string())
}

#[tauri::command]
pub async fn create_servicenow_incident(
    _issue_id: String,
    _config: serde_json::Value,
) -> Result<TicketResult, String> {
    Err("Integrations available in v0.2. Please update to the latest version.".to_string())
}

#[tauri::command]
pub async fn test_azuredevops_connection(
    _org_url: String,
    _credentials: serde_json::Value,
) -> Result<ConnectionResult, String> {
    Err("Integrations available in v0.2. Please update to the latest version.".to_string())
}

#[tauri::command]
pub async fn create_azuredevops_workitem(
    _issue_id: String,
    _project: String,
    _config: serde_json::Value,
) -> Result<TicketResult, String> {
    Err("Integrations available in v0.2. Please update to the latest version.".to_string())
}

// ─── OAuth2 Commands ────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OAuthInitResponse {
    pub auth_url: String,
    pub state: String,
}

/// Initiate OAuth2 authorization flow for a service.
/// Starts the callback server and returns the authorization URL.
#[tauri::command]
pub async fn initiate_oauth(
    service: String,
    app_state: State<'_, AppState>,
) -> Result<OAuthInitResponse, String> {
    // Start callback server if not already running
    let server_already_running = {
        let shutdown = CALLBACK_SERVER_SHUTDOWN.lock().map_err(|e| e.to_string())?;
        shutdown.is_some()
    };

    if !server_already_running {
        tracing::info!("Starting OAuth callback server");

        let (mut callback_rx, shutdown_tx) =
            crate::integrations::callback_server::start_callback_server(8765)
                .await
                .map_err(|e| format!("Failed to start callback server: {}", e))?;

        // Store shutdown channel
        {
            let mut shutdown = CALLBACK_SERVER_SHUTDOWN.lock().map_err(|e| e.to_string())?;
            *shutdown = Some(shutdown_tx);
        }

        // Clone the Arc fields from app_state for the spawned task
        let db = app_state.db.clone();
        let settings = app_state.settings.clone();
        let app_data_dir = app_state.app_data_dir.clone();

        tokio::spawn(async move {
            let app_state_for_callback = AppState {
                db,
                settings,
                app_data_dir,
            };
            while let Some(callback) = callback_rx.recv().await {
                tracing::info!("Received OAuth callback for state: {}", callback.state);

                // Retrieve service and verifier
                let (service, verifier) = {
                    let mut oauth_state = match OAUTH_STATE.lock() {
                        Ok(state) => state,
                        Err(e) => {
                            tracing::error!("Failed to lock OAuth state: {}", e);
                            continue;
                        }
                    };

                    match oauth_state.remove(&callback.state) {
                        Some((svc, ver)) => (svc, ver),
                        None => {
                            tracing::warn!("Unknown OAuth state: {}", callback.state);
                            continue;
                        }
                    }
                };

                // Call handle_oauth_callback internally
                let result = handle_oauth_callback_internal(
                    service,
                    callback.code,
                    verifier,
                    &app_state_for_callback,
                )
                .await;

                match result {
                    Ok(_) => tracing::info!("OAuth callback handled successfully"),
                    Err(e) => tracing::error!("OAuth callback failed: {}", e),
                }
            }

            tracing::info!("OAuth callback listener stopped");
        });
    }

    // Generate PKCE challenge
    let pkce = crate::integrations::auth::generate_pkce();

    // Generate state key for this OAuth session
    let state_key = uuid::Uuid::now_v7().to_string();

    // Store verifier and service name
    {
        let mut oauth_state = OAUTH_STATE
            .lock()
            .map_err(|e| format!("Failed to lock OAuth state: {}", e))?;
        oauth_state.insert(
            state_key.clone(),
            (service.clone(), pkce.code_verifier.clone()),
        );
    }

    // Build authorization URL based on service
    let (auth_endpoint, client_id, scope, redirect_uri) = match service.as_str() {
        "confluence" => (
            "https://auth.atlassian.com/authorize",
            std::env::var("CONFLUENCE_CLIENT_ID")
                .unwrap_or_else(|_| "confluence-client-id-placeholder".to_string()),
            "read:confluence-space.summary read:confluence-content.summary write:confluence-content",
            "http://localhost:8765/callback",
        ),
        "azuredevops" => (
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
            std::env::var("ADO_CLIENT_ID")
                .unwrap_or_else(|_| "ado-client-id-placeholder".to_string()),
            "vso.work vso.work_write",
            "http://localhost:8765/callback",
        ),
        "servicenow" => {
            // ServiceNow uses basic auth, not OAuth2
            return Err("ServiceNow uses basic authentication, not OAuth2".to_string());
        }
        _ => return Err(format!("Unknown service: {}", service)),
    };

    let auth_url = crate::integrations::auth::build_auth_url(
        auth_endpoint,
        &client_id,
        redirect_uri,
        scope,
        &pkce,
    );

    Ok(OAuthInitResponse {
        auth_url,
        state: state_key,
    })
}

/// Internal function to handle OAuth callback (used by callback server).
async fn handle_oauth_callback_internal(
    service: String,
    code: String,
    verifier: String,
    app_state: &AppState,
) -> Result<(), String> {
    // Get token endpoint and client_id based on service
    let (token_endpoint, client_id, redirect_uri) = match service.as_str() {
        "confluence" => (
            "https://auth.atlassian.com/oauth/token",
            std::env::var("CONFLUENCE_CLIENT_ID")
                .unwrap_or_else(|_| "confluence-client-id-placeholder".to_string()),
            "http://localhost:8765/callback",
        ),
        "azuredevops" => (
            "https://login.microsoftonline.com/common/oauth2/v2.0/token",
            std::env::var("ADO_CLIENT_ID")
                .unwrap_or_else(|_| "ado-client-id-placeholder".to_string()),
            "http://localhost:8765/callback",
        ),
        _ => return Err(format!("Unknown service: {}", service)),
    };

    // Exchange authorization code for access token
    let oauth_token = crate::integrations::auth::exchange_code(
        token_endpoint,
        &client_id,
        &code,
        redirect_uri,
        &verifier,
    )
    .await?;

    // Store token in database with encryption
    let token_hash = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(oauth_token.access_token.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    let encrypted_token = crate::integrations::auth::encrypt_token(&oauth_token.access_token)?;

    let expires_at = Some(
        chrono::DateTime::from_timestamp(oauth_token.expires_at, 0)
            .ok_or_else(|| "Invalid expires_at timestamp".to_string())?
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    );

    // Insert into credentials table
    let db = app_state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    db.execute(
        "INSERT OR REPLACE INTO credentials (id, service, token_hash, encrypted_token, created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            uuid::Uuid::now_v7().to_string(),
            service,
            token_hash,
            encrypted_token,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            expires_at,
        ],
    )
    .map_err(|e| format!("Failed to store credentials: {}", e))?;

    // Log audit event
    let audit_details = serde_json::json!({
        "service": service,
        "token_hash": token_hash,
        "expires_at": expires_at,
    });

    db.execute(
        "INSERT INTO audit_log (id, timestamp, action, entity_type, entity_id, user_id, details)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            uuid::Uuid::now_v7().to_string(),
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            "oauth_callback_success",
            "credential",
            service,
            "local",
            audit_details.to_string(),
        ],
    )
    .map_err(|e| format!("Failed to log audit event: {}", e))?;

    Ok(())
}

/// Handle OAuth2 callback (Tauri command for external/manual calls).
/// This is rarely used since callbacks are handled automatically by the callback server.
#[tauri::command]
pub async fn handle_oauth_callback(
    service: String,
    code: String,
    state_key: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    // Retrieve verifier from temporary state
    let verifier = {
        let mut oauth_state = OAUTH_STATE
            .lock()
            .map_err(|e| format!("Failed to lock OAuth state: {}", e))?;
        oauth_state
            .remove(&state_key)
            .map(|(_svc, ver)| ver)
            .ok_or_else(|| "Invalid or expired OAuth state".to_string())?
    };

    handle_oauth_callback_internal(service, code, verifier, app_state.inner()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_state_storage() {
        let key = "test-key".to_string();
        let service = "confluence".to_string();
        let verifier = "test-verifier".to_string();

        // Store
        {
            let mut state = OAUTH_STATE.lock().unwrap();
            state.insert(key.clone(), (service.clone(), verifier.clone()));
        }

        // Retrieve
        {
            let state = OAUTH_STATE.lock().unwrap();
            assert_eq!(state.get(&key), Some(&(service.clone(), verifier.clone())));
        }

        // Remove
        {
            let mut state = OAUTH_STATE.lock().unwrap();
            state.remove(&key);
        }

        // Verify removed
        {
            let state = OAUTH_STATE.lock().unwrap();
            assert!(!state.contains_key(&key));
        }
    }

    #[test]
    fn test_oauth_state_multiple_keys() {
        let key1 = "key1".to_string();
        let key2 = "key2".to_string();

        {
            let mut state = OAUTH_STATE.lock().unwrap();
            state.insert(
                key1.clone(),
                ("confluence".to_string(), "verifier1".to_string()),
            );
            state.insert(
                key2.clone(),
                ("azuredevops".to_string(), "verifier2".to_string()),
            );
        }

        {
            let mut state = OAUTH_STATE.lock().unwrap();
            state.remove(&key1);
        }

        let state = OAUTH_STATE.lock().unwrap();
        assert!(!state.contains_key(&key1));
        assert!(state.contains_key(&key2));
        assert_eq!(
            state.get(&key2),
            Some(&("azuredevops".to_string(), "verifier2".to_string()))
        );
    }

    #[test]
    fn test_oauth_init_response_serialization() {
        let response = OAuthInitResponse {
            auth_url: "https://example.com/auth".to_string(),
            state: "state-123".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("https://example.com/auth"));
        assert!(json.contains("state-123"));

        let deserialized: OAuthInitResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.auth_url, response.auth_url);
        assert_eq!(deserialized.state, response.state);
    }
}

// ─── Webview-Based Authentication (Option C) ────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct WebviewAuthRequest {
    pub service: String,
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebviewAuthResponse {
    pub success: bool,
    pub message: String,
    pub webview_id: String,
}

/// Open embedded browser window for user to log in.
/// After successful login, call extract_cookies_from_webview to capture session.
#[tauri::command]
pub async fn authenticate_with_webview(
    service: String,
    base_url: String,
    app_handle: tauri::AppHandle,
) -> Result<WebviewAuthResponse, String> {
    let webview_id = format!("{}-auth", service);

    // Open login page in embedded browser
    let _credentials = crate::integrations::webview_auth::authenticate_with_webview(
        app_handle,
        &service,
        &base_url,
    )
    .await?;

    Ok(WebviewAuthResponse {
        success: true,
        message: format!("Login window opened. Complete authentication in the browser."),
        webview_id,
    })
}

/// Extract cookies from webview after user completes login.
/// User should call this after they've successfully logged in.
#[tauri::command]
pub async fn extract_cookies_from_webview(
    service: String,
    webview_id: String,
    app_handle: tauri::AppHandle,
    app_state: State<'_, AppState>,
) -> Result<ConnectionResult, String> {
    // Get the webview window
    let webview_window = app_handle
        .get_webview_window(&webview_id)
        .ok_or_else(|| "Webview window not found".to_string())?;

    // Extract cookies using IPC mechanism (more reliable than platform-specific APIs)
    let cookies = crate::integrations::webview_auth::extract_cookies_via_ipc(
        &webview_window,
        &app_handle,
    )
    .await?;

    if cookies.is_empty() {
        return Err("No cookies found. Make sure you completed the login.".to_string());
    }

    // Encrypt and store cookies in database
    let cookies_json =
        serde_json::to_string(&cookies).map_err(|e| format!("Failed to serialize cookies: {}", e))?;
    let encrypted_cookies = crate::integrations::auth::encrypt_token(&cookies_json)?;

    let token_hash = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(cookies_json.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    // Store in database
    let db = app_state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    db.execute(
        "INSERT OR REPLACE INTO credentials (id, service, token_hash, encrypted_token, created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            uuid::Uuid::now_v7().to_string(),
            service,
            token_hash,
            encrypted_cookies,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            None::<String>, // Cookies don't have explicit expiry
        ],
    )
    .map_err(|e| format!("Failed to store cookies: {}", e))?;

    // Close the webview window
    if let Some(webview) = app_handle.get_webview_window(&webview_id) {
        webview
            .close()
            .map_err(|e| format!("Failed to close webview: {}", e))?;
    }

    Ok(ConnectionResult {
        success: true,
        message: format!("{} authentication saved successfully", service),
    })
}

// ─── Manual Token Authentication (Token Mode) ───────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenAuthRequest {
    pub service: String,
    pub token: String,
    pub token_type: String, // "Bearer", "Basic", "api_token"
    pub base_url: String,
}

/// Store a manually provided token (API key, PAT, etc.)
/// This is the fallback authentication method when OAuth2 and webview don't work.
#[tauri::command]
pub async fn save_manual_token(
    request: TokenAuthRequest,
    app_state: State<'_, AppState>,
) -> Result<ConnectionResult, String> {
    // Validate token by testing connection
    let test_result = match request.service.as_str() {
        "confluence" => {
            let config = crate::integrations::confluence::ConfluenceConfig {
                base_url: request.base_url.clone(),
                access_token: request.token.clone(),
            };
            crate::integrations::confluence::test_connection(&config).await
        }
        "azuredevops" => {
            let config = crate::integrations::azuredevops::AzureDevOpsConfig {
                organization_url: request.base_url.clone(),
                access_token: request.token.clone(),
                project: "".to_string(), // Project not needed for connection test
            };
            crate::integrations::azuredevops::test_connection(&config).await
        }
        "servicenow" => {
            // ServiceNow uses basic auth, token is base64(username:password)
            let config = crate::integrations::servicenow::ServiceNowConfig {
                instance_url: request.base_url.clone(),
                username: "".to_string(), // Encoded in token
                password: request.token.clone(),
            };
            crate::integrations::servicenow::test_connection(&config).await
        }
        _ => return Err(format!("Unknown service: {}", request.service)),
    };

    // If test fails, don't save the token
    if let Ok(result) = &test_result {
        if !result.success {
            return Ok(ConnectionResult {
                success: false,
                message: format!(
                    "Token validation failed: {}. Token not saved.",
                    result.message
                ),
            });
        }
    }

    // Encrypt and store token
    let encrypted_token = crate::integrations::auth::encrypt_token(&request.token)?;

    let token_hash = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(request.token.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    let db = app_state
        .db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    db.execute(
        "INSERT OR REPLACE INTO credentials (id, service, token_hash, encrypted_token, created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            uuid::Uuid::now_v7().to_string(),
            request.service,
            token_hash,
            encrypted_token,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            None::<String>,
        ],
    )
    .map_err(|e| format!("Failed to store token: {}", e))?;

    // Log audit event
    db.execute(
        "INSERT INTO audit_log (id, timestamp, action, entity_type, entity_id, user_id, details)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            uuid::Uuid::now_v7().to_string(),
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            "manual_token_saved",
            "credential",
            request.service,
            "local",
            serde_json::json!({
                "token_type": request.token_type,
                "token_hash": token_hash,
            })
            .to_string(),
        ],
    )
    .map_err(|e| format!("Failed to log audit event: {}", e))?;

    Ok(ConnectionResult {
        success: true,
        message: format!("{} token saved and validated successfully", request.service),
    })
}
