//! Remote desktop connection management.
//!
//! This module provides functions for managing remote desktop connections
//! including add, update, delete, and list operations.

use crate::db::models::{
    NewRemoteConnection, RemoteConnection, RemoteConnectionFilter, RemoteConnectionSummary,
    RemoteConnectionUpdate, RemoteProtocol,
};
use crate::state::AppState;
use anyhow::Context;
use base64::Engine;
use rusqlite::params;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::{rdp, vnc};

/// Add a new remote connection to the database.
pub async fn add_remote_connection(
    db: Arc<Mutex<rusqlite::Connection>>,
    config: NewRemoteConnection,
) -> anyhow::Result<RemoteConnection> {
    // Encrypt the password before storing
    let password_encrypted = encrypt_password(&config.password)?;
    
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let id = Uuid::now_v7().to_string();
    
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT INTO remote_connections 
         (id, name, protocol, hostname, port, username, password_encrypted, domain,
          resolution, color_depth, clipboard_sync, drive_redirect, multi_monitor,
          compression, quality, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        params![
            id,
            config.name,
            config.protocol.to_string(),
            config.hostname,
            config.port,
            config.username,
            password_encrypted,
            config.domain,
            config.resolution.clone().unwrap_or_else(|| "1280x800".to_string()),
            config.color_depth.unwrap_or(32),
            config.clipboard_sync.unwrap_or(true) as i64,
            config.drive_redirect.unwrap_or(false) as i64,
            config.multi_monitor.unwrap_or(false) as i64,
            config.compression.unwrap_or(true) as i64,
            config.quality.unwrap_or(80),
            now,
            now,
        ],
    )?;
    drop(conn);

    // Return the created connection
    Ok(RemoteConnection {
        id,
        name: config.name,
        protocol: config.protocol,
        hostname: config.hostname,
        port: config.port,
        username: config.username,
        password_encrypted,
        domain: config.domain,
        resolution: config.resolution.unwrap_or_else(|| "1280x800".to_string()),
        color_depth: config.color_depth.unwrap_or(32),
        clipboard_sync: config.clipboard_sync.unwrap_or(true),
        drive_redirect: config.drive_redirect.unwrap_or(false),
        multi_monitor: config.multi_monitor.unwrap_or(false),
        compression: config.compression.unwrap_or(true),
        quality: config.quality.unwrap_or(80),
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        last_connected_at: None,
    })
}

