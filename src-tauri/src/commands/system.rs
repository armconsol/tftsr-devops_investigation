use crate::db::models::{AuditEntry, AuditFilter};
use crate::ollama::{
    hardware, installer, manager, recommender, InstallGuide, ModelRecommendation, OllamaModel,
    OllamaStatus,
};
use crate::state::{AppSettings, AppState, ProviderConfig};
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
    // `update_channel` is intentionally not read here: the channel concept
    // was removed, but old clients/persisted payloads may still send the key
    // — it is silently ignored rather than erroring.
    if let Some(enabled) = partial_settings
        .get("debug_logging_enabled")
        .and_then(|v| v.as_bool())
    {
        settings.debug_logging_enabled = enabled;
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
    let previous_debug_logging = settings.debug_logging_enabled;
    let debug_logging_update = apply_partial_settings(&mut settings, &partial_settings);
    if let Some(enabled) = debug_logging_update {
        if let Err(e) = crate::set_debug_logging_enabled(enabled) {
            settings.debug_logging_enabled = previous_debug_logging;
            return Err(e);
        }
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

/// Get the application version. Read from the Tauri package info (populated
/// from `tauri.conf.json` at build time) rather than environment variables —
/// packaged builds don't have `APP_VERSION`/`CARGO_PKG_VERSION` set at
/// runtime, so reading them here always reported a stale/wrong version.
#[tauri::command]
pub async fn get_app_version(app: tauri::AppHandle) -> Result<String, String> {
    Ok(app.package_info().version.to_string())
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

/// Parse a (possibly `v`-prefixed) semver-ish string into
/// `(major, minor, patch, prerelease)`. Returns `None` for empty input.
fn parse_version(v: &str) -> Option<(u64, u64, u64, Option<String>)> {
    if v.is_empty() {
        return None;
    }
    let v = v.trim_start_matches('v');
    let (core, prerelease) = match v.split_once('-') {
        Some((core, pre)) => (core, Some(pre.to_string())),
        None => (v, None),
    };
    let mut parts = core.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    let major = parts.next().unwrap_or(0);
    let minor = parts.next().unwrap_or(0);
    let patch = parts.next().unwrap_or(0);
    Some((major, minor, patch, prerelease))
}

/// Compare two version strings. A release with no prerelease suffix is
/// considered newer than a prerelease of the same core version (matching
/// semver precedence: `3.1.0` > `3.1.0-beta.9`). Prerelease suffixes on
/// otherwise-equal cores are compared as their trailing numeric component
/// when present, falling back to a plain string comparison.
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let (a_core, a_pre) = match parse_version(a) {
        Some((maj, min, pat, pre)) => ((maj, min, pat), pre),
        None => {
            return if parse_version(b).is_some() {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        }
    };
    let (b_core, b_pre) = match parse_version(b) {
        Some((maj, min, pat, pre)) => ((maj, min, pat), pre),
        None => return Ordering::Greater,
    };

    match a_core.cmp(&b_core) {
        Ordering::Equal => match (&a_pre, &b_pre) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(ap), Some(bp)) => {
                let a_num = ap.rsplit('.').next().and_then(|n| n.parse::<u64>().ok());
                let b_num = bp.rsplit('.').next().and_then(|n| n.parse::<u64>().ok());
                match (a_num, b_num) {
                    (Some(an), Some(bn)) => an.cmp(&bn),
                    _ => ap.cmp(bp),
                }
            }
        },
        other => other,
    }
}

/// Pick the highest-versioned non-draft release from a Gitea releases API
/// response. Prereleases are included — installs may come from a beta
/// prerelease, so the channel concept (and its filtering) has been removed;
/// callers always get the newest thing that was actually published.
fn pick_latest_release(releases: &[serde_json::Value]) -> Option<&serde_json::Value> {
    releases
        .iter()
        .filter(|r| !r["draft"].as_bool().unwrap_or(false))
        .max_by(|a, b| {
            let a_tag = a["tag_name"].as_str().unwrap_or("");
            let b_tag = b["tag_name"].as_str().unwrap_or("");
            compare_versions(a_tag, b_tag)
        })
}

#[tauri::command]
pub async fn check_app_updates(
    app: tauri::AppHandle,
    _state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let current_version = app.package_info().version.to_string();

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

    let release = pick_latest_release(&releases).ok_or_else(|| "No releases found".to_string())?;

    let latest_tag = release["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v')
        .to_string();

    let update_available = compare_versions(&latest_tag, &current_version).is_gt();

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

#[cfg(test)]
mod updater_tests {
    use super::*;

    #[test]
    fn test_compare_versions_core_numeric() {
        assert!(compare_versions("1.3.0", "1.2.2").is_gt());
        assert!(compare_versions("2.0.0", "1.9.9").is_gt());
        assert!(compare_versions("1.2.2", "1.2.2").is_eq());
        assert!(compare_versions("1.2.1", "1.2.2").is_lt());
        assert!(compare_versions("0.9.0", "1.0.0").is_lt());
        assert!(compare_versions("1.2.3", "1.2.2").is_gt());
    }

    #[test]
    fn test_compare_versions_empty() {
        assert!(compare_versions("", "1.0.0").is_lt());
        assert!(compare_versions("1.0.0", "").is_gt());
    }

    #[test]
    fn test_compare_versions_release_beats_prerelease_of_same_core() {
        assert!(compare_versions("3.1.0", "3.1.0-beta.9").is_gt());
        assert!(compare_versions("3.1.0-beta.9", "3.1.0").is_lt());
    }

    #[test]
    fn test_compare_versions_prerelease_ordering() {
        assert!(compare_versions("3.1.0-beta.9", "3.1.0-beta.2").is_gt());
        assert!(compare_versions("3.1.0-beta.2", "3.1.0-beta.9").is_lt());
    }

    #[test]
    fn test_compare_versions_prerelease_vs_older_stable() {
        // A beta of the next release is still newer than the last stable.
        assert!(compare_versions("3.1.0-beta.2", "3.0.0").is_gt());
    }

    #[test]
    fn test_pick_latest_release_skips_drafts_includes_prereleases() {
        let releases = vec![
            serde_json::json!({"tag_name": "v3.0.0", "prerelease": false, "draft": false}),
            serde_json::json!({"tag_name": "v3.2.0", "prerelease": false, "draft": true}),
            serde_json::json!({"tag_name": "v3.1.0-beta.9", "prerelease": true, "draft": false}),
        ];
        let picked = pick_latest_release(&releases).unwrap();
        assert_eq!(picked["tag_name"].as_str().unwrap(), "v3.1.0-beta.9");
    }

    #[test]
    fn test_pick_latest_release_prefers_stable_over_older_prerelease_tag() {
        let releases = vec![
            serde_json::json!({"tag_name": "v3.1.0-beta.2", "prerelease": true, "draft": false}),
            serde_json::json!({"tag_name": "v3.1.0", "prerelease": false, "draft": false}),
        ];
        let picked = pick_latest_release(&releases).unwrap();
        assert_eq!(picked["tag_name"].as_str().unwrap(), "v3.1.0");
    }

    #[test]
    fn test_pick_latest_release_empty_or_all_drafts() {
        assert!(pick_latest_release(&[]).is_none());
        let releases = vec![serde_json::json!({"tag_name": "v1.0.0", "draft": true})];
        assert!(pick_latest_release(&releases).is_none());
    }

    #[test]
    fn test_apply_partial_settings_extracts_debug_logging_toggle() {
        let mut settings = AppSettings::default();
        let partial = serde_json::json!({
            "debug_logging_enabled": true
        });

        let update = apply_partial_settings(&mut settings, &partial);
        assert_eq!(update, Some(true));
        assert!(settings.debug_logging_enabled);
    }

    #[test]
    fn test_apply_partial_settings_ignores_legacy_update_channel_key() {
        // Old persisted settings/partial payloads may still send update_channel
        // from a pre-upgrade client; it must be silently ignored, not error.
        let mut settings = AppSettings::default();
        let partial = serde_json::json!({
            "update_channel": "beta",
            "theme": "light"
        });
        apply_partial_settings(&mut settings, &partial);
        assert_eq!(settings.theme, "light");
    }
}
