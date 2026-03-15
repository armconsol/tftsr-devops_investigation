use crate::integrations::{ConnectionResult, PublishResult, TicketResult};

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
