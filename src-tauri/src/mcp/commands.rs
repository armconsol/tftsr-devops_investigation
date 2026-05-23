use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex as TokioMutex;
use tracing::{info, warn};

use crate::mcp::models::{
    CreateMcpServerRequest, McpServer, McpServerStatus, UpdateMcpServerRequest,
};
use crate::mcp::store::{
    create_server, delete_server, get_resource_count, get_server, get_tool_count, list_servers,
    toggle_server, update_discovery_status, update_server,
};
use crate::state::AppState;

#[tauri::command]
pub async fn list_mcp_servers(state: State<'_, AppState>) -> Result<Vec<McpServer>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut servers = list_servers(&db)?;
    // Never expose encrypted auth values to the frontend
    for s in &mut servers {
        s.auth_value = None;
    }
    Ok(servers)
}

#[tauri::command]
pub async fn create_mcp_server(
    request: CreateMcpServerRequest,
    state: State<'_, AppState>,
) -> Result<McpServer, String> {
    let mut server = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        create_server(&db, &request)?
    };
    server.auth_value = None;
    Ok(server)
}

#[tauri::command]
pub async fn update_mcp_server(
    id: String,
    request: UpdateMcpServerRequest,
    state: State<'_, AppState>,
) -> Result<McpServer, String> {
    let mut server = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        update_server(&db, &id, &request)?
    };
    server.auth_value = None;
    Ok(server)
}

#[tauri::command]
pub async fn delete_mcp_server(
    id: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        delete_server(&db, &id)?;
    }
    // Remove live connection if present
    let mut connections = state.mcp_connections.lock().await;
    connections.remove(&id);
    drop(connections);

    info!(server_id = %id, "MCP server deleted");
    let _ = app_handle; // suppress unused warning
    Ok(())
}

#[tauri::command]
pub async fn toggle_mcp_server(
    id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    toggle_server(&db, &id, enabled)?;
    Ok(())
}

#[tauri::command]
pub async fn discover_mcp_server(
    id: String,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<McpServerStatus, String> {
    let server = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        get_server(&db, &id)?.ok_or_else(|| format!("Server {id} not found"))?
    };

    match crate::mcp::discovery::discover_server(&server, &app_handle).await {
        Ok(conn) => {
            let mut connections = state.mcp_connections.lock().await;
            connections.insert(id.clone(), Arc::new(TokioMutex::new(conn)));
            drop(connections);

            let (tool_count, resource_count, last_discovered_at) = {
                let db = state.db.lock().map_err(|e| e.to_string())?;
                let tc = get_tool_count(&db, &id)?;
                let rc = get_resource_count(&db, &id)?;
                let srv = get_server(&db, &id)?.unwrap();
                (tc, rc, srv.last_discovered_at)
            };

            Ok(McpServerStatus {
                server_id: id,
                status: "connected".to_string(),
                error: None,
                tool_count,
                resource_count,
                last_discovered_at,
            })
        }
        Err(e) => {
            {
                let db = state.db.lock().map_err(|db_err| db_err.to_string())?;
                update_discovery_status(&db, &id, "error", Some(&e))?;
            }
            Ok(McpServerStatus {
                server_id: id,
                status: "error".to_string(),
                error: Some(e),
                tool_count: 0,
                resource_count: 0,
                last_discovered_at: None,
            })
        }
    }
}

#[tauri::command]
pub async fn get_mcp_server_status(
    id: String,
    state: State<'_, AppState>,
) -> Result<McpServerStatus, String> {
    let (server, tool_count, resource_count) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let srv = get_server(&db, &id)?.ok_or_else(|| format!("Server {id} not found"))?;
        let tc = get_tool_count(&db, &id)?;
        let rc = get_resource_count(&db, &id)?;
        (srv, tc, rc)
    };

    Ok(McpServerStatus {
        server_id: id,
        status: server.discovery_status,
        error: server.discovery_error,
        tool_count,
        resource_count,
        last_discovered_at: server.last_discovered_at,
    })
}

#[tauri::command]
pub async fn initiate_mcp_oauth(
    id: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let server = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        get_server(&db, &id)?.ok_or_else(|| format!("Server {id} not found"))?
    };

    if server.auth_type != "oauth2" {
        return Err(format!(
            "Server {} uses auth_type '{}', not oauth2",
            id, server.auth_type
        ));
    }

    let config: serde_json::Value =
        serde_json::from_str(&server.transport_config).unwrap_or_default();

    let auth_endpoint = config
        .get("auth_endpoint")
        .and_then(|v| v.as_str())
        .ok_or("OAuth2 transport_config missing 'auth_endpoint'")?
        .to_string();

    let client_id = config
        .get("client_id")
        .and_then(|v| v.as_str())
        .ok_or("OAuth2 transport_config missing 'client_id'")?
        .to_string();

    let token_endpoint = config
        .get("token_endpoint")
        .and_then(|v| v.as_str())
        .ok_or("OAuth2 transport_config missing 'token_endpoint'")?
        .to_string();

    let redirect_uri = "http://localhost:12345/mcp-oauth-callback".to_string();
    let scope = config
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let pkce = crate::integrations::auth::generate_pkce();
    let auth_url = crate::integrations::auth::build_auth_url(
        &auth_endpoint,
        &client_id,
        &redirect_uri,
        &scope,
        &pkce,
    );

    // Open WebView window for OAuth
    let window_label = format!("mcp-oauth-{id}");
    let _ = tauri::WebviewWindowBuilder::new(
        &app_handle,
        &window_label,
        tauri::WebviewUrl::External(
            url::Url::parse(&auth_url).map_err(|e| format!("Invalid OAuth URL: {e}"))?,
        ),
    )
    .title(format!("Authenticate: {}", server.name))
    .inner_size(800.0, 700.0)
    .build()
    .map_err(|e| format!("Failed to open OAuth window: {e}"))?;

    // Monitor URL changes for the redirect callback
    // For now, return Ok and let the user copy the code manually
    // Full implementation would poll the webview URL
    warn!(server_id = %id, "OAuth2 WebView opened — token exchange not yet automated");

    // Exchange code → token
    // In production this would be driven by webview URL monitoring (see integrations::webview_auth)
    // This stub allows the UI to open the browser without crashing.
    let _ = (token_endpoint, pkce);

    Ok(())
}
