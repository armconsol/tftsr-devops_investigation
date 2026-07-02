use crate::db::models::{AuditEntry, AuditFilter};
use crate::ollama::{
    hardware, installer, manager, recommender, InstallGuide, ModelRecommendation, OllamaModel,
    OllamaStatus,
};
use crate::state::{AppSettings, AppState, ProviderConfig};
use std::env;
use tauri_plugin_opener::OpenerExt;

// --- Ollama commands ---

#[tauri::command]
pub async fn check_ollama_installed() -> Result<OllamaStatus, String> {
    installer::check_ollama().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_ollama_install_guide(platform: String) -> Result<InstallGuide, String> {
    Ok(installer::get_install_instructions(&platform))
}

#[tauri::command]
pub async fn list_ollama_models() -> Result<Vec<OllamaModel>, String> {
    manager::list_models().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pull_ollama_model(
    app_handle: tauri::AppHandle,
    model_name: String,
) -> Result<(), String> {
    manager::pull_model(app_handle, &model_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_ollama_model(model_name: String) -> Result<(), String> {
    manager::delete_model(&model_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn detect_hardware() -> Result<hardware::HardwareInfo, String> {
    hardware::probe_hardware().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn recommend_models() -> Result<Vec<ModelRecommendation>, String> {
    let hw = hardware::probe_hardware().map_err(|e| e.to_string())?;
    Ok(recommender::recommend_models(&hw))
}

// --- Settings commands ---

fn apply_partial_settings(
    settings: &mut AppSettings,
    partial_settings: &serde_json::Value,
) -> Option<bool> {
    if let Some(theme) = partial_settings.get("theme").and_then(|v| v.as_str()) {
        settings.theme = theme.to_string();
    }
    if let Some(active_provider) = partial_settings
        .get("active_provider")
        .and_then(|v| v.as_str())
    {
        settings.active_provider = Some(active_provider.to_string());
    }
    if let Some(ch) = partial_settings
        .get("update_channel")
        .and_then(|v| v.as_str())
    {
        settings.update_channel = ch.to_string();
    }
    if let Some(enabled) = partial_settings
        .get("debug_logging_enabled")
        .and_then(|v| v.as_bool())
    {
        return Some(enabled);
    }

    None
}

#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, AppState>) -> Result<AppSettings, String> {
    state
        .settings
        .lock()
        .map(|s| s.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_settings(
    partial_settings: serde_json::Value,
    state: tauri::State<'_, AppState>,
) -> Result<AppSettings, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    let debug_logging_update = apply_partial_settings(&mut settings, &partial_settings);
    if let Some(enabled) = debug_logging_update {
        crate::set_debug_logging_enabled(enabled)?;
        settings.debug_logging_enabled = enabled;
    }

    Ok(settings.clone())
}

// --- Audit log command ---

#[tauri::command]
pub async fn get_audit_log(
    filter: AuditFilter,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AuditEntry>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let limit = filter.limit.unwrap_or(100);

    let mut sql = String::from(
        "SELECT id, timestamp, action, entity_type, entity_id, user_id, details \
         FROM audit_log WHERE 1=1",
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if let Some(ref action) = filter.action {
        sql.push_str(&format!(" AND action = ?{index}", index = params.len() + 1));
        params.push(Box::new(action.clone()));
    }
    if let Some(ref entity_type) = filter.entity_type {
        sql.push_str(&format!(
            " AND entity_type = ?{index}",
            index = params.len() + 1
        ));
        params.push(Box::new(entity_type.clone()));
    }
    if let Some(ref entity_id) = filter.entity_id {
        sql.push_str(&format!(
            " AND entity_id = ?{index}",
            index = params.len() + 1
        ));
        params.push(Box::new(entity_id.clone()));
    }

    sql.push_str(" ORDER BY timestamp DESC");
    sql.push_str(&format!(" LIMIT ?{index}", index = params.len() + 1));
    params.push(Box::new(limit));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(AuditEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                action: row.get(2)?,
                entity_type: row.get(3)?,
                entity_id: row.get(4)?,
                user_id: row.get(5)?,
                details: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(rows)
}

// --- AI Provider persistence commands ---

/// Save an AI provider configuration to encrypted database
#[tauri::command]
pub async fn save_ai_provider(
    provider: ProviderConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Encrypt the API key
    let encrypted_key = crate::integrations::auth::encrypt_token(&provider.api_key)?;

    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.execute(
        "INSERT OR REPLACE INTO ai_providers
         (id, name, provider_type, api_url, encrypted_api_key, model, max_tokens, temperature,
          custom_endpoint_path, custom_auth_header, custom_auth_prefix, api_format, user_id, use_datastore_upload, supports_tool_calling, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, datetime('now'))",
        rusqlite::params![
            uuid::Uuid::now_v7().to_string(),
            provider.name,
            provider.provider_type,
            provider.api_url,
            encrypted_key,
            provider.model,
            provider.max_tokens,
            provider.temperature,
            provider.custom_endpoint_path,
            provider.custom_auth_header,
            provider.custom_auth_prefix,
            provider.api_format,
            provider.user_id,
            provider.use_datastore_upload,
            provider.supports_tool_calling,
        ],
    )
    .map_err(|e| format!("Failed to save AI provider: {e}"))?;

    Ok(())
}

/// Load all AI provider configurations from database
#[tauri::command]
pub async fn load_ai_providers(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ProviderConfig>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db
        .prepare(
            "SELECT name, provider_type, api_url, encrypted_api_key, model, max_tokens, temperature,
                    custom_endpoint_path, custom_auth_header, custom_auth_prefix, api_format, user_id, use_datastore_upload, supports_tool_calling
             FROM ai_providers
             ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let providers = stmt
        .query_map([], |row| {
            let encrypted_key: String = row.get(3)?;

            Ok((
                row.get::<_, String>(0)?,          // name
                row.get::<_, String>(1)?,          // provider_type
                row.get::<_, String>(2)?,          // api_url
                encrypted_key,                     // encrypted_api_key
                row.get::<_, String>(4)?,          // model
                row.get::<_, Option<u32>>(5)?,     // max_tokens
                row.get::<_, Option<f64>>(6)?,     // temperature
                row.get::<_, Option<String>>(7)?,  // custom_endpoint_path
                row.get::<_, Option<String>>(8)?,  // custom_auth_header
                row.get::<_, Option<String>>(9)?,  // custom_auth_prefix
                row.get::<_, Option<String>>(10)?, // api_format
                row.get::<_, Option<String>>(11)?, // user_id
                row.get::<_, Option<bool>>(12)?,   // use_datastore_upload
                row.get::<_, Option<bool>>(13)?,   // supports_tool_calling
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter_map(
            |(
                name,
                provider_type,
                api_url,
                encrypted_key,
                model,
                max_tokens,
                temperature,
                custom_endpoint_path,
                custom_auth_header,
                custom_auth_prefix,
                api_format,
                user_id,
                use_datastore_upload,
                supports_tool_calling,
            )| {
                // Decrypt the API key
                let api_key = crate::integrations::auth::decrypt_token(&encrypted_key).ok()?;

                Some(ProviderConfig {
                    name,
                    provider_type,
                    api_url,
                    api_key,
                    model,
                    max_tokens,
                    temperature,
                    custom_endpoint_path,
                    custom_auth_header,
                    custom_auth_prefix,
                    api_format,
                    session_id: None, // Session IDs are not persisted
                    user_id,
                    use_datastore_upload,
                    supports_tool_calling,
                })
            },
        )
        .collect();

    Ok(providers)
}

/// Delete an AI provider configuration
#[tauri::command]
pub async fn delete_ai_provider(
    name: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.execute("DELETE FROM ai_providers WHERE name = ?1", [&name])
        .map_err(|e| format!("Failed to delete AI provider: {e}"))?;

    Ok(())
}

/// Get the application version from build-time environment
#[tauri::command]
pub async fn get_app_version() -> Result<String, String> {
    env::var("APP_VERSION")
        .or_else(|_| env::var("CARGO_PKG_VERSION"))
        .map_err(|e| format!("Failed to get version: {e}"))
}

// --- Sudo credential commands ---

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SudoConfigStatus {
    pub configured: bool,
    pub username: String,
    pub updated_at: String,
}

/// Resolve the OS username to bind sudo credentials to.
fn resolve_sudo_username(provided: Option<String>) -> String {
    provided
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| {
            std::env::var("USER")
                .or_else(|_| std::env::var("LOGNAME"))
                .unwrap_or_else(|_| "local".to_string())
        })
}

#[tauri::command]
pub async fn set_sudo_password(
    password: String,
    username: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let encrypted = crate::integrations::auth::encrypt_token(&password)?;
    let uname = resolve_sudo_username(username);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    // DELETE then INSERT to guarantee exactly one row at all times.
    // INSERT OR REPLACE with a freshly generated UUID never matches the
    // existing primary key, so it inserts an additional row instead of
    // replacing — this is the correct singleton pattern for SQLite.
    db.execute("DELETE FROM sudo_config", [])
        .map_err(|e| format!("Failed to clear sudo config: {e}"))?;
    db.execute(
        "INSERT INTO sudo_config (id, username, encrypted_password, created_at, updated_at) \
         VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))",
        rusqlite::params![uuid::Uuid::now_v7().to_string(), uname, encrypted],
    )
    .map_err(|e| format!("Failed to store sudo config: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn get_sudo_config_status(
    state: tauri::State<'_, AppState>,
) -> Result<SudoConfigStatus, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result: Option<(String, String)> = db
        .prepare("SELECT username, updated_at FROM sudo_config LIMIT 1")
        .and_then(|mut stmt| {
            stmt.query_row([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
        })
        .ok();
    match result {
        Some((username, updated_at)) => Ok(SudoConfigStatus {
            configured: true,
            username,
            updated_at,
        }),
        None => Ok(SudoConfigStatus {
            configured: false,
            username: String::new(),
            updated_at: String::new(),
        }),
    }
}

#[tauri::command]
pub async fn test_sudo_password(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let (encrypted, stored_username) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.prepare("SELECT encrypted_password, username FROM sudo_config LIMIT 1")
            .and_then(|mut stmt| {
                stmt.query_row([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
            })
            .ok()
            .ok_or("No sudo password configured".to_string())?
    };
    let password = crate::integrations::auth::decrypt_token(&encrypted)?;
    // Scope the test to the stored username so credentials can only be
    // verified for the user they were saved for.
    let result = if stored_username.is_empty() {
        crate::commands::agentic::run_sudo_command(&password, &["true"])
    } else {
        crate::commands::agentic::run_sudo_command(&password, &["-u", &stored_username, "true"])
    }
    .map_err(|e| format!("Sudo test failed: {e}"))?;
    Ok(result.success)
}

#[tauri::command]
pub async fn clear_sudo_password(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM sudo_config", [])
        .map_err(|e| format!("Failed to clear sudo config: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod sudo_tests {
    use super::*;

    fn setup_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_set_sudo_singleton_delete_then_insert() {
        let conn = setup_db();
        // Insert two stale rows directly to simulate the old broken behaviour
        conn.execute(
            "INSERT INTO sudo_config (id, username, encrypted_password) VALUES ('id1', 'alice', 'enc1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO sudo_config (id, username, encrypted_password) VALUES ('id2', 'bob', 'enc2')",
            [],
        )
        .unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sudo_config", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2);

        // Apply the correct singleton pattern
        conn.execute("DELETE FROM sudo_config", []).unwrap();
        conn.execute(
            "INSERT INTO sudo_config (id, username, encrypted_password) VALUES ('id3', 'charlie', 'enc3')",
            [],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sudo_config", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "exactly one row must remain after set");

        let username: String = conn
            .query_row("SELECT username FROM sudo_config", [], |r| r.get(0))
            .unwrap();
        assert_eq!(username, "charlie");
    }

    #[test]
    fn test_resolve_sudo_username_uses_provided() {
        let result = resolve_sudo_username(Some("alice".to_string()));
        assert_eq!(result, "alice");
    }

    #[test]
    fn test_resolve_sudo_username_rejects_blank() {
        let result = resolve_sudo_username(Some("   ".to_string()));
        // blank string should fall through to env-based default
        assert!(!result.trim().is_empty(), "username must never be blank");
    }

    #[test]
    fn test_resolve_sudo_username_defaults_to_env() {
        let env_user = std::env::var("USER")
            .or_else(|_| std::env::var("LOGNAME"))
            .unwrap_or_else(|_| "local".to_string());
        let result = resolve_sudo_username(None);
        assert_eq!(result, env_user);
    }
}

// --- Updater commands ---

fn is_newer_version(latest: &str, current: &str) -> bool {
    if latest.is_empty() || current.is_empty() {
        return false;
    }
    let parse_version =
        |v: &str| -> Vec<u64> { v.split('.').filter_map(|p| p.parse().ok()).collect() };
    let latest_parts = parse_version(latest);
    let current_parts = parse_version(current);
    for i in 0..latest_parts.len().max(current_parts.len()) {
        let l = latest_parts.get(i).copied().unwrap_or(0);
        let c = current_parts.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }
    false
}

#[tauri::command]
pub async fn check_app_updates(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let current_version = app.package_info().version.to_string();

    let channel = {
        state
            .settings
            .lock()
            .map_err(|e| e.to_string())?
            .update_channel
            .clone()
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let response = client
        .get(
            "https://gogs.tftsr.com/api/v1/repos/sarman/tftsr-devops_investigation/releases?limit=20",
        )
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to check for updates: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Update server returned status: {}",
            response.status()
        ));
    }

    let releases: Vec<serde_json::Value> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse update response: {e}"))?;

    let release = releases
        .iter()
        .find(|r| {
            let is_pre = r["prerelease"].as_bool().unwrap_or(false);
            let is_draft = r["draft"].as_bool().unwrap_or(false);
            if is_draft {
                return false;
            }
            match channel.as_str() {
                "beta" => is_pre,
                _ => !is_pre,
            }
        })
        .ok_or_else(|| format!("No release found for channel: {channel}"))?;

    let latest_tag = release["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v')
        .to_string();

    let update_available = is_newer_version(&latest_tag, &current_version);

    let release_url = release["html_url"]
        .as_str()
        .unwrap_or("https://gogs.tftsr.com/sarman/tftsr-devops_investigation/releases")
        .to_string();

    let body = release["body"].as_str().unwrap_or("").to_string();

    Ok(serde_json::json!({
        "updateAvailable": update_available,
        "currentVersion": current_version,
        "latestVersion": latest_tag,
        "releaseUrl": release_url,
        "releaseNotes": body
    }))
}

#[tauri::command]
pub async fn install_app_updates(app: tauri::AppHandle) -> Result<(), String> {
    app.opener()
        .open_url(
            "https://gogs.tftsr.com/sarman/tftsr-devops_investigation/releases",
            None::<&str>,
        )
        .map_err(|e| format!("Failed to open browser: {e}"))
}

#[tauri::command]
pub async fn get_update_channel(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state
        .settings
        .lock()
        .map(|s| s.update_channel.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_update_channel(
    channel: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .settings
        .lock()
        .map_err(|e| e.to_string())?
        .update_channel = channel;
    Ok(())
}

#[cfg(test)]
mod updater_tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("1.3.0", "1.2.2"));
        assert!(is_newer_version("2.0.0", "1.9.9"));
        assert!(!is_newer_version("1.2.2", "1.2.2"));
        assert!(!is_newer_version("1.2.1", "1.2.2"));
        assert!(!is_newer_version("0.9.0", "1.0.0"));
        assert!(is_newer_version("1.2.3", "1.2.2"));
    }

    #[test]
    fn test_is_newer_version_empty() {
        assert!(!is_newer_version("", "1.0.0"));
        assert!(!is_newer_version("1.0.0", ""));
    }

    #[test]
    fn test_update_channel_default() {
        let settings = AppSettings::default();
        assert_eq!(settings.update_channel, "stable");
    }

    #[test]
    fn test_update_channel_serialization() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"stable\""));
    }

    #[test]
    fn test_apply_partial_settings_extracts_debug_logging_toggle() {
        let mut settings = AppSettings::default();
        let partial = serde_json::json!({
            "debug_logging_enabled": true
        });

        let update = apply_partial_settings(&mut settings, &partial);
        assert_eq!(update, Some(true));
        assert!(!settings.debug_logging_enabled);
    }
}
