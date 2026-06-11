use crate::proxmox::{ClusterInfo, ClusterType, ProxmoxClient};
use crate::state::AppState;
use chrono::Utc;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Proxmox cluster connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConnection {
    pub url: String,
    pub port: u16,
}

/// Add a Proxmox cluster
#[tauri::command]
pub async fn add_proxmox_cluster(
    id: String,
    name: String,
    cluster_type: ClusterType,
    connection: ClusterConnection,
    username: String,
    password: &str,
    state: State<'_, AppState>,
) -> Result<ClusterInfo, String> {
    // Create client and authenticate
    let client = ProxmoxClient::new(&connection.url, connection.port, &username);
    let ticket = client
        .authenticate(password)
        .await
        .map_err(|e| format!("Authentication failed: {}", e))?;

    // Encrypt credentials for storage
    let credentials = serde_json::json!({
        "ticket": ticket,
        "username": username
    });
    let encrypted_credentials = crate::integrations::auth::encrypt_token(
        &serde_json::to_string(&credentials).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("Failed to encrypt credentials: {}", e))?;

    // Create cluster info
    let cluster = ClusterInfo {
        id: id.clone(),
        name,
        cluster_type,
        url: connection.url,
        port: connection.port,
        username,
        created_at: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    // Store in database
    {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        db.execute(
            "INSERT INTO proxmox_clusters (id, name, cluster_type, url, port, auth_method, encrypted_credentials, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                cluster.id,
                cluster.name,
                match cluster.cluster_type {
                    ClusterType::VE => "ve",
                    ClusterType::PBS => "pbs",
                },
                cluster.url,
                cluster.port,
                "root",
                encrypted_credentials,
                cluster.created_at,
                cluster.updated_at,
            ],
        )
        .map_err(|e| format!("Failed to store cluster: {}", e))?;
    }

    // Store in memory for quick access
    {
        let mut clusters = state.proxmox_clusters.lock().await;
        clusters.insert(id, Arc::new(Mutex::new(client)));
    }

    Ok(cluster)
}

/// Remove a Proxmox cluster
#[tauri::command]
pub async fn remove_proxmox_cluster(id: String, state: State<'_, AppState>) -> Result<(), String> {
    // Remove from database
    {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        db.execute("DELETE FROM proxmox_clusters WHERE id = ?1", [id.clone()])
            .map_err(|e| format!("Failed to remove cluster: {}", e))?;
    }

    // Remove from memory
    {
        let mut clusters = state.clusters.lock().await;
        clusters.remove(&id);
    }

    Ok(())
}

/// List all Proxmox clusters
#[tauri::command]
pub async fn list_proxmox_clusters(state: State<'_, AppState>) -> Result<Vec<ClusterInfo>, String> {
    let clusters = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        let mut stmt = db
            .prepare(
                "SELECT id, name, cluster_type, url, port, created_at, updated_at FROM proxmox_clusters",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let cluster_iter = stmt
            .query_map([], |row| {
                Ok(ClusterInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    cluster_type: match row.get::<_, String>(2)?.as_str() {
                        "ve" => ClusterType::VE,
                        "pbs" => ClusterType::PBS,
                        _ => ClusterType::VE,
                    },
                    url: row.get(3)?,
                    port: row.get(4)?,
                    username: "".to_string(), // Will be decrypted when needed
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to query clusters: {}", e))?;

        cluster_iter
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    };

    clusters
}

/// Get a specific Proxmox cluster
#[tauri::command]
pub async fn get_proxmox_cluster(
    id: String,
    state: State<'_, AppState>,
) -> Result<Option<ClusterInfo>, String> {
    let cluster = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        let mut stmt = db
            .prepare(
                "SELECT id, name, cluster_type, url, port, created_at, updated_at FROM proxmox_clusters WHERE id = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        stmt.query_row([id], |row| {
            Ok(ClusterInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                cluster_type: match row.get::<_, String>(2)?.as_str() {
                    "ve" => ClusterType::VE,
                    "pbs" => ClusterType::PBS,
                    _ => ClusterType::VE,
                },
                url: row.get(3)?,
                port: row.get(4)?,
                username: "".to_string(),
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .optional()
        .map_err(|e| format!("Failed to query cluster: {}", e))?
    };

    Ok(cluster)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_type_serialization() {
        let json = serde_json::to_string(&ClusterType::VE).unwrap();
        assert_eq!(json, "\"ve\"");

        let ve: ClusterType = serde_json::from_str("\"ve\"").unwrap();
        assert_eq!(ve, ClusterType::VE);

        let pbs: ClusterType = serde_json::from_str("\"pbs\"").unwrap();
        assert_eq!(pbs, ClusterType::PBS);
    }

    #[test]
    fn test_cluster_info_serialization() {
        let cluster = ClusterInfo {
            id: "proxmox-1".to_string(),
            name: "Production".to_string(),
            cluster_type: ClusterType::VE,
            url: "https://pve.example.com".to_string(),
            port: 8006,
            username: "root@pam".to_string(),
            created_at: "2026-06-10 12:00:00".to_string(),
            updated_at: "2026-06-10 12:00:00".to_string(),
        };

        let json = serde_json::to_string(&cluster).unwrap();
        let deserialized: ClusterInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(cluster.id, deserialized.id);
        assert_eq!(cluster.name, deserialized.name);
    }
}