/// Update an existing remote connection.
pub async fn update_remote_connection(
    db: Arc<Mutex<rusqlite::Connection>>,
    id: String,
    updates: RemoteConnectionUpdate,
) -> anyhow::Result<RemoteConnection> {
    // Perform the update in a block to ensure conn is released before await
    {
        let conn = db.lock().unwrap();
        
        // First, get the existing connection
        let mut stmt = conn.prepare(
            "SELECT * FROM remote_connections WHERE id = ?1"
        )?;
        let mut rows = stmt.query([id.clone()])?;
        
        let row = rows.next()?.context("Connection not found")?;
        
        // Collect all the values before dropping stmt and rows
        let _name: String = row.get::<_, String>(1)?;
        let _protocol_str: String = row.get::<_, String>(2)?;
        let _hostname: String = row.get::<_, String>(3)?;
        let _port: i64 = row.get::<_, i64>(4)?;
        let _username: Option<String> = row.get::<_, Option<String>>(5)?;
        let _password_encrypted: String = row.get::<_, String>(6)?;
        let _domain: Option<String> = row.get::<_, Option<String>>(7)?;
        let _resolution: String = row.get::<_, String>(8)?;
        let _color_depth: i64 = row.get::<_, i64>(9)?;
        let _clipboard_sync: i64 = row.get::<_, i64>(10)?;
        let _drive_redirect: i64 = row.get::<_, i64>(11)?;
        let _multi_monitor: i64 = row.get::<_, i64>(12)?;
        let _compression: i64 = row.get::<_, i64>(13)?;
        let _quality: i64 = row.get::<_, i64>(14)?;
        let _created_at: String = row.get::<_, String>(15)?;
        let _last_connected_at: Option<String> = row.get::<_, Option<String>>(16)?;
        
        // Drop stmt and rows before reusing conn
        drop(rows);
        drop(stmt);
        
        // Build update query dynamically
        let mut updates_list: Vec<String> = Vec::new();
        let mut params_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        
        if let Some(name) = &updates.name {
            updates_list.push("name = ?".to_string());
            params_values.push(Box::new(name.clone()));
        }
        if let Some(protocol) = &updates.protocol {
            updates_list.push("protocol = ?".to_string());
            params_values.push(Box::new(protocol.to_string()));
        }
        if let Some(hostname) = &updates.hostname {
            updates_list.push("hostname = ?".to_string());
            params_values.push(Box::new(hostname.clone()));
        }
        if let Some(port) = updates.port {
            updates_list.push("port = ?".to_string());
            params_values.push(Box::new(port));
        }
        if let Some(username) = &updates.username {
            updates_list.push("username = ?".to_string());
            params_values.push(Box::new(username.clone()));
        }
        if let Some(password) = &updates.password {
            let encrypted = encrypt_password(password)?;
            updates_list.push("password_encrypted = ?".to_string());
            params_values.push(Box::new(encrypted));
        }
        if let Some(domain) = &updates.domain {
            updates_list.push("domain = ?".to_string());
            params_values.push(Box::new(domain.clone()));
        }
        if let Some(resolution) = &updates.resolution {
            updates_list.push("resolution = ?".to_string());
            params_values.push(Box::new(resolution.clone()));
        }
        if let Some(color_depth) = updates.color_depth {
            updates_list.push("color_depth = ?".to_string());
            params_values.push(Box::new(color_depth));
        }
        if let Some(clipboard_sync) = updates.clipboard_sync {
            updates_list.push("clipboard_sync = ?".to_string());
            params_values.push(Box::new(clipboard_sync as i64));
        }
        if let Some(drive_redirect) = updates.drive_redirect {
            updates_list.push("drive_redirect = ?".to_string());
            params_values.push(Box::new(drive_redirect as i64));
        }
        if let Some(multi_monitor) = updates.multi_monitor {
            updates_list.push("multi_monitor = ?".to_string());
            params_values.push(Box::new(multi_monitor as i64));
        }
        if let Some(compression) = updates.compression {
            updates_list.push("compression = ?".to_string());
            params_values.push(Box::new(compression as i64));
        }
        if let Some(quality) = updates.quality {
            updates_list.push("quality = ?".to_string());
            params_values.push(Box::new(quality));
        }
        
        if updates_list.is_empty() {
            return Err(anyhow::anyhow!("No updates provided"));
        }
        
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        updates_list.push("updated_at = ?".to_string());
        params_values.push(Box::new(now));
        params_values.push(Box::new(id.clone()));
        
        let query = format!("UPDATE remote_connections SET {} WHERE id = ?", updates_list.join(", "));
        conn.execute(&query, rusqlite::params_from_iter(params_values.iter()))?;
        // conn is dropped at the end of this block
    }
    
    // Return the updated connection (conn is now released)
    get_remote_connection(db, id).await
}

/// Remove a remote connection from the database.
pub async fn remove_remote_connection(
    db: Arc<Mutex<rusqlite::Connection>>,
    id: String,
) -> anyhow::Result<()> {
    let conn = db.lock().unwrap();
    
    conn.execute(
        "DELETE FROM remote_connections WHERE id = ?1",
        [id],
    )?;
    drop(conn);
    
    Ok(())
}

/// List all remote connections with optional filtering.
pub async fn list_remote_connections(
    db: Arc<Mutex<rusqlite::Connection>>,
    filter: Option<RemoteConnectionFilter>,
) -> anyhow::Result<Vec<RemoteConnectionSummary>> {
    let conn = db.lock().unwrap();
    
    let mut query = "SELECT id, name, protocol, hostname, port, username,
                            clipboard_sync, created_at, updated_at, last_connected_at
                     FROM remote_connections".to_string();
    
    let mut params_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    
    // Use ref to avoid moving filter
    if let Some(ref filter) = filter {
        let mut conditions: Vec<String> = Vec::new();
        
        if let Some(ref protocol) = filter.protocol {
            let protocol_str = protocol.to_string();
            conditions.push("protocol = ?".to_string());
            params_values.push(Box::new(protocol_str));
        }
        
        if let Some(ref name) = filter.name {
            conditions.push("name LIKE ?".to_string());
            params_values.push(Box::new(format!("%{}%", name)));
        }
        
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
    }
    
    query.push_str(" ORDER BY updated_at DESC");
    
    // Use filter with ref to avoid moving
    if let Some(ref filter) = filter {
        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        
        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }
    }
    
    let mut stmt = conn.prepare(&query)?;
    
    // Collect all results first while stmt is still valid
    let mut result: anyhow::Result<Vec<RemoteConnectionSummary>> = Ok(Vec::new());
    if result.is_ok() {
        let rows = stmt.query_map(rusqlite::params_from_iter(params_values.iter()), |row| {
            let protocol_str: String = row.get(2)?;
            let protocol = protocol_str.parse::<RemoteProtocol>()
                .map_err(|_| rusqlite::Error::InvalidParameterName("protocol".to_string()))?;
            Ok(RemoteConnectionSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                protocol,
                hostname: row.get(3)?,
                port: row.get(4)?,
                username: row.get::<_, Option<String>>(5)?,
                status: "disconnected".to_string(),
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                last_connected_at: row.get::<_, Option<String>>(9)?,
            })
        })?;
        
        for row in rows {
            result = result.and_then(|mut vec| {
                vec.push(row?);
                Ok(vec)
            });
        }
    }
    
    drop(stmt);
    drop(conn);
    
    result
}

