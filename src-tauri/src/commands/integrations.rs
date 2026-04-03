use crate::integrations::{ConnectionResult, PublishResult, TicketResult};
use crate::state::AppState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

// Global OAuth state storage (verifier per state key)
lazy_static::lazy_static! {
    static ref OAUTH_STATE: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
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
/// Returns the authorization URL and a state key.
#[tauri::command]
pub async fn initiate_oauth(
    service: String,
    _state: State<'_, AppState>,
) -> Result<OAuthInitResponse, String> {
    // Generate PKCE challenge
    let pkce = crate::integrations::auth::generate_pkce();

    // Generate state key for this OAuth session
    let state_key = uuid::Uuid::now_v7().to_string();

    // Store verifier temporarily
    {
        let mut oauth_state = OAUTH_STATE
            .lock()
            .map_err(|e| format!("Failed to lock OAuth state: {}", e))?;
        oauth_state.insert(state_key.clone(), pkce.code_verifier.clone());
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

/// Handle OAuth2 callback after user authorization.
/// Exchanges authorization code for access token and stores it.
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
            .ok_or_else(|| "Invalid or expired OAuth state".to_string())?
    };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_state_storage() {
        let key = "test-key".to_string();
        let verifier = "test-verifier".to_string();

        // Store
        {
            let mut state = OAUTH_STATE.lock().unwrap();
            state.insert(key.clone(), verifier.clone());
        }

        // Retrieve
        {
            let state = OAUTH_STATE.lock().unwrap();
            assert_eq!(state.get(&key), Some(&verifier));
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
            state.insert(key1.clone(), "verifier1".to_string());
            state.insert(key2.clone(), "verifier2".to_string());
        }

        {
            let mut state = OAUTH_STATE.lock().unwrap();
            state.remove(&key1);
        }

        let state = OAUTH_STATE.lock().unwrap();
        assert!(!state.contains_key(&key1));
        assert!(state.contains_key(&key2));
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
