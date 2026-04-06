use crate::db::models::{AuditEntry, AuditFilter};
use crate::ollama::{
    hardware, installer, manager, recommender, InstallGuide, ModelRecommendation, OllamaModel,
    OllamaStatus,
};
use crate::state::{AppSettings, AppState, ProviderConfig};

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

    if let Some(theme) = partial_settings.get("theme").and_then(|v| v.as_str()) {
        settings.theme = theme.to_string();
    }
    if let Some(active_provider) = partial_settings
        .get("active_provider")
        .and_then(|v| v.as_str())
    {
        settings.active_provider = Some(active_provider.to_string());
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
          custom_endpoint_path, custom_auth_header, custom_auth_prefix, api_format, user_id, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, datetime('now'))",
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
        ],
    )
    .map_err(|e| format!("Failed to save AI provider: {}", e))?;

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
                    custom_endpoint_path, custom_auth_header, custom_auth_prefix, api_format, user_id
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
        .map_err(|e| format!("Failed to delete AI provider: {}", e))?;

    Ok(())
}