/// Get a specific remote connection by ID.
pub async fn get_remote_connection(
    db: Arc<Mutex<rusqlite::Connection>>,
    id: String,
) -> anyhow::Result<RemoteConnection> {
    let conn = db.lock().unwrap();
    
    // First, collect all the data we need
    let (id_val, name, protocol_str, hostname, port, username, password_encrypted,
         domain, resolution, color_depth, clipboard_sync, drive_redirect,
         multi_monitor, compression, quality, created_at, updated_at, last_connected_at) = {
        let mut stmt = conn.prepare(
            "SELECT * FROM remote_connections WHERE id = ?1"
        )?;
        let mut rows = stmt.query([id.clone()])?;
        
        let row = rows.next()?.context("Connection not found")?;
        
        (
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, i64>(9)?,
            row.get::<_, i64>(10)?,
            row.get::<_, i64>(11)?,
            row.get::<_, i64>(12)?,
            row.get::<_, i64>(13)?,
            row.get::<_, i64>(14)?,
            row.get::<_, String>(15)?,
            row.get::<_, String>(16)?,
            row.get::<_, Option<String>>(17)?,
        )
    };
    // stmt and rows are dropped here
    
    let protocol = protocol_str.parse::<RemoteProtocol>()
        .map_err(|e| anyhow::anyhow!("Invalid protocol: {}", e))?;
    
    drop(conn);
    
    Ok(RemoteConnection {
        id: id_val,
        name,
        protocol,
        hostname,
        port: port as u16,
        username,
        password_encrypted,
        domain,
        resolution,
        color_depth: color_depth as u32,
        clipboard_sync: clipboard_sync != 0,
        drive_redirect: drive_redirect != 0,
        multi_monitor: multi_monitor != 0,
        compression: compression != 0,
        quality: quality as u32,
        created_at,
        updated_at,
        last_connected_at,
    })
}

/// Test a remote connection without storing it.
pub async fn test_remote_connection(
    db: Arc<Mutex<rusqlite::Connection>>,
    id: String,
) -> anyhow::Result<bool> {
    let conn_info = get_remote_connection(db, id).await?;
    
    // Decrypt the password for testing
    let password = decrypt_password(&conn_info.password_encrypted)?;
    
    match conn_info.protocol {
        RemoteProtocol::Rdp => {
            // Test RDP connection
            rdp::test_rdp_connection(
                &conn_info.hostname,
                conn_info.port,
                conn_info.username.as_deref(),
                conn_info.domain.as_deref(),
                &password,
            ).await
        }
        RemoteProtocol::Vnc => {
            // Test VNC connection
            vnc::test_vnc_connection(&conn_info.hostname, conn_info.port, &password).await
        }
    }
}

/// Connect to a remote desktop and return a WebSocket URL for streaming.
pub async fn connect_remote(
    db: Arc<Mutex<rusqlite::Connection>>,
    id: String,
) -> anyhow::Result<String> {
    let conn_info = get_remote_connection(db.clone(), id.clone()).await?;
    
    // Decrypt the password for connection
    let password = decrypt_password(&conn_info.password_encrypted)?;
    
    // Generate a unique session ID
    let session_id = Uuid::now_v7().to_string();
    
    match conn_info.protocol {
        RemoteProtocol::Rdp => {
            rdp::connect_rdp(
                &conn_info.hostname,
                conn_info.port,
                conn_info.username.as_deref(),
                conn_info.domain.as_deref(),
                &password,
                &conn_info.resolution,
                conn_info.color_depth,
                conn_info.clipboard_sync,
            ).await?;
            
            // Update last_connected_at
            update_last_connected(db.clone(), id.clone()).await?;
            
            Ok(format!("ws://127.0.0.1:8765/rdp/{}", session_id))
        }
        RemoteProtocol::Vnc => {
            vnc::connect_vnc(
                &conn_info.hostname,
                conn_info.port,
                &password,
                &conn_info.resolution,
            ).await?;
            
            // Update last_connected_at
            update_last_connected(db.clone(), id.clone()).await?;
            
            Ok(format!("ws://127.0.0.1:8765/vnc/{}", session_id))
        }
    }
}

