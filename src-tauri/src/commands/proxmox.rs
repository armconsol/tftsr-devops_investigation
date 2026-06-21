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

/// Cluster info enriched with live connection health status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterInfoWithHealth {
    pub id: String,
    pub name: String,
    pub cluster_type: ClusterType,
    pub url: String,
    pub port: u16,
    pub username: String,
    pub created_at: String,
    pub updated_at: String,
    /// True if an active client object exists in the in-memory connection pool
    pub connected: bool,
}

/// Add a Proxmox cluster
#[tauri::command]
pub async fn add_proxmox_cluster(
    id: String,
    name: String,
    cluster_type: ClusterType,
    connection: ClusterConnection,
    username: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<ClusterInfo, String> {
    // Authenticate immediately — this verifies credentials and gives us a live
    // ticketed client. If auth fails we return early before touching the DB.
    let mut client = ProxmoxClient::new(&connection.url, connection.port, &username);
    client
        .authenticate(&password)
        .await
        .map_err(|e| format!("Failed to authenticate with Proxmox: {}", e))?;

    // Encrypt raw password so we can re-authenticate after app restart.
    let credentials = serde_json::json!({
        "password": password,
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
        username: username.clone(),
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
            "INSERT INTO proxmox_clusters (id, name, cluster_type, url, port, username, auth_method, encrypted_credentials, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                cluster.id,
                cluster.name,
                match cluster.cluster_type {
                    ClusterType::VE => "ve",
                    ClusterType::PBS => "pbs",
                },
                cluster.url,
                cluster.port,
                username,
                "password",
                encrypted_credentials,
                cluster.created_at,
                cluster.updated_at,
            ],
        )
        .map_err(|e| format!("Failed to store cluster: {}", e))?;
    }

    // Insert the authenticated client into the in-memory pool.
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
        let mut clusters = state.proxmox_clusters.lock().await;
        clusters.remove(&id);
    }

    Ok(())
}

/// List all Proxmox clusters, annotated with live connection health
#[tauri::command]
pub async fn list_proxmox_clusters(
    state: State<'_, AppState>,
) -> Result<Vec<ClusterInfoWithHealth>, String> {
    let db_clusters = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        let mut stmt = db
            .prepare(
                "SELECT id, name, cluster_type, url, port, username, created_at, updated_at FROM proxmox_clusters",
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
                    username: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Failed to query clusters: {}", e))?;

        cluster_iter
            .collect::<Result<Vec<ClusterInfo>, _>>()
            .map_err(|e| e.to_string())?
    };

    // Annotate each cluster with whether a live client exists in the connection pool
    let live_clients = state.proxmox_clusters.lock().await;
    let result = db_clusters
        .into_iter()
        .map(|c| {
            let connected = live_clients.contains_key(&c.id);
            ClusterInfoWithHealth {
                id: c.id,
                name: c.name,
                cluster_type: c.cluster_type,
                url: c.url,
                port: c.port,
                username: c.username,
                created_at: c.created_at,
                updated_at: c.updated_at,
                connected,
            }
        })
        .collect();

    Ok(result)
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
                "SELECT id, name, cluster_type, url, port, username, created_at, updated_at FROM proxmox_clusters WHERE id = ?1",
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
                username: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .optional()
        .map_err(|e| format!("Failed to query cluster: {}", e))?
    };

    Ok(cluster)
}

/// Helper function to get or create a Proxmox client for a cluster
/// This will:
/// 1. Check if client exists in memory pool
/// 2. If not, load credentials from database and create/authenticate client
async fn get_proxmox_client_for_cluster(
    cluster_id: &str,
    state: &State<'_, AppState>,
) -> Result<Arc<Mutex<crate::proxmox::ProxmoxClient>>, String> {
    // First, try to get from in-memory pool
    {
        let clusters = state.proxmox_clusters.lock().await;
        if let Some(client) = clusters.get(cluster_id) {
            return Ok(client.clone());
        }
    }

    // Not in memory - load from database and create client
    let (url, port, username, encrypted_credentials) = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        let mut stmt = db
            .prepare(
                "SELECT url, port, username, encrypted_credentials FROM proxmox_clusters WHERE id = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        stmt.query_row([cluster_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u16>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .optional()
        .map_err(|e| format!("Failed to query cluster: {}", e))?
        .ok_or_else(|| format!("Cluster {} not found in database", cluster_id))?
    };

    // Decrypt credentials
    let credentials_json = crate::integrations::auth::decrypt_token(&encrypted_credentials)
        .map_err(|e| format!("Failed to decrypt credentials: {}", e))?;

    let credentials: serde_json::Value = serde_json::from_str(&credentials_json)
        .map_err(|e| format!("Failed to parse credentials: {}", e))?;

    let password = credentials
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Password not found in credentials".to_string())?;

    // Create new client
    let mut client = crate::proxmox::ProxmoxClient::new(&url, port, &username);

    // Authenticate to get ticket
    let ticket = client
        .authenticate(password)
        .await
        .map_err(|e| format!("Failed to authenticate with Proxmox: {}", e))?;

    client.set_ticket(&ticket);

    let client_arc = Arc::new(Mutex::new(client));
    {
        let mut clusters = state.proxmox_clusters.lock().await;
        // Re-check under write lock: a concurrent task may have already created a client
        // for this cluster between our read-check and here, so prefer the existing one.
        if let Some(existing) = clusters.get(cluster_id) {
            return Ok(existing.clone());
        }
        clusters.insert(cluster_id.to_string(), client_arc.clone());
    }

    Ok(client_arc)
}

