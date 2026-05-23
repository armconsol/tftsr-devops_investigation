use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tracing::{info, warn};

use crate::mcp::client::{connect_http, connect_stdio, list_resources, list_tools, McpConnection};
use crate::mcp::models::McpServer;
use crate::mcp::store::{
    get_server_auth_value, list_servers, replace_resources, replace_tools, update_discovery_status,
};

/// Discover a single MCP server: connect, list tools/resources, persist.
/// Returns the updated connection on success or a descriptive error string.
/// Enforces a 60-second hard timeout over the entire connect + discover sequence.
pub async fn discover_server(
    server: &McpServer,
    app_handle: &tauri::AppHandle,
) -> Result<McpConnection, String> {
    tokio::time::timeout(
        std::time::Duration::from_secs(60),
        discover_server_inner(server, app_handle),
    )
    .await
    .map_err(|_| format!("Discovery of '{}' timed out after 60s", server.name))?
}

async fn discover_server_inner(
    server: &McpServer,
    app_handle: &tauri::AppHandle,
) -> Result<McpConnection, String> {
    use tauri::Manager;

    let state = app_handle.state::<crate::state::AppState>();

    // Decrypt auth value if present
    let auth_value = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        get_server_auth_value(&db, &server.id)?
    };

    // Connect based on transport type
    let conn = match server.transport_type.as_str() {
        "stdio" => {
            let config: serde_json::Value =
                serde_json::from_str(&server.transport_config).unwrap_or_default();
            let command = config
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "stdio transport_config missing 'command' field".to_string())?;
            let args: Vec<String> = config
                .get("args")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            connect_stdio(command, &args).await?
        }
        "http" => {
            let auth_header = auth_value.as_deref();
            connect_http(&server.url, auth_header).await?
        }
        other => return Err(format!("Unknown transport type: {other}")),
    };

    // List tools and resources
    let tools = list_tools(&conn, &server.id, &server.name).await?;
    let resources = list_resources(&conn, &server.id).await?;

    // Persist to DB
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        replace_tools(&db, &server.id, &tools)?;
        replace_resources(&db, &server.id, &resources)?;
        update_discovery_status(&db, &server.id, "connected", None)?;
    }

    info!(
        server_id = %server.id,
        tools = tools.len(),
        resources = resources.len(),
        "MCP server discovered"
    );

    Ok(conn)
}

/// Connect and discover all enabled MCP servers at startup.
/// Errors are logged but never fatal.
pub async fn init_all_servers(app_handle: &tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;

    let state = app_handle.state::<crate::state::AppState>();

    let servers: Vec<McpServer> = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        list_servers(&db)?
            .into_iter()
            .filter(|s| s.enabled)
            .collect()
    };

    for server in servers {
        let server_id = server.id.clone();
        match discover_server(&server, app_handle).await {
            Ok(conn) => {
                let connections = state.mcp_connections.lock().await;
                // Store in state — we clone the Arc so the connection stays alive
                drop(connections); // drop before re-locking
                let mut connections = state.mcp_connections.lock().await;
                connections.insert(server_id, Arc::new(TokioMutex::new(conn)));
            }
            Err(e) => {
                warn!(server_id = %server_id, error = %e, "MCP server discovery failed at startup");
                // Mark as unreachable in DB (best-effort)
                if let Ok(db) = state.db.lock() {
                    let _ = update_discovery_status(&db, &server_id, "unreachable", Some(&e));
                }
            }
        }
    }

    Ok(())
}