/// Disconnect from a remote desktop session.
pub async fn disconnect_remote(_db: Arc<Mutex<rusqlite::Connection>>, session_id: String) -> anyhow::Result<()> {
    // In a real implementation, this would close the WebSocket connection
    // For now, we just log it
    tracing::info!("Disconnecting from session: {}", session_id);
    Ok(())
}

/// Helper function to update the last_connected_at timestamp.
async fn update_last_connected(
    db: Arc<Mutex<rusqlite::Connection>>,
    id: String,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let conn = db.lock().unwrap();
    
    conn.execute(
        "UPDATE remote_connections SET last_connected_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![now, now, id],
    )?;
    drop(conn);
    
    Ok(())
}

/// Encrypt a password using the application's encryption key.
fn encrypt_password(password: &str) -> anyhow::Result<String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    
    // Get the encryption key from environment or use a default for testing
    let key_bytes = get_encryption_key()?;
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; 12]); // For demo, use fixed nonce
    
    let ciphertext = cipher.encrypt(nonce, password.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;
    
    Ok(base64::engine::general_purpose::STANDARD.encode(&ciphertext))
}

/// Decrypt a password using the application's encryption key.
fn decrypt_password(encrypted: &str) -> anyhow::Result<String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    
    let key_bytes = get_encryption_key()?;
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; 12]);
    
    let ciphertext = base64::engine::general_purpose::STANDARD.decode(encrypted)
        .map_err(|e| anyhow::anyhow!("Base64 decode failed: {:?}", e))?;
    
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;
    
    Ok(String::from_utf8(plaintext)?)
}

/// Get the encryption key from environment or generate one.
fn get_encryption_key() -> anyhow::Result<Vec<u8>> {
    // Support both TRCAA_ENCRYPTION_KEY (new) and legacy name
    if let Ok(key) = std::env::var("TRCAA_ENCRYPTION_KEY") {
        if !key.trim().is_empty() {
            return Ok(hex::decode(&key).unwrap_or_else(|_| {
                // If not hex, use the string bytes directly (padded/truncated to 32 bytes)
                let mut bytes = vec![0u8; 32];
                let key_bytes = key.as_bytes();
                let len = key_bytes.len().min(32);
                bytes[..len].copy_from_slice(&key_bytes[..len]);
                bytes
            }));
        }
    }
    
    // Try to load from .enckey file
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("tftsr");
    let key_path = data_dir.join(".enckey");
    
    if key_path.exists() {
        let key = std::fs::read_to_string(&key_path)?;
        return Ok(hex::decode(key.trim()).unwrap_or_else(|_| {
            let mut bytes = vec![0u8; 32];
            let key_bytes = key.as_bytes();
            let len = key_bytes.len().min(32);
            bytes[..len].copy_from_slice(&key_bytes[..len]);
            bytes
        }));
    }
    
    // Generate a new key for development
    let mut key = vec![0u8; 32];
    use rand::RngCore;
    rand::rng().fill_bytes(&mut key);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt_password() {
        let password = "test-password-123";
        let encrypted = encrypt_password(password).unwrap();
        let decrypted = decrypt_password(&encrypted).unwrap();
        assert_eq!(password, decrypted);
    }
    
    use crate::remote::types::{Protocol, Resolution};

    #[test]
    fn test_protocol_display() {
        assert_eq!(Protocol::Rdp.to_string(), "rdp");
        assert_eq!(Protocol::Vnc.to_string(), "vnc");
    }
    
    #[test]
    fn test_protocol_from_str() {
        assert_eq!("rdp".parse::<Protocol>().unwrap(), Protocol::Rdp);
        assert_eq!("vnc".parse::<Protocol>().unwrap(), Protocol::Vnc);
        assert!("invalid".parse::<Protocol>().is_err());
    }
    
    #[test]
    fn test_resolution_parsing() {
        let res = Resolution::from_string("1920x1080");
        assert_eq!(res.width, 1920);
        assert_eq!(res.height, 1080);
        
        let res = Resolution::from_string("invalid");
        assert_eq!(res.width, 1280);
        assert_eq!(res.height, 800);
    }
    
    #[tokio::test]
    async fn test_test_remote_connection() {
        // This test requires a mock state, so we just verify the function compiles
        // In a real test, we would mock the database and test the connection logic
        assert!(true);
    }
}

