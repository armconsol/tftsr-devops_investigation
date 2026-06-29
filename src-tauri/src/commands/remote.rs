//! Remote Desktop Connection Commands
//!
//! Tauri IPC handlers for remote desktop functionality including RDP, VNC, and SSH tunneling.

use tauri::State;
use tracing::{info, warn};

use crate::db::models::{
    NewRemoteConnection, RemoteConnectionFilter, RemoteConnectionSummary, RemoteConnectionUpdate,
};
use crate::remote::connection::{
    create_remote_connection as db_create, delete_remote_connection as db_delete,
    get_remote_connection as db_get, get_remote_connection_full, get_remote_rdp_password,
    get_remote_ssh_credentials, list_remote_connections as db_list,
    update_remote_connection as db_update,
};
use crate::remote::rdp::RdpSession;
use crate::state::AppState;

/// List all remote connections
#[tauri::command]
pub fn list_remote_connections(
    app_state: State<'_, AppState>,
    filter: Option<String>,
) -> Result<Vec<RemoteConnectionSummary>, String> {
    let conn = app_state.db.lock().unwrap();

    let filter_struct = RemoteConnectionFilter {
        protocol: None,
        name: filter,
        limit: None,
        offset: None,
    };

    db_list(&conn, &filter_struct).map_err(|e| {
        warn!("Failed to list remote connections: {}", e);
        e.to_string()
    })
}

/// Get a specific remote connection by ID
#[tauri::command]
pub fn get_remote_connection(
    app_state: State<'_, AppState>,
    id: String,
) -> Result<RemoteConnectionSummary, String> {
    let conn = app_state.db.lock().unwrap();

    db_get(&conn, &id).map_err(|e| {
        warn!("Failed to get remote connection {}: {}", id, e);
        e.to_string()
    })
}

/// Create a new remote connection
#[tauri::command]
pub fn create_remote_connection(
    app_state: State<'_, AppState>,
    new_conn: NewRemoteConnection,
) -> Result<RemoteConnectionSummary, String> {
    let conn = app_state.db.lock().unwrap();

    let connection = db_create(&conn, &new_conn).map_err(|e| {
        warn!("Failed to create remote connection: {}", e);
        e.to_string()
    })?;

    // Convert to summary
    Ok(RemoteConnectionSummary {
        id: connection.id,
        name: connection.name,
        protocol: connection.protocol,
        hostname: connection.hostname,
        port: connection.port,
        username: connection.username,
        status: "active".to_string(),
        ssh_enabled: connection.ssh_enabled,
        created_at: connection.created_at,
        updated_at: connection.updated_at,
        last_connected_at: connection.last_connected_at,
    })
}

/// Update an existing remote connection
#[tauri::command]
pub fn update_remote_connection(
    app_state: State<'_, AppState>,
    id: String,
    update: RemoteConnectionUpdate,
) -> Result<RemoteConnectionSummary, String> {
    let conn = app_state.db.lock().unwrap();

    db_update(&conn, &id, &update).map_err(|e| {
        warn!("Failed to update remote connection {}: {}", id, e);
        e.to_string()
    })
}

/// Delete a remote connection
#[tauri::command]
pub fn delete_remote_connection(app_state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = app_state.db.lock().unwrap();

    db_delete(&conn, &id).map_err(|e| {
        warn!("Failed to delete remote connection {}: {}", id, e);
        e.to_string()
    })
}

/// Start an RDP session.
///
/// `password` is optional: when omitted or blank, the stored (encrypted) RDP
/// password for the connection is used. This avoids re-prompting the user when
/// they already saved a password on the connection entry.
#[tauri::command]
pub async fn start_rdp_session(
    app_state: State<'_, AppState>,
    connection_id: String,
    password: Option<String>,
) -> Result<RdpSession, String> {
    info!("Starting RDP session for connection: {}", connection_id);

    // Get the full connection details, SSH credentials, and stored RDP password.
    let (connection, ssh_password, ssh_private_key, ssh_key_passphrase, stored_rdp_password) = {
        let conn = app_state.db.lock().unwrap();
        let connection = get_remote_connection_full(&conn, &connection_id)
            .map_err(|e| format!("Failed to get connection: {}", e))?;
        let (ssh_pw, ssh_key, ssh_pass) = get_remote_ssh_credentials(&conn, &connection_id)
            .map_err(|e| format!("Failed to get SSH credentials: {}", e))?;
        let stored_rdp_pw = get_remote_rdp_password(&conn, &connection_id)
            .map_err(|e| format!("Failed to get RDP credentials: {}", e))?;
        (connection, ssh_pw, ssh_key, ssh_pass, stored_rdp_pw)
    };

    // Prefer an explicitly supplied password; otherwise use the stored one.
    let effective_password = match password {
        Some(p) if !p.trim().is_empty() => p,
        _ => stored_rdp_password.ok_or_else(|| {
            "No password provided and no saved password for this connection".to_string()
        })?,
    };

    // Clone the manager so we can hold it across the async connect call without
    // keeping the std Mutex guard locked over an await point.
    let rdp_manager = {
        let guard = app_state.rdp_manager.lock().unwrap();
        (*guard).clone()
    };

    // Create RDP session
    let internal_session = rdp_manager
        .create_session(
            &connection,
            &effective_password,
            ssh_password,
            ssh_private_key,
            ssh_key_passphrase,
        )
        .map_err(|e| format!("Failed to create RDP session: {}", e))?;

    let session_id = internal_session.id.clone();
    let ws_url = rdp_manager
        .start_session_with_password(&session_id, &effective_password)
        .await
        .map_err(|e| format!("Failed to start RDP session: {}", e))?;

    // Return public session info
    let session = RdpSession {
        id: session_id.clone(),
        connection_id: internal_session.connection_id,
        hostname: internal_session.hostname,
        port: internal_session.port,
        username: internal_session.username,
        resolution: internal_session.resolution,
        color_depth: internal_session.color_depth,
        websocket_port: internal_session.websocket_port,
        websocket_url: ws_url,
        connected: true,
        ssh_enabled: internal_session.ssh_config.is_some(),
    };

    info!("RDP session started successfully: {}", session.id);
    Ok(session)
}

/// Stop an RDP session
#[tauri::command]
pub fn stop_rdp_session(app_state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    info!("Stopping RDP session: {}", session_id);

    let rdp_manager = app_state.rdp_manager.lock().unwrap();

    rdp_manager
        .stop_session(&session_id)
        .map_err(|e| format!("Failed to stop RDP session: {}", e))?;

    info!("RDP session stopped: {}", session_id);
    Ok(())
}

/// Get RDP session details
#[tauri::command]
pub fn get_rdp_session(
    app_state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<RdpSession>, String> {
    let rdp_manager = app_state.rdp_manager.lock().unwrap();
    let session = rdp_manager.get_session(&session_id);
    Ok(session)
}

/// Resize a running RDP session's display (Display Control Virtual Channel).
#[tauri::command]
pub async fn resize_rdp_session(
    app_state: State<'_, AppState>,
    session_id: String,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let rdp_manager = {
        let guard = app_state.rdp_manager.lock().unwrap();
        (*guard).clone()
    };
    rdp_manager
        .resize_session(&session_id, width, height)
        .await
        .map_err(|e| e.to_string())
}
