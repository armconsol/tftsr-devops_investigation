use crate::db::models::{AuditEntry, AuditFilter};
use crate::ollama::{
    hardware, installer, manager, recommender, InstallGuide, ModelRecommendation, OllamaModel,
    OllamaStatus,
};
use crate::state::{AppSettings, AppState};

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

// Security note: the bundled binary's integrity is guaranteed by the CI release pipeline
// which verifies SHA256 checksums against Ollama's published sha256sums.txt before bundling.
// Runtime re-verification is not performed here; the app bundle itself is the trust boundary.
#[tauri::command]
pub async fn install_ollama_from_bundle(
    app: tauri::AppHandle,
) -> Result<String, String> {
    use std::fs;
    use std::path::PathBuf;
    use tauri::Manager;

    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e: tauri::Error| e.to_string())?
        .join("ollama")
        .join(if cfg!(windows) { "ollama.exe" } else { "ollama" });

    if !resource_path.exists() {
        return Err("Bundled Ollama not found in resources".to_string());
    }

    #[cfg(unix)]
    let install_path = PathBuf::from("/usr/local/bin/ollama");
    #[cfg(windows)]
    let install_path = {
        let local_app_data = std::env::var("LOCALAPPDATA").map_err(|e| e.to_string())?;
        PathBuf::from(local_app_data)
            .join("Programs")
            .join("Ollama")
            .join("ollama.exe")
    };

    if let Some(parent) = install_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::copy(&resource_path, &install_path).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&install_path, fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
    }

    Ok(format!("Ollama installed to {}", install_path.display()))
}