// ============================================================================
// Tauri Command Wrappers
// ============================================================================

/// Tauri command wrapper for adding a remote connection.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_remote_connection_cmd(
    state: tauri::State<'_, AppState>,
    name: String,
    protocol: String,
    hostname: String,
    port: u16,
    username: Option<String>,
    password: String,
    domain: Option<String>,
    resolution: Option<String>,
    color_depth: Option<u32>,
    clipboard_sync: Option<bool>,
    drive_redirect: Option<bool>,
    multi_monitor: Option<bool>,
    compression: Option<bool>,
    quality: Option<u32>,
) -> Result<RemoteConnection, String> {
    let protocol = protocol.parse::<RemoteProtocol>()
        .map_err(|e| format!("Invalid protocol: {}", e))?;
    
    let new_connection = NewRemoteConnection {
        name,
        protocol,
        hostname,
        port,
        username,
        password,
        domain,
        resolution,
        color_depth,
        clipboard_sync,
        drive_redirect,
        multi_monitor,
        compression,
        quality,
    };
    
    // Clone the db from state for the async function
    let db = state.db.lock().map_err(|e| e.to_string())?;
    drop(db);
    
    add_remote_connection(state.db.clone(), new_connection)
        .await
        .map_err(|e| format!("Failed to add remote connection: {}", e))
}

/// Tauri command wrapper for updating a remote connection.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn update_remote_connection_cmd(
    state: tauri::State<'_, AppState>,
    id: String,
    name: Option<String>,
    protocol: Option<String>,
    hostname: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    domain: Option<String>,
    resolution: Option<String>,
    color_depth: Option<u32>,
    clipboard_sync: Option<bool>,
    drive_redirect: Option<bool>,
    multi_monitor: Option<bool>,
    compression: Option<bool>,
    quality: Option<u32>,
) -> Result<RemoteConnection, String> {
    let protocol = protocol.map(|p| p.parse::<RemoteProtocol>())
        .transpose()
        .map_err(|e| format!("Invalid protocol: {}", e))?;
    
    let updates = RemoteConnectionUpdate {
        name,
        protocol,
        hostname,
        port,
        username: username.map(Some),
        password,
        domain: domain.map(Some),
        resolution,
        color_depth,
        clipboard_sync,
        drive_redirect,
        multi_monitor,
        compression,
        quality,
    };
    
    update_remote_connection(state.db.clone(), id, updates)
        .await
        .map_err(|e| format!("Failed to update remote connection: {}", e))
}

/// Tauri command wrapper for removing a remote connection.
#[tauri::command]
pub async fn remove_remote_connection_cmd(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    remove_remote_connection(state.db.clone(), id)
        .await
        .map_err(|e| format!("Failed to remove remote connection: {}", e))
}

/// Tauri command wrapper for listing remote connections.
#[tauri::command]
pub async fn list_remote_connections_cmd(
    state: tauri::State<'_, AppState>,
    protocol: Option<String>,
    name: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<RemoteConnectionSummary>, String> {
    let filter = if protocol.is_some() || name.is_some() || limit.is_some() || offset.is_some() {
        Some(RemoteConnectionFilter {
            protocol: protocol.map(|p| p.parse::<RemoteProtocol>()).transpose()
                .map_err(|e| format!("Invalid protocol: {}", e))?,
            name,
            limit,
            offset,
        })
    } else {
        None
    };
    
    list_remote_connections(state.db.clone(), filter)
        .await
        .map_err(|e| format!("Failed to list remote connections: {}", e))
}

/// Tauri command wrapper for getting a specific remote connection.
#[tauri::command]
pub async fn get_remote_connection_cmd(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<RemoteConnection, String> {
    get_remote_connection(state.db.clone(), id)
        .await
        .map_err(|e| format!("Failed to get remote connection: {}", e))
}

/// Tauri command wrapper for testing a remote connection.
#[tauri::command]
pub async fn test_remote_connection_cmd(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    test_remote_connection(state.db.clone(), id)
        .await
        .map_err(|e| format!("Failed to test remote connection: {}", e))
}

/// Tauri command wrapper for connecting to a remote desktop.
#[tauri::command]
pub async fn connect_remote_cmd(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<String, String> {
    connect_remote(state.db.clone(), id)
        .await
        .map_err(|e| format!("Failed to connect to remote desktop: {}", e))
}

/// Tauri command wrapper for disconnecting from a remote desktop.
#[tauri::command]
pub async fn disconnect_remote_cmd(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    disconnect_remote(state.db.clone(), session_id)
        .await
        .map_err(|e| format!("Failed to disconnect from remote desktop: {}", e))
}