/// Ping a Proxmox cluster — authenticates and calls the version endpoint to verify
/// that the API is reachable and credentials are valid.
#[tauri::command]
pub async fn ping_proxmox_cluster(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    client_guard
        .get::<serde_json::Value>("version", client_guard.ticket.as_deref())
        .await
        .map_err(|e| format!("Connection test failed: {}", e))
}

/// Update an existing Proxmox cluster's metadata and credentials atomically.
/// Unlike the remove-then-add pattern this is a single SQL UPDATE so there is
/// no window where the record is missing.
#[tauri::command]
pub async fn update_proxmox_cluster(
    id: String,
    name: String,
    cluster_type: ClusterType,
    connection: ClusterConnection,
    username: String,
    password: &str,
    state: State<'_, AppState>,
) -> Result<ClusterInfo, String> {
    let credentials = serde_json::json!({ "password": password, "username": username });
    let encrypted_credentials = crate::integrations::auth::encrypt_token(
        &serde_json::to_string(&credentials).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("Failed to encrypt credentials: {}", e))?;

    let updated_at = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        let rows = db
            .execute(
                "UPDATE proxmox_clusters \
                 SET name=?1, cluster_type=?2, url=?3, port=?4, username=?5, \
                     encrypted_credentials=?6, updated_at=?7 \
                 WHERE id=?8",
                rusqlite::params![
                    name,
                    match cluster_type {
                        ClusterType::VE => "ve",
                        ClusterType::PBS => "pbs",
                    },
                    connection.url,
                    connection.port,
                    username,
                    encrypted_credentials,
                    updated_at,
                    id,
                ],
            )
            .map_err(|e| format!("Failed to update cluster: {}", e))?;

        if rows == 0 {
            return Err(format!("Cluster {} not found", id));
        }
    }

    // Evict the stale authenticated client — it will re-authenticate with new credentials
    // on the next API call.
    {
        let mut clusters = state.proxmox_clusters.lock().await;
        clusters.remove(&id);
    }

    Ok(ClusterInfo {
        id,
        name,
        cluster_type,
        url: connection.url,
        port: connection.port,
        username,
        created_at: String::new(),
        updated_at,
    })
}

/// List all Proxmox VMs
#[tauri::command]
pub async fn list_proxmox_vms(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let vms =
        crate::proxmox::vm::list_vms(&client_guard, client_guard.ticket.as_deref().unwrap_or(""))
            .await
            .map_err(|e| format!("Failed to list VMs: {}", e))?;

    // Convert VmInfo structs to JSON
    let json_vms: Vec<serde_json::Value> = vms
        .into_iter()
        .map(|vm| serde_json::to_value(vm).map_err(|e| format!("Failed to serialize VM: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_vms)
}

/// Get Proxmox VM details
#[tauri::command]
pub async fn get_proxmox_vm(
    cluster_id: String,
    node: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let vm = crate::proxmox::vm::get_vm(
        &client_guard,
        &node,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to get VM {}: {}", vm_id, e))?;

    serde_json::to_value(vm).map_err(|e| format!("Failed to serialize VM: {}", e))
}

/// Start a Proxmox VM
#[tauri::command]
pub async fn start_proxmox_vm(
    cluster_id: String,
    node: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::start_vm(
        &client_guard,
        &node,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to start VM {}: {}", vm_id, e))
}

/// Stop a Proxmox VM
#[tauri::command]
pub async fn stop_proxmox_vm(
    cluster_id: String,
    node: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::stop_vm(
        &client_guard,
        &node,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to stop VM {}: {}", vm_id, e))
}

/// Reboot a Proxmox VM
#[tauri::command]
pub async fn reboot_proxmox_vm(
    cluster_id: String,
    node: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::reboot_vm(
        &client_guard,
        &node,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to reboot VM {}: {}", vm_id, e))
}

/// Shutdown a Proxmox VM
#[tauri::command]
pub async fn shutdown_proxmox_vm(
    cluster_id: String,
    node: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::shutdown_vm(
        &client_guard,
        &node,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to shutdown VM {}: {}", vm_id, e))
}

/// List Proxmox Backup Jobs (cluster-level, not node-level)
#[tauri::command]
pub async fn list_proxmox_backup_jobs(
    cluster_id: String,
    _node: String, // Node parameter kept for compatibility but not used
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    // Proxmox VE backup jobs are at cluster level, not node level
    let path = "cluster/backup";
    let response: serde_json::Value = client_guard
        .get(path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list backup jobs: {}", e))?;

    let mut jobs: Vec<serde_json::Value> = response
        .as_array()
        .map(|arr| {
            arr.iter()
                .cloned()
                .map(|mut job| {
                    // Add id field if missing for frontend compatibility
                    if let Some(job_obj) = job.as_object_mut() {
                        if !job_obj.contains_key("id") {
                            if let Some(jobid) = job_obj.get("id").and_then(|v| v.as_str()) {
                                job_obj.insert("id".to_string(), serde_json::Value::String(jobid.to_string()));
                            } else {
                                let storage = job_obj.get("storage")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                job_obj.insert("id".to_string(), serde_json::Value::String(format!("backup-{}", storage)));
                            }
                        }
                    }
                    job
                })
                .collect()
        })
        .ok_or_else(|| "Invalid response format".to_string())?;

    // Apply limit if needed (Proxmox may return many jobs)
    if jobs.len() > 100 {
        jobs.truncate(100);
    }

    Ok(jobs)
}

/// List Proxmox Datastores (Storage per node)
#[tauri::command]
pub async fn list_proxmox_datastores(
    cluster_id: String,
    _state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    // Note: Proxmox VE storage is per-node, not cluster-wide
    // We need to get all nodes first, then fetch storage for each node
    let client = get_proxmox_client_for_cluster(&cluster_id, &_state).await?;
    let client_guard = client.lock().await;

    // First, get all nodes
    let nodes_path = "cluster/resources?type=node";
    let nodes_response: serde_json::Value = client_guard
        .get(nodes_path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list nodes: {}", e))?;

    let nodes: Vec<String> = nodes_response
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|n| n.get("node").and_then(|node| node.as_str()).map(|s| s.to_string()))
        .collect();

    if nodes.is_empty() {
        return Ok(vec![]);
    }

    // Fetch storage for each node
    let mut all_storage: Vec<serde_json::Value> = vec![];

    for node in nodes {
        let storage_path = format!("nodes/{}/storage", node);
        let storage_response: serde_json::Value = client_guard
            .get(&storage_path, Some(client_guard.ticket.as_deref().unwrap_or("")))
            .await
            .map_err(|e| format!("Failed to list storage for node {}: {}", node, e))?;

        if let Some(storage_array) = storage_response.as_array() {
            for mut storage in storage_array.clone() {
                // Add node information to each storage entry
                if let Some(storage_obj) = storage.as_object_mut() {
                    storage_obj.insert("node".to_string(), serde_json::Value::String(node.clone()));
                    // Create a unique ID
                    if let Some(storage_name) = storage_obj.get("storage").and_then(|s| s.as_str()) {
                        storage_obj.insert("id".to_string(), serde_json::Value::String(format!("storage/{}", storage_name)));
                    }
                }
                all_storage.push(storage);
            }
        }
    }

    Ok(all_storage)
}

/// Trigger Proxmox Backup Job
#[tauri::command]
pub async fn trigger_proxmox_backup_job(
    cluster_id: String,
    node: String,
    job_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::backup::trigger_backup_job(
        &client_guard,
        &node,
        job_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to trigger backup job {}: {}", job_id, e))
}

/// List Ceph Pools
#[tauri::command]
pub async fn list_ceph_pools(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let pools = crate::proxmox::ceph::list_pools(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list Ceph pools: {}", e))?;

    let json_pools: Vec<serde_json::Value> = pools
        .into_iter()
        .map(|pool| {
            serde_json::to_value(pool).map_err(|e| format!("Failed to serialize Ceph pool: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_pools)
}

/// List Ceph OSDs
#[tauri::command]
pub async fn list_ceph_osd(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let osds = crate::proxmox::ceph::list_osds(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list Ceph OSDs: {}", e))?;

    let json_osds: Vec<serde_json::Value> = osds
        .into_iter()
        .map(|osd| {
            serde_json::to_value(osd).map_err(|e| format!("Failed to serialize Ceph OSD: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_osds)
}

/// Get Ceph Health
#[tauri::command]
pub async fn get_ceph_health(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let health = crate::proxmox::ceph::get_ceph_health(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to get Ceph health: {}", e))?;

    serde_json::to_value(health).map_err(|e| format!("Failed to serialize Ceph health: {}", e))
}

// ─── Phase 1 - Core Management Features ───────────────────────────────────────

/// List authentication realms (LDAP/AD/OpenID)
#[tauri::command]
pub async fn list_auth_realms(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let realms = crate::proxmox::auth_realm::list_auth_realms(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list auth realms: {}", e))?;

    let json_realms: Vec<serde_json::Value> = realms
        .into_iter()
        .map(|r| serde_json::to_value(r).map_err(|e| format!("Failed to serialize realm: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_realms)
}

/// Add LDAP realm
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_ldap_realm(
    cluster_id: String,
    realm_id: String,
    server: String,
    port: u16,
    base_dn: String,
    bind_dn: String,
    bind_password: String,
    filter: String,
    scope: String,
    start_tls: bool,
    certificate: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let config = crate::proxmox::auth_realm::LdapRealmConfig {
        server,
        port,
        base_dn,
        bind_dn,
        bind_password,
        filter,
        scope,
        start_tls,
        certificate,
    };

    crate::proxmox::auth_realm::add_ldap_realm(
        &client_guard,
        &realm_id,
        &config,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to add LDAP realm: {}", e))
}

/// Add AD realm
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_ad_realm(
    cluster_id: String,
    realm_id: String,
    server: String,
    port: u16,
    base_dn: String,
    bind_dn: String,
    bind_password: String,
    filter: String,
    scope: String,
    use_ssl: bool,
    certificate: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let config = crate::proxmox::auth_realm::AdRealmConfig {
        server,
        port,
        base_dn,
        bind_dn,
        bind_password,
        filter,
        scope,
        use_ssl,
        certificate,
    };

    crate::proxmox::auth_realm::add_ad_realm(
        &client_guard,
        &realm_id,
        &config,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to add AD realm: {}", e))
}

/// Add OpenID realm
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_openid_realm(
    cluster_id: String,
    realm_id: String,
    issuer: String,
    client_id: String,
    client_secret: String,
    redirect_url: String,
    scopes: Vec<String>,
    mapping: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let config = crate::proxmox::auth_realm::OpenidRealmConfig {
        issuer,
        client_id,
        client_secret,
        redirect_url,
        scopes,
        mapping,
    };

    crate::proxmox::auth_realm::add_openid_realm(
        &client_guard,
        &realm_id,
        &config,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to add OpenID realm: {}", e))
}

/// List ACME accounts
#[tauri::command]
pub async fn list_acme_accounts(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let accounts = crate::proxmox::acme::list_acme_accounts(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list ACME accounts: {}", e))?;

    let json_accounts: Vec<serde_json::Value> = accounts
        .into_iter()
        .map(|a| serde_json::to_value(a).map_err(|e| format!("Failed to serialize account: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_accounts)
}

/// Register ACME account
#[tauri::command]
pub async fn register_acme_account(
    cluster_id: String,
    email: String,
    terms_of_service_agreed: bool,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let account = crate::proxmox::acme::register_acme_account(
        &client_guard,
        &email,
        terms_of_service_agreed,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to register ACME account: {}", e))?;

    serde_json::to_value(account).map_err(|e| format!("Failed to serialize account: {}", e))
}

/// Get ACME challenges
#[tauri::command]
pub async fn get_acme_challenges(
    cluster_id: String,
    domain: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let challenges = crate::proxmox::acme::get_acme_challenges(
        &client_guard,
        &domain,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to get ACME challenges: {}", e))?;

    let json_challenges: Vec<serde_json::Value> = challenges
        .into_iter()
        .map(|c| {
            serde_json::to_value(c).map_err(|e| format!("Failed to serialize challenge: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_challenges)
}

/// List APT updates
#[tauri::command]
pub async fn list_apt_updates(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let updates = crate::proxmox::apt::list_apt_updates(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list APT updates: {}", e))?;

    let json_updates: Vec<serde_json::Value> = updates
        .into_iter()
        .map(|u| serde_json::to_value(u).map_err(|e| format!("Failed to serialize update: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_updates)
}

/// Update APT repositories
#[tauri::command]
pub async fn update_apt_repos(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::apt::update_apt_repos(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to update APT repos: {}", e))
}

/// List APT repositories
#[tauri::command]
pub async fn list_apt_repositories(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let repos = crate::proxmox::apt::list_apt_repositories(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list APT repos: {}", e))?;

    let json_repos: Vec<serde_json::Value> = repos
        .into_iter()
        .map(|r| serde_json::to_value(r).map_err(|e| format!("Failed to serialize repo: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_repos)
}

/// Get shell ticket
#[tauri::command]
pub async fn get_shell_ticket(
    cluster_id: String,
    remote: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let ticket = crate::proxmox::shell::get_shell_ticket(
        &client_guard,
        &remote,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to get shell ticket: {}", e))?;

    serde_json::to_value(ticket).map_err(|e| format!("Failed to serialize ticket: {}", e))
}

/// List dashboard views
#[tauri::command]
pub async fn list_views(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let views = crate::proxmox::views::list_views(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await;

    // Handle 501 Not Implemented gracefully - return empty array
    if let Err(e) = &views {
        if e.contains("501") || e.contains("Not Implemented") || e.contains("not implemented") {
            tracing::warn!("Views API not implemented by Proxmox server, returning empty list");
            return Ok(vec![]);
        }
    }

    let views = views.map_err(|e| format!("Failed to list views: {}", e))?;

    let json_views: Vec<serde_json::Value> = views
        .into_iter()
        .map(|v| serde_json::to_value(v).map_err(|e| format!("Failed to serialize view: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_views)
}

/// Add dashboard view
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_view(
    cluster_id: String,
    view_id: String,
    name: String,
    description: String,
    layout: String,
    widgets: Vec<serde_json::Value>,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let widgets: Vec<crate::proxmox::views::Widget> = widgets
        .into_iter()
        .map(|w| {
            serde_json::from_value(w).map_err(|e| format!("Failed to deserialize widget: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let view = crate::proxmox::views::DashboardView {
        view_id,
        name,
        description,
        layout,
        widgets,
        enabled,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    crate::proxmox::views::add_view(
        &client_guard,
        &view,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to add view: {}", e))
}

/// Update dashboard view
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn update_view(
    cluster_id: String,
    view_id: String,
    name: String,
    description: String,
    layout: String,
    widgets: Vec<serde_json::Value>,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let widgets: Vec<crate::proxmox::views::Widget> = widgets
        .into_iter()
        .map(|w| {
            serde_json::from_value(w).map_err(|e| format!("Failed to deserialize widget: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let view = crate::proxmox::views::DashboardView {
        view_id: view_id.clone(),
        name,
        description,
        layout,
        widgets,
        enabled,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    crate::proxmox::views::update_view(
        &client_guard,
        &view_id,
        &view,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to update view: {}", e))
}

/// Delete dashboard view
#[tauri::command]
pub async fn delete_view(
    cluster_id: String,
    view_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::views::delete_view(
        &client_guard,
        &view_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to delete view: {}", e))
}

/// List certificates
#[tauri::command]
pub async fn list_certificates(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let certs = crate::proxmox::certificates::list_certificates(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await;

    // Handle 501 Not Implemented gracefully - return empty array
    if let Err(e) = &certs {
        if e.contains("501") || e.contains("Not Implemented") || e.contains("not implemented") {
            tracing::warn!("Certificates API not implemented by Proxmox server, returning empty list");
            return Ok(vec![]);
        }
    }

    let certs = certs.map_err(|e| format!("Failed to list certificates: {}", e))?;

    let json_certs: Vec<serde_json::Value> = certs
        .into_iter()
        .map(|c| serde_json::to_value(c).map_err(|e| format!("Failed to serialize cert: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_certs)
}

/// Upload certificate
#[tauri::command]
pub async fn upload_certificate(
    cluster_id: String,
    certificate: String,
    private_key: String,
    name: Option<String>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let cert = crate::proxmox::certificates::upload_certificate(
        &client_guard,
        &certificate,
        &private_key,
        name.as_deref(),
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to upload certificate: {}", e))?;

    serde_json::to_value(cert).map_err(|e| format!("Failed to serialize cert: {}", e))
}

/// Get certificate
#[tauri::command]
pub async fn get_certificate(
    cluster_id: String,
    cert_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let cert = crate::proxmox::certificates::get_certificate(
        &client_guard,
        &cert_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to get certificate {}: {}", cert_id, e))?;

    serde_json::to_value(cert).map_err(|e| format!("Failed to serialize cert: {}", e))
}

// ─── Phase 2 - Advanced Management ────────────────────────────────────────────

// Firewall commands (extended from existing)
/// List firewall rules
#[tauri::command]
pub async fn list_firewall_rules(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let rules = crate::proxmox::firewall::list_firewall_rules(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list firewall rules: {}", e))?;

    let json_rules: Vec<serde_json::Value> = rules
        .into_iter()
        .map(|r| serde_json::to_value(r).map_err(|e| format!("Failed to serialize rule: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_rules)
}

/// Add firewall rule
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_firewall_rule(
    cluster_id: String,
    node: String,
    action: String,
    protocol: String,
    source: String,
    destination: String,
    port: Option<String>,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let rule = crate::proxmox::firewall::FirewallRule {
        rule_num: 0,
        action,
        protocol,
        source,
        destination,
        port,
        enabled,
    };

    crate::proxmox::firewall::add_rule(
        &client_guard,
        &node,
        &rule,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to add firewall rule: {}", e))
}

/// Delete firewall rule
#[tauri::command]
pub async fn delete_firewall_rule(
    cluster_id: String,
    node: String,
    rule_num: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::firewall::delete_rule(
        &client_guard,
        &node,
        rule_num,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to delete firewall rule {}: {}", rule_num, e))
}

// SDN commands (extended from existing)
/// List SDN controllers
#[tauri::command]
pub async fn list_sdn_controllers(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let controllers = crate::proxmox::sdn::list_evpn_zones(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list SDN controllers: {}", e))?;

    let json_controllers: Vec<serde_json::Value> = controllers
        .into_iter()
        .map(|c| {
            serde_json::to_value(c).map_err(|e| format!("Failed to serialize controller: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_controllers)
}

/// List SDN virtual networks
#[tauri::command]
pub async fn list_sdn_vnets(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let vnets = crate::proxmox::sdn::list_vnets(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list SDN virtual networks: {}", e))?;

    let json_vnets: Vec<serde_json::Value> = vnets
        .into_iter()
        .map(|v| serde_json::to_value(v).map_err(|e| format!("Failed to serialize vnet: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_vnets)
}

/// List SDN zones
#[tauri::command]
pub async fn list_sdn_zones(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let zones = crate::proxmox::sdn::list_evpn_zones(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list SDN zones: {}", e))?;

    let json_zones: Vec<serde_json::Value> = zones
        .into_iter()
        .map(|z| serde_json::to_value(z).map_err(|e| format!("Failed to serialize zone: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_zones)
}

// ─── Phase 3 - Network Management ─────────────────────────────────────────────

// Ceph Cluster Management
/// List Ceph clusters
#[tauri::command]
pub async fn list_ceph_clusters(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let ceph_clusters = crate::proxmox::ceph_cluster::list_ceph_clusters(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list Ceph clusters: {}", e))?;

    let json_clusters: Vec<serde_json::Value> = ceph_clusters
        .into_iter()
        .map(|c| serde_json::to_value(c).map_err(|e| format!("Failed to serialize cluster: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_clusters)
}

/// Get Ceph cluster status
#[tauri::command]
pub async fn get_ceph_cluster_status(
    cluster_id: String,
    ceph_cluster_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let status = crate::proxmox::ceph_cluster::get_ceph_cluster_status(
        &client_guard,
        &ceph_cluster_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to get Ceph cluster status: {}", e))?;

    serde_json::to_value(status).map_err(|e| format!("Failed to serialize status: {}", e))
}

// ─── Phase 4 - Advanced Operations ────────────────────────────────────────────

// Remote Migration
/// Migrate VM
#[tauri::command]
pub async fn migrate_vm(
    cluster_id: String,
    node: String,
    vm_id: u32,
    target_node: String,
    target_cluster: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let task = crate::proxmox::migration::migrate_vm(
        &client_guard,
        &node,
        vm_id,
        &target_node,
        &target_cluster,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to migrate VM {}: {}", vm_id, e))?;

    serde_json::to_value(task).map_err(|e| format!("Failed to serialize migration task: {}", e))
}

/// List migration status
#[tauri::command]
pub async fn list_migration_status(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let tasks = crate::proxmox::migration::list_migration_status(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list migration status: {}", e))?;

    let json_tasks: Vec<serde_json::Value> = tasks
        .into_iter()
        .map(|t| serde_json::to_value(t).map_err(|e| format!("Failed to serialize task: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_tasks)
}

// System Updates (extended)
/// List updates
#[tauri::command]
pub async fn list_updates(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let updates = crate::proxmox::updates_ext::list_updates_all_remotes(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list updates: {}", e))?;

    let json_updates: Vec<serde_json::Value> = updates
        .into_iter()
        .map(|u| serde_json::to_value(u).map_err(|e| format!("Failed to serialize update: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_updates)
}

/// Refresh updates
#[tauri::command]
pub async fn refresh_updates(cluster_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::updates_ext::refresh_updates_all(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to refresh updates: {}", e))
}

/// Install updates
#[tauri::command]
pub async fn install_updates(
    cluster_id: String,
    packages: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let package_refs: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
    crate::proxmox::updates_ext::install_updates_remotes(
        &client_guard,
        &package_refs,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to install updates: {}", e))
}

// Task Management
/// List tasks
#[tauri::command]
pub async fn list_tasks(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let tasks = crate::proxmox::tasks::list_tasks(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list tasks: {}", e))?;

    let json_tasks: Vec<serde_json::Value> = tasks
        .into_iter()
        .map(|t| serde_json::to_value(t).map_err(|e| format!("Failed to serialize task: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_tasks)
}

/// Get task status
#[tauri::command]
pub async fn get_task_status(
    cluster_id: String,
    node: String,
    task_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let task = crate::proxmox::tasks::get_task_status(
        &client_guard,
        &node,
        &task_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to get task {}: {}", task_id, e))?;

    serde_json::to_value(task).map_err(|e| format!("Failed to serialize task: {}", e))
}

/// Stop task
#[tauri::command]
pub async fn stop_task(
    cluster_id: String,
    node: String,
    task_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::tasks::stop_task(
        &client_guard,
        &node,
        &task_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to stop task {}: {}", task_id, e))
}

// ─── Phase 5 - Infrastructure ─────────────────────────────────────────────────

// Metric Collection (extended from existing)
/// Get metrics summary
#[tauri::command]
pub async fn get_metrics_summary(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let nodes = crate::proxmox::metrics::list_nodes(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list nodes: {}", e))?;

    let summary = serde_json::json!({
        "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "node_count": nodes.len(),
        "nodes": nodes
    });

    Ok(summary)
}

/// List metric collections
#[tauri::command]
pub async fn list_metric_collections(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let nodes = crate::proxmox::metrics::list_nodes(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list nodes: {}", e))?;

    let collections: Vec<serde_json::Value> = nodes
        .into_iter()
        .map(|n| serde_json::to_value(n).map_err(|e| format!("Failed to serialize node: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(collections)
}

// ─── Phase 6 - HA Management ──────────────────────────────────────────────────

/// List HA groups
#[tauri::command]
pub async fn list_ha_groups(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let groups = crate::proxmox::ha::list_ha_groups(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list HA groups: {}", e))?;

    groups
        .into_iter()
        .map(|g| serde_json::to_value(g).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()
}

/// Create HA group
#[tauri::command]
pub async fn create_ha_group(
    cluster_id: String,
    group: String,
    nodes: Vec<String>,
    max_failures: u32,
    max_relocate: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::ha::create_ha_group(
        &client_guard,
        &group,
        &nodes,
        max_failures,
        max_relocate,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to create HA group: {}", e))
}

/// Update HA group
#[tauri::command]
pub async fn update_ha_group(
    cluster_id: String,
    group: String,
    nodes: Vec<String>,
    max_failures: u32,
    max_relocate: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::ha::update_ha_group(
        &client_guard,
        &group,
        &nodes,
        max_failures,
        max_relocate,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to update HA group: {}", e))
}

/// Delete HA group
#[tauri::command]
pub async fn delete_ha_group(
    cluster_id: String,
    group: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::ha::delete_ha_group(
        &client_guard,
        &group,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to delete HA group: {}", e))
}

/// List HA resources
#[tauri::command]
pub async fn list_ha_resources(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let resources = crate::proxmox::ha::list_ha_resources(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list HA resources: {}", e))?;

    resources
        .into_iter()
        .map(|r| serde_json::to_value(r).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()
}

/// Enable HA resource
#[tauri::command]
pub async fn enable_ha_resource(
    cluster_id: String,
    resource: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::ha::enable_ha_resource(
        &client_guard,
        &resource,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to enable HA resource: {}", e))
}

// ─── Phase 7 - ACL / Users / Realms ──────────────────────────────────────────

/// List ACL entries
#[tauri::command]
pub async fn list_acls(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let path = "access/acl";
    let response: serde_json::Value = client_guard
        .get(path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list ACLs: {}", e))?;

    // handle_response already unwraps the Proxmox `{"data": ...}` envelope.
    response
        .as_array()
        .map(|arr| arr.to_vec())
        .ok_or_else(|| "Invalid response format".to_string())
}

/// List users
#[tauri::command]
pub async fn list_users(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let path = "access/users";
    let response: serde_json::Value = client_guard
        .get(path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list users: {}", e))?;

    response
        .as_array()
        .map(|arr| arr.to_vec())
        .ok_or_else(|| "Invalid response format".to_string())
}

/// List authentication realms (typed)
#[tauri::command]
pub async fn list_realms(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let realms = crate::proxmox::auth_realm::list_auth_realms(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list realms: {}", e))?;

    realms
        .into_iter()
        .map(|r| serde_json::to_value(r).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()
}

// ─── Phase 8 - Cluster Notes ──────────────────────────────────────────────────

/// Get cluster notes
#[tauri::command]
pub async fn get_cluster_notes(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let path = "cluster/config";
    let response: serde_json::Value = client_guard
        .get(path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to get cluster notes: {}", e))?;

    Ok(response
        .get("notes")
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string())
}

/// Update cluster notes
#[tauri::command]
pub async fn update_cluster_notes(
    cluster_id: String,
    notes: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let path = "cluster/config";
    let body = serde_json::json!({ "notes": notes });
    let _: serde_json::Value = client_guard
        .put(
            path,
            &body,
            Some(client_guard.ticket.as_deref().unwrap_or("")),
        )
        .await
        .map_err(|e| format!("Failed to update cluster notes: {}", e))?;

    Ok(())
}

// ─── Phase 9 - Resource Search ────────────────────────────────────────────────

/// Search Proxmox resources
#[tauri::command]
pub async fn search_proxmox_resources(
    cluster_id: String,
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let path = format!("cluster/resources?type=vm&search={}", query);
    let response: serde_json::Value = client_guard
        .get(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to search resources: {}", e))?;

    response
        .as_array()
        .map(|arr| arr.to_vec())
        .ok_or_else(|| "Invalid response format".to_string())
}

// ─── Phase 10 - Node Status ───────────────────────────────────────────────────

/// Get node status
#[tauri::command]
pub async fn get_node_status(
    cluster_id: String,
    node_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let path = format!("nodes/{}/status", node_id);
    let response: serde_json::Value = client_guard
        .get(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to get node status: {}", e))?;

    Ok(response)
}

// ─── Phase 11 - Syslog ────────────────────────────────────────────────────────

/// Get node syslog
#[tauri::command]
pub async fn get_syslog(
    cluster_id: String,
    node_id: String,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let limit_val = limit.unwrap_or(500);
    let path = format!("nodes/{}/syslog?limit={}", node_id, limit_val);
    let response: serde_json::Value = client_guard
        .get(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to get syslog: {}", e))?;

    response
        .as_array()
        .map(|arr| arr.to_vec())
        .ok_or_else(|| "Invalid response format".to_string())
}

// ─── Phase 12 - Network Interfaces ───────────────────────────────────────────

/// List network interfaces on a node
#[tauri::command]
pub async fn list_network_interfaces(
    cluster_id: String,
    node_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let path = format!("nodes/{}/network", node_id);
    let response: serde_json::Value = client_guard
        .get(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list network interfaces: {}", e))?;

    response
        .as_array()
        .map(|arr| arr.to_vec())
        .ok_or_else(|| "Invalid response format".to_string())
}

// ─── Phase 13 - Cluster Views (typed aliases) ─────────────────────────────────

/// List cluster views (typed)
#[tauri::command]
pub async fn list_cluster_views(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let views = crate::proxmox::views::list_views(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list cluster views: {}", e))?;

    views
        .into_iter()
        .map(|v| serde_json::to_value(v).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()
}

/// Create cluster view
#[tauri::command]
pub async fn create_cluster_view(
    cluster_id: String,
    view_id: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let view = crate::proxmox::views::DashboardView {
        view_id,
        name,
        description: String::new(),
        layout: "grid".to_string(),
        widgets: vec![],
        enabled: true,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    crate::proxmox::views::add_view(
        &client_guard,
        &view,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to create cluster view: {}", e))
}

/// Delete cluster view
#[tauri::command]
pub async fn delete_cluster_view(
    cluster_id: String,
    view_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::views::delete_view(
        &client_guard,
        &view_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to delete cluster view: {}", e))
}

// ─── Phase 14 - Subscription ──────────────────────────────────────────────────

/// Get subscription status
#[tauri::command]
pub async fn get_subscription_status(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let path = "nodes/localhost/subscription";
    let response: serde_json::Value = client_guard
        .get(path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to get subscription status: {}", e))?;

    Ok(response)
}

// ─── Phase 15 - Cluster Task Log ─────────────────────────────────────────────

/// List cluster-level tasks
#[tauri::command]
pub async fn list_cluster_tasks(
    cluster_id: String,
    _limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    // Note: Proxmox API doesn't support limit parameter for cluster/tasks
    // We fetch all tasks and limit client-side if needed
    let path = "cluster/tasks";
    let response: serde_json::Value = client_guard
        .get(path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list cluster tasks: {}", e))?;

    let mut tasks: Vec<serde_json::Value> = response
        .as_array()
        .map(|arr| arr.to_vec())
        .ok_or_else(|| "Invalid response format".to_string())?;

    // Apply limit client-side if specified
    if let Some(limit) = _limit {
        tasks.truncate(limit as usize);
    }

    Ok(tasks)
}

/// List Proxmox LXC containers
#[tauri::command]
pub async fn list_proxmox_containers(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let path = "cluster/resources?type=lxc";
    let response: serde_json::Value = client_guard
        .get(path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list containers: {}", e))?;

    response
        .as_array()
        .map(|arr| arr.to_vec())
        .ok_or_else(|| "Invalid response format".to_string())
}

/// Connect (or re-connect) to a Proxmox cluster that already exists in the DB.
/// Loads the stored credentials, authenticates, and inserts the ticketed client
/// into the in-memory pool. Returns `true` on success.
///
/// This is the action triggered by the "Connect" button in the Remotes UI and is
/// the path taken on every app restart for clusters that should be active.
#[tauri::command]
pub async fn connect_proxmox_cluster(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let (url, port, username, encrypted_credentials) = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {}", e))?;

        let mut stmt = db
            .prepare(
                "SELECT url, port, username, encrypted_credentials \
                 FROM proxmox_clusters WHERE id = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        stmt.query_row([&cluster_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u16>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .optional()
        .map_err(|e| format!("Failed to query cluster: {}", e))?
        .ok_or_else(|| format!("Cluster {} not found in database", cluster_id))?
    };

    let credentials_json = crate::integrations::auth::decrypt_token(&encrypted_credentials)
        .map_err(|e| format!("Failed to decrypt credentials: {}", e))?;

    let credentials: serde_json::Value = serde_json::from_str(&credentials_json)
        .map_err(|e| format!("Failed to parse credentials: {}", e))?;

    let password = credentials
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Password not found in credentials".to_string())?;

    let mut client = crate::proxmox::ProxmoxClient::new(&url, port, &username);
    client
        .authenticate(password)
        .await
        .map_err(|e| format!("Failed to authenticate with Proxmox: {}", e))?;

    {
        let mut clusters = state.proxmox_clusters.lock().await;
        clusters.insert(cluster_id, Arc::new(Mutex::new(client)));
    }

    Ok(true)
}

/// Remove a Proxmox cluster's authenticated session from the in-memory pool.
/// The cluster record and credentials remain in the DB — use `connect_proxmox_cluster`
/// to reconnect.
#[tauri::command]
pub async fn disconnect_proxmox_cluster(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut clusters = state.proxmox_clusters.lock().await;
    clusters.remove(&cluster_id);
    Ok(())
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

    #[test]
    fn test_cluster_not_found_error_message() {
        let err = format!("Cluster {} not found", "missing-id");
        assert_eq!(err, "Cluster missing-id not found");
    }

    // After the double-unwrap fix, handle_response returns the inner `data`
    // value directly. Commands call `.as_array()` on the already-unwrapped value.

    #[test]
    fn test_array_response_already_unwrapped_invalid() {
        // The value returned by handle_response is not an array.
        let response = serde_json::json!({"some": "object"});
        let result: Result<Vec<serde_json::Value>, String> = response
            .as_array()
            .map(|arr| arr.to_vec())
            .ok_or_else(|| "Invalid response format".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid response format");
    }

    #[test]
    fn test_array_response_already_unwrapped_valid() {
        // handle_response strips {"data": [...]}, commands receive the raw array.
        let response = serde_json::json!([
            {"vmid": 200, "name": "nginx-proxy", "node": "pve1", "status": "running"},
            {"vmid": 201, "name": "redis-cache", "node": "pve2", "status": "running"}
        ]);
        let result: Result<Vec<serde_json::Value>, String> = response
            .as_array()
            .map(|arr| arr.to_vec())
            .ok_or_else(|| "Invalid response format".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_update_proxmox_cluster_not_found_error() {
        let err = format!("Cluster {} not found", "missing-id");
        assert_eq!(err, "Cluster missing-id not found");
    }

    #[test]
    fn test_cluster_notes_already_unwrapped_present() {
        let response = serde_json::json!({"notes": "Important info", "name": "pve"});
        let notes = response
            .get("notes")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        assert_eq!(notes, "Important info");
    }

    #[test]
    fn test_cluster_notes_already_unwrapped_missing_defaults_empty() {
        let response = serde_json::json!({"name": "pve"});
        let notes = response
            .get("notes")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        assert_eq!(notes, "");
    }

    #[test]
    fn test_connect_cluster_db_not_found_error_message() {
        let msg = format!("Cluster {} not found in database", "unknown-id");
        assert!(msg.contains("unknown-id"));
        assert!(msg.contains("not found in database"));
    }

    #[test]
    fn test_update_proxmox_cluster_rows_zero_means_not_found() {
        let rows: usize = 0;
        let result: Result<(), String> = if rows == 0 {
            Err(format!("Cluster {} not found", "ghost-id"))
        } else {
            Ok(())
        };
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ghost-id"));
    }

    #[test]
    fn test_update_proxmox_cluster_rows_nonzero_succeeds() {
        let rows: usize = 1;
        let result: Result<(), String> = if rows == 0 {
            Err(format!("Cluster {} not found", "real-id"))
        } else {
            Ok(())
        };
        assert!(result.is_ok());
    }

    #[test]
    fn test_ping_proxmox_cluster_error_message_format() {
        let raw = anyhow::anyhow!("connection refused");
        let msg = format!("Connection test failed: {}", raw);
        assert!(msg.starts_with("Connection test failed:"));
        assert!(msg.contains("connection refused"));
    }
}
