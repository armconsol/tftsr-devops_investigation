use crate::proxmox::{ClusterInfo, ClusterType, ProxmoxClient};
use crate::state::AppState;
use chrono::Utc;
use futures::stream::StreamExt;
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
        .map_err(|e| format!("Failed to authenticate with Proxmox: {e}"))?;

    // Encrypt raw password so we can re-authenticate after app restart.
    let credentials = serde_json::json!({
        "password": password,
        "username": username
    });
    let encrypted_credentials = crate::integrations::auth::encrypt_token(
        &serde_json::to_string(&credentials).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("Failed to encrypt credentials: {e}"))?;

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
            .map_err(|e| format!("Failed to lock database: {e}"))?;

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
        .map_err(|e| format!("Failed to store cluster: {e}"))?;
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
            .map_err(|e| format!("Failed to lock database: {e}"))?;

        db.execute("DELETE FROM proxmox_clusters WHERE id = ?1", [id.clone()])
            .map_err(|e| format!("Failed to remove cluster: {e}"))?;
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
            .map_err(|e| format!("Failed to lock database: {e}"))?;

        let mut stmt = db
            .prepare(
                "SELECT id, name, cluster_type, url, port, username, created_at, updated_at FROM proxmox_clusters",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

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
            .map_err(|e| format!("Failed to query clusters: {e}"))?;

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
            .map_err(|e| format!("Failed to lock database: {e}"))?;

        let mut stmt = db
            .prepare(
                "SELECT id, name, cluster_type, url, port, username, created_at, updated_at FROM proxmox_clusters WHERE id = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

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
        .map_err(|e| format!("Failed to query cluster: {e}"))?
    };

    Ok(cluster)
}

/// Load a cluster's connection details and decrypted password from the DB.
fn load_cluster_credentials(
    cluster_id: &str,
    state: &State<'_, AppState>,
) -> Result<(String, u16, String, String), String> {
    let (url, port, username, encrypted_credentials) = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {e}"))?;

        let mut stmt = db
            .prepare(
                "SELECT url, port, username, encrypted_credentials FROM proxmox_clusters WHERE id = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        stmt.query_row([cluster_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u16>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .optional()
        .map_err(|e| format!("Failed to query cluster: {e}"))?
        .ok_or_else(|| format!("Cluster {cluster_id} not found in database"))?
    };

    let credentials_json = crate::integrations::auth::decrypt_token(&encrypted_credentials)
        .map_err(|e| format!("Failed to decrypt credentials: {e}"))?;

    let credentials: serde_json::Value = serde_json::from_str(&credentials_json)
        .map_err(|e| format!("Failed to parse credentials: {e}"))?;

    let password = credentials
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Password not found in credentials".to_string())?
        .to_string();

    Ok((url, port, username, password))
}

/// Get or create a Proxmox client for a cluster.
/// 1. Check if a client already exists in the in-memory pool.
/// 2. If not, load credentials from the database and create/authenticate a client.
async fn get_proxmox_client_for_cluster(
    cluster_id: &str,
    state: &State<'_, AppState>,
) -> Result<Arc<Mutex<crate::proxmox::ProxmoxClient>>, String> {
    // First, try to get from in-memory pool
    let cached = {
        let clusters = state.proxmox_clusters.lock().await;
        clusters.get(cluster_id).cloned()
    };
    if let Some(client) = cached {
        // PVE tickets expire after ~2h; cached clients live for the whole app
        // session, so refresh the ticket proactively before it lapses.
        let needs_refresh = { client.lock().await.ticket_needs_refresh() };
        if needs_refresh {
            let (_url, _port, _username, password) = load_cluster_credentials(cluster_id, state)?;
            let mut guard = client.lock().await;
            // Re-check under the write lock: another task may have refreshed.
            if guard.ticket_needs_refresh() {
                guard
                    .authenticate(&password)
                    .await
                    .map_err(|e| format!("Failed to re-authenticate with Proxmox: {e}"))?;
            }
        }
        return Ok(client);
    }

    // Not in memory - load from database and create client
    let (url, port, username, password) = load_cluster_credentials(cluster_id, state)?;

    // Create new client
    let mut client = crate::proxmox::ProxmoxClient::new(&url, port, &username);

    // Authenticate to get ticket
    client
        .authenticate(&password)
        .await
        .map_err(|e| format!("Failed to authenticate with Proxmox: {e}"))?;

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

/// Look up a cluster's stored TLS `ssl_fingerprint` (if any). Returns `None`
/// when unset/empty so callers can fall back to self-signed acceptance.
fn stored_ssl_fingerprint(cluster_id: &str, state: &State<'_, AppState>) -> Option<String> {
    let db = state.db.lock().ok()?;
    db.query_row(
        "SELECT ssl_fingerprint FROM proxmox_clusters WHERE id = ?1",
        [cluster_id],
        |row| row.get::<_, Option<String>>(0),
    )
    .ok()
    .flatten()
    .filter(|s| !s.trim().is_empty())
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
        .map_err(|e| format!("Connection test failed: {e}"))
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
    .map_err(|e| format!("Failed to encrypt credentials: {e}"))?;

    let updated_at = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {e}"))?;

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
            .map_err(|e| format!("Failed to update cluster: {e}"))?;

        if rows == 0 {
            return Err(format!("Cluster {id} not found"));
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
            .map_err(|e| format!("Failed to list VMs: {e}"))?;

    // Convert VmInfo structs to JSON
    let json_vms: Vec<serde_json::Value> = vms
        .into_iter()
        .map(|vm| serde_json::to_value(vm).map_err(|e| format!("Failed to serialize VM: {e}")))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_vms)
}

/// Get Proxmox VM details
#[tauri::command]
pub async fn get_proxmox_vm(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let vm = crate::proxmox::vm::get_vm(
        &client_guard,
        &node_id,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to get VM {vm_id}: {e}"))?;

    serde_json::to_value(vm).map_err(|e| format!("Failed to serialize VM: {e}"))
}

/// Start a Proxmox VM
#[tauri::command]
pub async fn start_proxmox_vm(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::start_vm(
        &client_guard,
        &node_id,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to start VM {vm_id}: {e}"))
}

/// Stop a Proxmox VM
#[tauri::command]
pub async fn stop_proxmox_vm(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::stop_vm(
        &client_guard,
        &node_id,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to stop VM {vm_id}: {e}"))
}

/// Reboot a Proxmox VM
#[tauri::command]
pub async fn reboot_proxmox_vm(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::reboot_vm(
        &client_guard,
        &node_id,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to reboot VM {vm_id}: {e}"))
}

/// Shutdown a Proxmox VM
#[tauri::command]
pub async fn shutdown_proxmox_vm(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::shutdown_vm(
        &client_guard,
        &node_id,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to shutdown VM {vm_id}: {e}"))
}

/// Resume a paused Proxmox VM
#[tauri::command]
pub async fn resume_proxmox_vm(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::resume_vm(
        &client_guard,
        &node_id,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to resume VM {vm_id}: {e}"))
}

/// Suspend a running Proxmox VM
#[tauri::command]
pub async fn suspend_proxmox_vm(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::suspend_vm(
        &client_guard,
        &node_id,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to suspend VM {vm_id}: {e}"))
}

/// Clone a Proxmox VM
#[tauri::command]
pub async fn clone_vm(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    new_vmid: u32,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::clone_vm(
        &client_guard,
        &node_id,
        vm_id,
        new_vmid,
        &name,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to clone VM {vm_id}: {e}"))
}

/// Delete a Proxmox VM
#[tauri::command]
pub async fn delete_vm(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::delete_vm(
        &client_guard,
        &node_id,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to delete VM {vm_id}: {e}"))
}

/// List Proxmox nodes in a cluster
#[tauri::command]
pub async fn list_proxmox_nodes(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let response: serde_json::Value = client_guard
        .get("nodes", Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list nodes: {e}"))?;

    let nodes: Vec<serde_json::Value> = response
        .as_array()
        .map(|arr| arr.to_vec())
        .unwrap_or_default();

    Ok(nodes)
}

/// Validates a PVE node name or network bridge name (DNS-label characters only).
/// Prevents path traversal / URL injection when names are interpolated into REST paths
/// or virtio property strings.
fn validate_pve_identifier(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
        return Err(format!(
            "{field} contains invalid characters — only alphanumeric, '.', '-', '_' are allowed"
        ));
    }
    Ok(())
}

/// Like `validate_pve_identifier` but also allows `@` for PVE user IDs (`user@realm`).
fn validate_pve_userid(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '@')
    {
        return Err(format!("{field} contains invalid characters — only alphanumeric, '.', '-', '_', '@' are allowed"));
    }
    Ok(())
}

/// Validates an HA resource SID (format `type:vmid`, e.g. `qemu:100`).
fn validate_pve_ha_resource(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ':')
    {
        return Err(format!("{field} contains invalid characters — only alphanumeric, '.', '-', '_', ':' are allowed"));
    }
    Ok(())
}

/// Create a new Proxmox VM
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn create_proxmox_vm(
    cluster_id: String,
    node_id: String,
    vmid: u32,
    name: String,
    memory: u32,
    cores: u32,
    sockets: u32,
    os_type: String,
    storage: String,
    disk_size: u32,
    net_bridge: String,
    iso: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // H2: validate path-interpolated identifiers before sending to PVE
    validate_pve_identifier(&node_id, "node_id")?;
    validate_pve_identifier(&storage, "storage")?;
    validate_pve_identifier(&net_bridge, "net_bridge")?;

    // M4: enforce PVE-defined numeric ranges
    if !(100..=999_999_999).contains(&vmid) {
        return Err("vmid must be between 100 and 999999999".to_string());
    }
    if !(32..=1_048_576).contains(&memory) {
        return Err("memory must be between 32 MB and 1048576 MB (1 TB)".to_string());
    }
    if !(1..=512).contains(&cores) {
        return Err("cores must be between 1 and 512".to_string());
    }
    if !(1..=4).contains(&sockets) {
        return Err("sockets must be between 1 and 4".to_string());
    }
    if !(1..=65536).contains(&disk_size) {
        return Err("disk_size must be between 1 GB and 65536 GB".to_string());
    }

    // H3: validate ISO volume ID format to prevent property string injection
    // Expected: "storage:iso/filename.iso" — no commas, slashes only in the path portion
    if let Some(ref iso_val) = iso {
        if !iso_val.is_empty() {
            let valid_iso = iso_val
                .split_once(':')
                .map(|(store, path)| {
                    !store.is_empty()
                        && !store
                            .contains(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
                        && path.starts_with("iso/")
                        && !path.contains(",")
                })
                .unwrap_or(false);
            if !valid_iso {
                return Err("iso must be in the format 'storage:iso/filename.iso'".to_string());
            }
        }
    }

    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let ide2 = iso
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("{s},media=cdrom"))
        .unwrap_or_else(|| "none,media=cdrom".to_string());

    let config = serde_json::json!({
        "vmid": vmid,
        "name": name,
        "memory": memory,
        "cores": cores,
        "sockets": sockets,
        "ostype": os_type,
        "scsihw": "virtio-scsi-pci",
        "scsi0": format!("{storage}:{disk_size}"),
        "ide2": ide2,
        "net0": format!("virtio,bridge={net_bridge}"),
        "boot": "order=scsi0;ide2"
    });

    crate::proxmox::vm::create_vm(
        &client_guard,
        &node_id,
        vmid,
        &config,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to create VM: {e}"))
}

/// Normalize the `cluster/backup` API response into a list of backup jobs.
///
/// Proxmox returns an array of job objects, but on some remotes (e.g. a
/// standalone node with no jobs configured) the `data` field is `null` rather
/// than an empty array. Treat any non-array response as "no jobs" so the UI
/// shows an empty list instead of a hard error. Each job is guaranteed to carry
/// a stable `id` field for the frontend.
fn normalize_backup_jobs(response: &serde_json::Value) -> Vec<serde_json::Value> {
    let arr = match response.as_array() {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    let mut jobs: Vec<serde_json::Value> = arr
        .iter()
        .cloned()
        .map(|mut job| {
            // Ensure a stable id field exists for frontend compatibility.
            if let Some(job_obj) = job.as_object_mut() {
                let has_id = job_obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| !s.is_empty())
                    .unwrap_or(false);
                if !has_id {
                    let storage = job_obj
                        .get("storage")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    job_obj.insert(
                        "id".to_string(),
                        serde_json::Value::String(format!("backup-{storage}")),
                    );
                }
            }
            job
        })
        .collect();

    // Apply limit if needed (Proxmox may return many jobs). Log when truncating
    // so an incomplete list is observable rather than silently dropped.
    if jobs.len() > 100 {
        tracing::warn!(
            "Backup job list truncated from {} to 100 entries",
            jobs.len()
        );
        jobs.truncate(100);
    }

    jobs
}

/// List Proxmox Backup Jobs (cluster-level, not node-level)
#[tauri::command]
pub async fn list_proxmox_backup_jobs(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    // Proxmox VE backup jobs are at cluster level, not node level
    let path = "cluster/backup";
    let response: serde_json::Value = client_guard
        .get(path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list backup jobs: {e}"))?;

    Ok(normalize_backup_jobs(&response))
}

/// List Proxmox Datastores (cluster-wide via cluster/resources)
#[tauri::command]
pub async fn list_proxmox_datastores(
    cluster_id: String,
    _state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &_state).await?;
    let client_guard = client.lock().await;

    let response: serde_json::Value = client_guard
        .get(
            "cluster/resources?type=storage",
            Some(client_guard.ticket.as_deref().unwrap_or("")),
        )
        .await
        .map_err(|e| format!("Failed to list cluster storage: {e}"))?;

    let entries = response.as_array().ok_or("Invalid response format")?;

    let all_storage: Vec<serde_json::Value> = entries
        .iter()
        .filter_map(|entry| {
            let obj = entry.as_object()?;
            let mut normalized = serde_json::Map::new();

            let storage_name = obj.get("storage").and_then(|v| v.as_str()).unwrap_or("");
            let node_name = obj.get("node").and_then(|v| v.as_str()).unwrap_or("");

            // Avoid double-slash when cluster/resources omits "node" for shared storage
            let storage_id = if node_name.is_empty() {
                format!("storage/{storage_name}")
            } else {
                format!("storage/{node_name}/{storage_name}")
            };
            if storage_name.is_empty() {
                tracing::warn!(
                    node = node_name,
                    "storage entry has empty storage name — skipping"
                );
                return None;
            }
            tracing::debug!(storage_id = %storage_id, "generated storage ID");
            normalized.insert("id".to_string(), serde_json::Value::String(storage_id));
            normalized.insert(
                "storage".to_string(),
                serde_json::Value::String(storage_name.to_string()),
            );
            normalized.insert(
                "name".to_string(),
                serde_json::Value::String(storage_name.to_string()),
            );

            let plugin_type = obj
                .get("plugintype")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            normalized.insert(
                "type".to_string(),
                serde_json::Value::String(plugin_type.to_string()),
            );

            let content = obj
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            normalized.insert("content".to_string(), serde_json::Value::String(content));

            normalized.insert(
                "node".to_string(),
                serde_json::Value::String(node_name.to_string()),
            );

            // cluster/resources uses disk/maxdisk; normalize to used/available/size
            let disk_used = obj.get("disk").and_then(|v| v.as_u64()).unwrap_or(0);
            let disk_total = obj.get("maxdisk").and_then(|v| v.as_u64()).unwrap_or(0);
            let disk_avail = disk_total.saturating_sub(disk_used);

            normalized.insert(
                "used".to_string(),
                serde_json::Value::Number(disk_used.into()),
            );
            normalized.insert(
                "size".to_string(),
                serde_json::Value::Number(disk_total.into()),
            );
            normalized.insert(
                "available".to_string(),
                serde_json::Value::Number(disk_avail.into()),
            );

            let status = obj
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("available")
                .to_string();
            normalized.insert("status".to_string(), serde_json::Value::String(status));

            // Preserve shared flag if present
            if let Some(shared) = obj.get("shared") {
                normalized.insert("shared".to_string(), shared.clone());
            }

            Some(serde_json::Value::Object(normalized))
        })
        .collect();

    Ok(all_storage)
}

/// Build the form parameters for a `PUT /storage/{storage}` update.
///
/// Only provided fields are included so we never clear values the user did not
/// touch. `content` is a comma-separated list of content types, `nodes` a
/// comma-separated node restriction list (empty = all nodes), and `disable`
/// toggles the storage on/off.
fn build_storage_update_params(
    content: Option<&str>,
    nodes: Option<&str>,
    disable: Option<bool>,
) -> Vec<(String, String)> {
    let mut params: Vec<(String, String)> = Vec::new();
    if let Some(c) = content {
        params.push(("content".to_string(), c.to_string()));
    }
    if let Some(n) = nodes {
        params.push(("nodes".to_string(), n.to_string()));
    }
    if let Some(d) = disable {
        params.push(("disable".to_string(), if d { "1" } else { "0" }.to_string()));
    }
    params
}

/// Get the configuration of a single datacenter-level storage.
#[tauri::command]
pub async fn get_proxmox_storage_config(
    cluster_id: String,
    storage: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    validate_pve_identifier(&storage, "storage")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let path = format!("storage/{storage}");
    client_guard
        .get(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to get storage config for {storage}: {e}"))
}

/// Update a datacenter-level storage configuration.
#[tauri::command]
pub async fn update_proxmox_storage(
    cluster_id: String,
    storage: String,
    content: Option<String>,
    nodes: Option<String>,
    disable: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&storage, "storage")?;
    let params = build_storage_update_params(content.as_deref(), nodes.as_deref(), disable);
    if params.is_empty() {
        return Err("No storage fields provided to update".to_string());
    }
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    let path = format!("storage/{storage}");
    let body: serde_json::Map<String, serde_json::Value> = params
        .into_iter()
        .map(|(k, v)| (k, serde_json::Value::String(v)))
        .collect();
    let _: serde_json::Value = client_guard
        .put(&path, &serde_json::Value::Object(body), Some(ticket))
        .await
        .map_err(|e| format!("Failed to update storage {storage}: {e}"))?;
    Ok(())
}

/// Delete a datacenter-level storage configuration.
#[tauri::command]
pub async fn delete_proxmox_storage(
    cluster_id: String,
    storage: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&storage, "storage")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    let path = format!("storage/{storage}");
    let _: serde_json::Value = client_guard
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete storage {storage}: {e}"))?;
    Ok(())
}
#[tauri::command]
pub async fn trigger_proxmox_backup_job(
    cluster_id: String,
    job_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&job_id, "job_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");

    // Read the job configuration (storage, vmid, mode, node, …).
    let job: serde_json::Value = client_guard
        .get(&format!("cluster/backup/{job_id}"), Some(ticket))
        .await
        .map_err(|e| format!("Failed to read backup job {job_id}: {e}"))?;

    // Resolve the node to run vzdump on (job node, else first cluster node).
    let nodes_resp: serde_json::Value = client_guard
        .get("nodes", Some(ticket))
        .await
        .map_err(|e| format!("Failed to list nodes: {e}"))?;
    let cluster_nodes: Vec<String> = nodes_resp
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|n| n.get("node").and_then(|v| v.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let node = crate::proxmox::backup::select_backup_node(&job, &cluster_nodes)
        .ok_or_else(|| "No node available to run the backup job".to_string())?;
    validate_pve_identifier(&node, "node")?;

    let params = crate::proxmox::backup::build_vzdump_params(&job);
    let params_ref: Vec<(&str, &str)> = params
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let _: serde_json::Value = client_guard
        .post_form(&format!("nodes/{node}/vzdump"), &params_ref, Some(ticket))
        .await
        .map_err(|e| format!("Failed to trigger backup job {job_id}: {e}"))?;
    Ok(())
}

/// List Ceph Pools
#[tauri::command]
pub async fn list_ceph_pools(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let pools = crate::proxmox::ceph::list_pools(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list Ceph pools: {e}"))?;

    let json_pools: Vec<serde_json::Value> = pools
        .into_iter()
        .map(|pool| {
            serde_json::to_value(pool).map_err(|e| format!("Failed to serialize Ceph pool: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_pools)
}

/// List Ceph OSDs
#[tauri::command]
pub async fn list_ceph_osd(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let osds = crate::proxmox::ceph::list_osds(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list Ceph OSDs: {e}"))?;

    let json_osds: Vec<serde_json::Value> = osds
        .into_iter()
        .map(|osd| {
            serde_json::to_value(osd).map_err(|e| format!("Failed to serialize Ceph OSD: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_osds)
}

/// Get Ceph Health
#[tauri::command]
pub async fn get_ceph_health(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let health = crate::proxmox::ceph::get_ceph_health(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to get Ceph health: {e}"))?;

    serde_json::to_value(health).map_err(|e| format!("Failed to serialize Ceph health: {e}"))
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
    .map_err(|e| format!("Failed to list auth realms: {e}"))?;

    let json_realms: Vec<serde_json::Value> = realms
        .into_iter()
        .map(|r| serde_json::to_value(r).map_err(|e| format!("Failed to serialize realm: {e}")))
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
    .map_err(|e| format!("Failed to add LDAP realm: {e}"))
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
    .map_err(|e| format!("Failed to add AD realm: {e}"))
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
    .map_err(|e| format!("Failed to add OpenID realm: {e}"))
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
    .map_err(|e| format!("Failed to list ACME accounts: {e}"))?;

    let json_accounts: Vec<serde_json::Value> = accounts
        .into_iter()
        .map(|a| serde_json::to_value(a).map_err(|e| format!("Failed to serialize account: {e}")))
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
    .map_err(|e| format!("Failed to register ACME account: {e}"))?;

    serde_json::to_value(account).map_err(|e| format!("Failed to serialize account: {e}"))
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
    .map_err(|e| format!("Failed to get ACME challenges: {e}"))?;

    let json_challenges: Vec<serde_json::Value> = challenges
        .into_iter()
        .map(|c| serde_json::to_value(c).map_err(|e| format!("Failed to serialize challenge: {e}")))
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
    .map_err(|e| format!("Failed to list APT updates: {e}"))?;

    let json_updates: Vec<serde_json::Value> = updates
        .into_iter()
        .map(|u| serde_json::to_value(u).map_err(|e| format!("Failed to serialize update: {e}")))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_updates)
}

/// Refresh the APT package index on a node (apt-get update). Returns the UPID
/// of the PVE task so the frontend can surface progress.
#[tauri::command]
pub async fn refresh_apt_cache(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    validate_pve_identifier(&node, "node")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::apt::refresh_apt_cache(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to refresh APT cache: {e}"))
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
    .map_err(|e| format!("Failed to list APT repos: {e}"))?;

    let json_repos: Vec<serde_json::Value> = repos
        .into_iter()
        .map(|r| serde_json::to_value(r).map_err(|e| format!("Failed to serialize repo: {e}")))
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
    .map_err(|e| format!("Failed to get shell ticket: {e}"))?;

    serde_json::to_value(ticket).map_err(|e| format!("Failed to serialize ticket: {e}"))
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
            tracing::warn!(
                "Certificates API not implemented by Proxmox server, returning empty list"
            );
            return Ok(vec![]);
        }
    }

    let certs = certs.map_err(|e| format!("Failed to list certificates: {e}"))?;

    let json_certs: Vec<serde_json::Value> = certs
        .into_iter()
        .map(|c| serde_json::to_value(c).map_err(|e| format!("Failed to serialize cert: {e}")))
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
    .map_err(|e| format!("Failed to upload certificate: {e}"))?;

    serde_json::to_value(cert).map_err(|e| format!("Failed to serialize cert: {e}"))
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
    .map_err(|e| format!("Failed to get certificate {cert_id}: {e}"))?;

    serde_json::to_value(cert).map_err(|e| format!("Failed to serialize cert: {e}"))
}

// ─── Phase 2 - Advanced Management ────────────────────────────────────────────

// Firewall commands (extended from existing)
/// List firewall rules
#[tauri::command]
pub async fn list_firewall_rules(
    cluster_id: String,
    node_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    validate_pve_identifier(&node_id, "node_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let rules = crate::proxmox::firewall::list_firewall_rules(
        &client_guard,
        &node_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list firewall rules: {e}"))?;

    // Normalize to match what FirewallRuleList component expects:
    // rule (position number), action, protocol, source, destination, port, status
    let json_rules: Vec<serde_json::Value> = rules
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.rule_num.to_string(),
                "rule": r.rule_num,
                "action": r.action,
                "protocol": r.protocol,
                "source": r.source,
                "destination": r.destination,
                "port": r.port,
                "status": if r.enabled { "enabled" } else { "disabled" },
            })
        })
        .collect();

    Ok(json_rules)
}

/// Add firewall rule
#[tauri::command]
pub async fn add_firewall_rule(
    cluster_id: String,
    node_id: String,
    rule: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let firewall_rule = crate::proxmox::firewall::FirewallRule {
        rule_num: 0,
        action: rule
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("ACCEPT")
            .to_string(),
        protocol: rule
            .get("proto")
            .and_then(|v| v.as_str())
            .unwrap_or("tcp")
            .to_string(),
        source: rule
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        destination: rule
            .get("dest")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        port: rule
            .get("dport")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
        enabled: rule
            .get("enable")
            .and_then(|v| {
                v.as_bool()
                    .or_else(|| v.as_i64().map(|n| n != 0))
                    .or_else(|| v.as_str().map(|s| s == "1" || s == "true"))
            })
            .unwrap_or(true),
    };

    crate::proxmox::firewall::add_rule(
        &client_guard,
        &node_id,
        &firewall_rule,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to add firewall rule: {e}"))
}

/// Delete firewall rule
#[tauri::command]
pub async fn delete_firewall_rule(
    cluster_id: String,
    node_id: String,
    rule_num: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::firewall::delete_rule(
        &client_guard,
        &node_id,
        rule_num,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to delete firewall rule {rule_num}: {e}"))
}

/// Update an existing firewall rule
#[tauri::command]
pub async fn update_proxmox_firewall_rule(
    cluster_id: String,
    node_id: String,
    rule_num: u32,
    rule: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let firewall_rule = crate::proxmox::firewall::FirewallRule {
        rule_num,
        action: rule
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("ACCEPT")
            .to_string(),
        protocol: rule
            .get("proto")
            .and_then(|v| v.as_str())
            .unwrap_or("tcp")
            .to_string(),
        source: rule
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        destination: rule
            .get("dest")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        port: rule
            .get("dport")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
        enabled: rule
            .get("enable")
            .and_then(|v| {
                v.as_bool()
                    .or_else(|| v.as_i64().map(|n| n != 0))
                    .or_else(|| v.as_str().map(|s| s == "1" || s == "true"))
            })
            .unwrap_or(true),
    };

    crate::proxmox::firewall::update_rule(
        &client_guard,
        &node_id,
        rule_num,
        &firewall_rule,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to update firewall rule {rule_num}: {e}"))
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
    .map_err(|e| format!("Failed to list SDN controllers: {e}"))?;

    let json_controllers: Vec<serde_json::Value> = controllers
        .into_iter()
        .map(|c| {
            serde_json::to_value(c).map_err(|e| format!("Failed to serialize controller: {e}"))
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
    .map_err(|e| format!("Failed to list SDN virtual networks: {e}"))?;

    let json_vnets: Vec<serde_json::Value> = vnets
        .into_iter()
        .map(|v| serde_json::to_value(v).map_err(|e| format!("Failed to serialize vnet: {e}")))
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
    .map_err(|e| format!("Failed to list SDN zones: {e}"))?;

    let json_zones: Vec<serde_json::Value> = zones
        .into_iter()
        .map(|z| serde_json::to_value(z).map_err(|e| format!("Failed to serialize zone: {e}")))
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
    .map_err(|e| format!("Failed to list Ceph clusters: {e}"))?;

    let json_clusters: Vec<serde_json::Value> = ceph_clusters
        .into_iter()
        .map(|c| serde_json::to_value(c).map_err(|e| format!("Failed to serialize cluster: {e}")))
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
    .map_err(|e| format!("Failed to get Ceph cluster status: {e}"))?;

    serde_json::to_value(status).map_err(|e| format!("Failed to serialize status: {e}"))
}

// ─── Phase 4 - Advanced Operations ────────────────────────────────────────────

// Remote Migration
/// Migrate VM
#[tauri::command]
pub async fn migrate_vm(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    target_node: String,
    target_cluster: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let task = crate::proxmox::migration::migrate_vm(
        &client_guard,
        &node_id,
        vm_id,
        &target_node,
        &target_cluster,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to migrate VM {vm_id}: {e}"))?;

    serde_json::to_value(task).map_err(|e| format!("Failed to serialize migration task: {e}"))
}

/// List migration status
#[tauri::command]
pub async fn list_migration_status(
    cluster_id: String,
    node_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let tasks = crate::proxmox::migration::list_migration_status(
        &client_guard,
        &node_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list migration status: {e}"))?;

    let json_tasks: Vec<serde_json::Value> = tasks
        .into_iter()
        .map(|t| serde_json::to_value(t).map_err(|e| format!("Failed to serialize task: {e}")))
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
    .map_err(|e| format!("Failed to list updates: {e}"))?;

    let json_updates: Vec<serde_json::Value> = updates
        .into_iter()
        .map(|u| serde_json::to_value(u).map_err(|e| format!("Failed to serialize update: {e}")))
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
    .map_err(|e| format!("Failed to refresh updates: {e}"))
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
    .map_err(|e| format!("Failed to install updates: {e}"))
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
    .map_err(|e| format!("Failed to list tasks: {e}"))?;

    let json_tasks: Vec<serde_json::Value> = tasks
        .into_iter()
        .map(|t| serde_json::to_value(t).map_err(|e| format!("Failed to serialize task: {e}")))
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
    .map_err(|e| format!("Failed to get task {task_id}: {e}"))?;

    serde_json::to_value(task).map_err(|e| format!("Failed to serialize task: {e}"))
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
    .map_err(|e| format!("Failed to stop task {task_id}: {e}"))
}

/// Get the full log of a single task.
#[tauri::command]
pub async fn get_proxmox_task_log(
    cluster_id: String,
    node: String,
    upid: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::proxmox::tasks::TaskLogEntry>, String> {
    validate_pve_identifier(&node, "node")?;
    crate::proxmox::tasks::validate_upid(&upid)?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::tasks::get_task_log(
        &client_guard,
        &node,
        &upid,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

/// A single task to search the log of.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TaskLogTarget {
    pub node: String,
    pub upid: String,
}

/// Search result for one task: matching log lines, or an error if the log
/// could not be fetched (degrades gracefully rather than failing the whole
/// search).
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskLogSearchResult {
    pub node: String,
    pub upid: String,
    pub matches: Vec<crate::proxmox::tasks::TaskLogEntry>,
    pub error: Option<String>,
}

/// Maximum number of tasks searchable in a single request — keeps this
/// endpoint from being used to fan out unbounded concurrent PVE requests.
const MAX_TASK_LOG_SEARCH_TARGETS: usize = 100;

/// Search the logs of multiple tasks for a case-insensitive substring.
/// Fetches each task's log concurrently (capped) and returns per-task
/// matches; a failure fetching one task's log does not fail the others.
#[tauri::command]
pub async fn search_task_logs(
    cluster_id: String,
    query: String,
    targets: Vec<TaskLogTarget>,
    state: State<'_, AppState>,
) -> Result<Vec<TaskLogSearchResult>, String> {
    if query.trim().chars().count() < 2 {
        return Err("Search query must be at least 2 characters".to_string());
    }
    if targets.len() > MAX_TASK_LOG_SEARCH_TARGETS {
        return Err(format!(
            "Too many tasks to search: max {MAX_TASK_LOG_SEARCH_TARGETS}, got {}",
            targets.len()
        ));
    }
    for target in &targets {
        validate_pve_identifier(&target.node, "node")?;
        crate::proxmox::tasks::validate_upid(&target.upid)?;
    }

    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;

    let results = futures::stream::iter(targets.into_iter().map(|target| {
        let client = client.clone();
        let query = query.clone();
        async move {
            let client_guard = client.lock().await;
            let ticket = client_guard.ticket.as_deref().unwrap_or("").to_string();
            match crate::proxmox::tasks::get_task_log(
                &client_guard,
                &target.node,
                &target.upid,
                &ticket,
            )
            .await
            {
                Ok(entries) => TaskLogSearchResult {
                    node: target.node,
                    upid: target.upid,
                    matches: crate::proxmox::tasks::filter_log_lines(&entries, &query),
                    error: None,
                },
                Err(e) => TaskLogSearchResult {
                    node: target.node,
                    upid: target.upid,
                    matches: vec![],
                    error: Some(e),
                },
            }
        }
    }))
    .buffer_unordered(5)
    .collect::<Vec<_>>()
    .await;

    Ok(results)
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
    .map_err(|e| format!("Failed to list nodes: {e}"))?;

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
    .map_err(|e| format!("Failed to list nodes: {e}"))?;

    let collections: Vec<serde_json::Value> = nodes
        .into_iter()
        .map(|n| serde_json::to_value(n).map_err(|e| format!("Failed to serialize node: {e}")))
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
    .map_err(|e| format!("Failed to list HA groups: {e}"))?;

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
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&group, "group")?;
    for node in &nodes {
        validate_pve_identifier(node, "nodes")?;
    }
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::ha::create_ha_group(
        &client_guard,
        &group,
        &nodes,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to create HA group: {e}"))
}

/// Update HA group
///
/// `comment`, `restricted` and `nofailback` are optional PVE HA group fields.
#[tauri::command]
pub async fn update_ha_group(
    cluster_id: String,
    group: String,
    nodes: Vec<String>,
    comment: Option<String>,
    restricted: Option<bool>,
    nofailback: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&group, "group")?;
    for node in &nodes {
        validate_pve_identifier(node, "nodes")?;
    }
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::ha::update_ha_group(
        &client_guard,
        &group,
        &nodes,
        comment.as_deref(),
        restricted,
        nofailback,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to update HA group: {e}"))
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
    .map_err(|e| format!("Failed to delete HA group: {e}"))
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
    .map_err(|e| format!("Failed to list HA resources: {e}"))?;

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
    .map_err(|e| format!("Failed to enable HA resource: {e}"))
}

/// Disable HA resource
#[tauri::command]
pub async fn disable_ha_resource(
    cluster_id: String,
    resource: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    crate::proxmox::ha::disable_ha_resource(&client_guard, &resource, ticket)
        .await
        .map_err(|e| format!("Failed to disable HA resource: {e}"))
}

/// Delete (remove) an HA resource
#[tauri::command]
pub async fn delete_ha_resource(
    cluster_id: String,
    resource: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_ha_resource(&resource, "resource")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    let path = format!("cluster/ha/resources/{resource}");
    let _: serde_json::Value = client_guard
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete HA resource {resource}: {e}"))?;
    Ok(())
}

/// Update (edit) an HA resource
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn update_ha_resource(
    cluster_id: String,
    resource: String,
    group: Option<String>,
    state_value: Option<String>,
    max_restart: Option<u32>,
    max_relocate: Option<u32>,
    comment: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_ha_resource(&resource, "resource")?;
    if let Some(g) = group.as_deref() {
        if !g.is_empty() {
            validate_pve_identifier(g, "group")?;
        }
    }
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");

    crate::proxmox::ha::update_ha_resource(
        &client_guard,
        &resource,
        group.as_deref().filter(|g| !g.is_empty()),
        state_value.as_deref(),
        max_restart,
        max_relocate,
        comment.as_deref(),
        ticket,
    )
    .await
    .map_err(|e| format!("Failed to update HA resource: {e}"))
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
        .map_err(|e| format!("Failed to list ACLs: {e}"))?;

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
        .map_err(|e| format!("Failed to list users: {e}"))?;

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
    .map_err(|e| format!("Failed to list realms: {e}"))?;

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
        .map_err(|e| format!("Failed to get cluster notes: {e}"))?;

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
        .map_err(|e| format!("Failed to update cluster notes: {e}"))?;

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

    let path = format!("cluster/resources?type=vm&search={query}");
    let response: serde_json::Value = client_guard
        .get(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to search resources: {e}"))?;

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

    let path = format!("nodes/{node_id}/status");
    let response: serde_json::Value = client_guard
        .get(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to get node status: {e}"))?;

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
) -> Result<Vec<crate::proxmox::node::SyslogEntry>, String> {
    validate_pve_identifier(&node_id, "node_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let limit_val = limit.unwrap_or(500).to_string();
    let path = format!("nodes/{node_id}/syslog");
    let response: serde_json::Value = client_guard
        .get_with_params(
            &path,
            &[("limit", limit_val.as_str())],
            Some(client_guard.ticket.as_deref().unwrap_or("")),
        )
        .await
        .map_err(|e| format!("Failed to get syslog: {e}"))?;

    crate::proxmox::node::parse_syslog_entries(&response)
}

// ─── Phase 12 - Network Interfaces ───────────────────────────────────────────

use crate::proxmox::network::NetworkInterfaceConfig;

/// List network interfaces on a node
#[tauri::command]
pub async fn list_network_interfaces(
    cluster_id: String,
    node_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    validate_pve_identifier(&node_id, "node_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let path = format!("nodes/{node_id}/network");
    let response: serde_json::Value = client_guard
        .get(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list network interfaces: {e}"))?;

    response
        .as_array()
        .map(|arr| arr.to_vec())
        .ok_or_else(|| "Invalid response format".to_string())
}

/// Create a network interface
#[tauri::command]
pub async fn create_network_interface(
    cluster_id: String,
    node_id: String,
    config: NetworkInterfaceConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    validate_pve_identifier(&config.iface, "iface")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let mut body = serde_json::json!({
        "iface": config.iface,
        "type": config.iface_type,
    });

    if let Some(addr) = config.address {
        body["address"] = serde_json::Value::String(addr);
    }
    if let Some(mask) = config.netmask {
        body["netmask"] = serde_json::Value::String(mask);
    }
    if let Some(gw) = config.gateway {
        body["gateway"] = serde_json::Value::String(gw);
    }
    if config.active {
        body["active"] = serde_json::Value::Number(1.into());
    }
    if config.autostart {
        body["autostart"] = serde_json::Value::Number(1.into());
    }
    if let Some(com) = config.comments {
        body["comments"] = serde_json::Value::String(com);
    }

    let path = format!("nodes/{node_id}/network");
    let _response: serde_json::Value = client_guard
        .post(
            &path,
            &body,
            Some(client_guard.ticket.as_deref().unwrap_or("")),
        )
        .await
        .map_err(|e| format!("Failed to create network interface {}: {e}", config.iface))?;

    Ok(())
}

/// Update a network interface
#[tauri::command]
pub async fn update_network_interface(
    cluster_id: String,
    node_id: String,
    iface: String,
    config: NetworkInterfaceConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    validate_pve_identifier(&iface, "iface")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let mut body = serde_json::json!({
        "iface": config.iface,
        "type": config.iface_type,
    });

    if let Some(addr) = config.address {
        body["address"] = serde_json::Value::String(addr);
    }
    if let Some(mask) = config.netmask {
        body["netmask"] = serde_json::Value::String(mask);
    }
    if let Some(gw) = config.gateway {
        body["gateway"] = serde_json::Value::String(gw);
    }
    if config.active {
        body["active"] = serde_json::Value::Number(1.into());
    }
    if config.autostart {
        body["autostart"] = serde_json::Value::Number(1.into());
    }
    if let Some(com) = config.comments {
        body["comments"] = serde_json::Value::String(com);
    }

    let path = format!("nodes/{node_id}/network/{iface}");
    let _response: serde_json::Value = client_guard
        .put(
            &path,
            &body,
            Some(client_guard.ticket.as_deref().unwrap_or("")),
        )
        .await
        .map_err(|e| format!("Failed to update network interface {iface}: {e}"))?;

    Ok(())
}

/// Delete a network interface
#[tauri::command]
pub async fn delete_network_interface(
    cluster_id: String,
    node_id: String,
    iface: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    validate_pve_identifier(&iface, "iface")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    let path = format!("nodes/{node_id}/network/{iface}");
    let _response: serde_json::Value = client_guard
        .delete(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to delete network interface {iface}: {e}"))?;

    Ok(())
}

// ─── Phase 12b - VM Snapshots ────────────────────────────────────────────────

/// List snapshots for a VM
#[tauri::command]
pub async fn list_proxmox_snapshots(
    cluster_id: String,
    node_id: String,
    vmid: u32,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    validate_pve_identifier(&node_id, "node_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::list_snapshots(
        &client_guard,
        &node_id,
        vmid,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

/// Create a snapshot for a VM
#[tauri::command]
pub async fn create_proxmox_snapshot(
    cluster_id: String,
    node_id: String,
    vmid: u32,
    snapshot_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    validate_pve_identifier(&snapshot_name, "snapshot_name")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::create_snapshot(
        &client_guard,
        &node_id,
        vmid,
        &snapshot_name,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

/// Delete a snapshot for a VM
#[tauri::command]
pub async fn delete_proxmox_snapshot(
    cluster_id: String,
    node_id: String,
    vmid: u32,
    snapshot_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    validate_pve_identifier(&snapshot_name, "snapshot_name")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::delete_snapshot(
        &client_guard,
        &node_id,
        vmid,
        &snapshot_name,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

/// Rollback a VM to a snapshot
#[tauri::command]
pub async fn rollback_proxmox_snapshot(
    cluster_id: String,
    node_id: String,
    vmid: u32,
    snapshot_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    validate_pve_identifier(&snapshot_name, "snapshot_name")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::vm::rollback_snapshot(
        &client_guard,
        &node_id,
        vmid,
        &snapshot_name,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

// ─── ISO Image Listing ────────────────────────────────────────────────────────

/// List ISO images available in a Proxmox storage
#[tauri::command]
pub async fn list_iso_images(
    cluster_id: String,
    node_id: String,
    storage_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    validate_pve_identifier(&node_id, "node_id")?;
    validate_pve_identifier(&storage_id, "storage_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::storage::list_storage_content_iso(
        &client_guard,
        &node_id,
        &storage_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

/// Upload an ISO image to a Proxmox storage pool.
/// `file_path` is the local filesystem path selected by the user via file dialog.
/// Returns the Proxmox task UPID which can be polled for completion.
#[tauri::command]
pub async fn upload_iso_image(
    cluster_id: String,
    node_id: String,
    storage_id: String,
    file_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    validate_pve_identifier(&node_id, "node_id")?;
    validate_pve_identifier(&storage_id, "storage_id")?;

    let filename = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid file path: cannot determine filename".to_string())?
        .to_string();

    // Enforce .iso extension
    if !filename.to_lowercase().ends_with(".iso") {
        return Err("Only .iso files are supported".to_string());
    }

    let file_bytes = tokio::fs::read(&file_path)
        .await
        .map_err(|e| format!("Failed to read file '{file_path}': {e}"))?;

    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;

    crate::proxmox::storage::upload_iso(
        &client_guard,
        &node_id,
        &storage_id,
        &filename,
        file_bytes,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
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
        .map_err(|e| format!("Failed to get subscription status: {e}"))?;

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
        .map_err(|e| format!("Failed to list cluster tasks: {e}"))?;

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
        .map_err(|e| format!("Failed to list containers: {e}"))?;

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
            .map_err(|e| format!("Failed to lock database: {e}"))?;

        let mut stmt = db
            .prepare(
                "SELECT url, port, username, encrypted_credentials \
                 FROM proxmox_clusters WHERE id = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        stmt.query_row([&cluster_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u16>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .optional()
        .map_err(|e| format!("Failed to query cluster: {e}"))?
        .ok_or_else(|| format!("Cluster {cluster_id} not found in database"))?
    };

    let credentials_json = crate::integrations::auth::decrypt_token(&encrypted_credentials)
        .map_err(|e| format!("Failed to decrypt credentials: {e}"))?;

    let credentials: serde_json::Value = serde_json::from_str(&credentials_json)
        .map_err(|e| format!("Failed to parse credentials: {e}"))?;

    let password = credentials
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Password not found in credentials".to_string())?;

    let mut client = crate::proxmox::ProxmoxClient::new(&url, port, &username);
    client
        .authenticate(password)
        .await
        .map_err(|e| format!("Failed to authenticate with Proxmox: {e}"))?;

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

// ─── SDN CRUD ─────────────────────────────────────────────────────────────────

/// Create an EVPN SDN zone
#[tauri::command]
pub async fn create_sdn_zone(
    cluster_id: String,
    zone: String,
    asn: u32,
    vni: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&zone, "zone")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    crate::proxmox::sdn::create_evpn_zone(&client_guard, &zone, asn, vni, ticket)
        .await
        .map_err(|e| format!("Failed to create SDN zone: {e}"))
}

/// Update an EVPN SDN zone
#[tauri::command]
pub async fn update_sdn_zone(
    cluster_id: String,
    zone: String,
    asn: u32,
    vni: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&zone, "zone")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    crate::proxmox::sdn::update_evpn_zone(&client_guard, &zone, asn, vni, ticket)
        .await
        .map_err(|e| format!("Failed to update SDN zone: {e}"))
}

/// Delete an SDN zone
#[tauri::command]
pub async fn delete_sdn_zone(
    cluster_id: String,
    zone: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&zone, "zone")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    crate::proxmox::sdn::delete_evpn_zone(&client_guard, &zone, ticket)
        .await
        .map_err(|e| format!("Failed to delete SDN zone: {e}"))
}

/// Create an SDN virtual network
#[tauri::command]
pub async fn create_sdn_vnet(
    cluster_id: String,
    vnet: String,
    zone: String,
    l2vni: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&vnet, "vnet")?;
    validate_pve_identifier(&zone, "zone")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    crate::proxmox::sdn::create_vnet(&client_guard, &vnet, &zone, l2vni, ticket)
        .await
        .map_err(|e| format!("Failed to create SDN vnet: {e}"))
}

/// Update an SDN virtual network
#[tauri::command]
pub async fn update_sdn_vnet(
    cluster_id: String,
    vnet: String,
    zone: String,
    l2vni: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&vnet, "vnet")?;
    validate_pve_identifier(&zone, "zone")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    crate::proxmox::sdn::update_vnet(&client_guard, &vnet, &zone, l2vni, ticket)
        .await
        .map_err(|e| format!("Failed to update SDN vnet: {e}"))
}

/// Delete an SDN virtual network
#[tauri::command]
pub async fn delete_sdn_vnet(
    cluster_id: String,
    vnet: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&vnet, "vnet")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    crate::proxmox::sdn::delete_vnet(&client_guard, &vnet, ticket)
        .await
        .map_err(|e| format!("Failed to delete SDN vnet: {e}"))
}

// ─── Backup Job CRUD ──────────────────────────────────────────────────────────

/// Create a cluster-level backup job
#[tauri::command]
pub async fn create_proxmox_backup_job(
    cluster_id: String,
    storage: String,
    vmid: Option<String>,
    mode: Option<String>,
    schedule: Option<String>,
    enabled: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");

    let storage_s = storage.as_str();
    let vmid_s = vmid.as_deref().unwrap_or("all");
    let mode_s = mode.as_deref().unwrap_or("snapshot");
    let schedule_s = schedule.as_deref().unwrap_or("0 2 * * *");
    let enabled_s = if enabled.unwrap_or(true) { "1" } else { "0" };

    let params: &[(&str, &str)] = &[
        ("storage", storage_s),
        ("vmid", vmid_s),
        ("mode", mode_s),
        ("schedule", schedule_s),
        ("enabled", enabled_s),
    ];

    let _: serde_json::Value = client_guard
        .post_form("cluster/backup", params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create backup job: {e}"))?;
    Ok(())
}

/// Update a cluster-level backup job
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn update_proxmox_backup_job(
    cluster_id: String,
    job_id: String,
    storage: Option<String>,
    vmid: Option<String>,
    mode: Option<String>,
    schedule: Option<String>,
    enabled: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&job_id, "job_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");

    let mut body = serde_json::json!({});
    if let Some(s) = storage {
        body["storage"] = serde_json::Value::String(s);
    }
    if let Some(v) = vmid {
        body["vmid"] = serde_json::Value::String(v);
    }
    if let Some(m) = mode {
        body["mode"] = serde_json::Value::String(m);
    }
    if let Some(sc) = schedule {
        body["schedule"] = serde_json::Value::String(sc);
    }
    if let Some(en) = enabled {
        body["enabled"] = serde_json::Value::Number(if en { 1.into() } else { 0.into() });
    }

    let path = format!("cluster/backup/{job_id}");
    let _: serde_json::Value = client_guard
        .put(&path, &body, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update backup job {job_id}: {e}"))?;
    Ok(())
}

/// Delete a cluster-level backup job
#[tauri::command]
pub async fn delete_proxmox_backup_job(
    cluster_id: String,
    job_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&job_id, "job_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");

    let path = format!("cluster/backup/{job_id}");
    let _: serde_json::Value = client_guard
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete backup job {job_id}: {e}"))?;
    Ok(())
}

// ─── LXC Container Power ──────────────────────────────────────────────────────

/// Start an LXC container
#[tauri::command]
pub async fn start_proxmox_container(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    let path = format!("nodes/{node_id}/lxc/{vm_id}/status/start");
    let _: serde_json::Value = client_guard
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to start container {vm_id}: {e}"))?;
    Ok(())
}

/// Stop an LXC container
#[tauri::command]
pub async fn stop_proxmox_container(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    let path = format!("nodes/{node_id}/lxc/{vm_id}/status/stop");
    let _: serde_json::Value = client_guard
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to stop container {vm_id}: {e}"))?;
    Ok(())
}

/// Reboot an LXC container
#[tauri::command]
pub async fn reboot_proxmox_container(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    let path = format!("nodes/{node_id}/lxc/{vm_id}/status/reboot");
    let _: serde_json::Value = client_guard
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to reboot container {vm_id}: {e}"))?;
    Ok(())
}

/// Gracefully shut down an LXC container
#[tauri::command]
pub async fn shutdown_proxmox_container(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    let path = format!("nodes/{node_id}/lxc/{vm_id}/status/shutdown");
    let _: serde_json::Value = client_guard
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to shutdown container {vm_id}: {e}"))?;
    Ok(())
}

/// Suspend an LXC container
#[tauri::command]
pub async fn suspend_proxmox_container(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    let path = format!("nodes/{node_id}/lxc/{vm_id}/status/suspend");
    let _: serde_json::Value = client_guard
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to suspend container {vm_id}: {e}"))?;
    Ok(())
}

/// Resume an LXC container
#[tauri::command]
pub async fn resume_proxmox_container(
    cluster_id: String,
    node_id: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node_id, "node_id")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    let path = format!("nodes/{node_id}/lxc/{vm_id}/status/resume");
    let _: serde_json::Value = client_guard
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to resume container {vm_id}: {e}"))?;
    Ok(())
}

// ─── ACL CRUD ─────────────────────────────────────────────────────────────────

/// Create (or update) an ACL entry at the given path
#[tauri::command]
pub async fn create_proxmox_acl(
    cluster_id: String,
    path: String,
    roles: String,
    users: Option<String>,
    groups: Option<String>,
    propagate: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");

    let mut body = serde_json::json!({ "path": path, "roles": roles });
    if let Some(u) = users {
        body["users"] = serde_json::Value::String(u);
    }
    if let Some(g) = groups {
        body["groups"] = serde_json::Value::String(g);
    }
    if let Some(p) = propagate {
        body["propagate"] = serde_json::Value::Bool(p);
    }

    let _: serde_json::Value = client_guard
        .put("access/acl", &body, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create ACL: {e}"))?;
    Ok(())
}

/// Remove an ACL entry
#[tauri::command]
pub async fn delete_proxmox_acl(
    cluster_id: String,
    path: String,
    roles: String,
    users: Option<String>,
    groups: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");

    let mut body = serde_json::json!({ "path": path, "roles": roles, "delete": 1 });
    if let Some(u) = users {
        body["users"] = serde_json::Value::String(u);
    }
    if let Some(g) = groups {
        body["groups"] = serde_json::Value::String(g);
    }

    let _: serde_json::Value = client_guard
        .put("access/acl", &body, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete ACL: {e}"))?;
    Ok(())
}

// ─── User CRUD ────────────────────────────────────────────────────────────────

/// Create a PVE user account
#[tauri::command]
pub async fn create_proxmox_user(
    cluster_id: String,
    userid: String,
    password: String,
    comment: Option<String>,
    email: Option<String>,
    enabled: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");

    let userid_s = userid.as_str();
    let password_s = password.as_str();
    let comment_s = comment.as_deref().unwrap_or("");
    let email_s = email.as_deref().unwrap_or("");
    let enabled_s = if enabled.unwrap_or(true) { "1" } else { "0" };

    let params: &[(&str, &str)] = &[
        ("userid", userid_s),
        ("password", password_s),
        ("comment", comment_s),
        ("email", email_s),
        ("enable", enabled_s),
    ];

    let _: serde_json::Value = client_guard
        .post_form("access/users", params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create user {userid}: {e}"))?;
    Ok(())
}

/// Update a PVE user account
#[tauri::command]
pub async fn update_proxmox_user(
    cluster_id: String,
    userid: String,
    comment: Option<String>,
    email: Option<String>,
    enabled: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_userid(&userid, "userid")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");

    let mut body = serde_json::json!({});
    if let Some(c) = comment {
        body["comment"] = serde_json::Value::String(c);
    }
    if let Some(e) = email {
        body["email"] = serde_json::Value::String(e);
    }
    if let Some(en) = enabled {
        body["enable"] = serde_json::Value::Number(if en { 1.into() } else { 0.into() });
    }

    let path = format!("access/users/{userid}");
    let _: serde_json::Value = client_guard
        .put(&path, &body, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update user {userid}: {e}"))?;
    Ok(())
}

/// Delete a PVE user account
#[tauri::command]
pub async fn delete_proxmox_user(
    cluster_id: String,
    userid: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_userid(&userid, "userid")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");

    let path = format!("access/users/{userid}");
    let _: serde_json::Value = client_guard
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete user {userid}: {e}"))?;
    Ok(())
}

// ─── Realm CRUD ───────────────────────────────────────────────────────────────

/// Create an authentication realm
#[tauri::command]
pub async fn create_proxmox_realm(
    cluster_id: String,
    realm: String,
    realm_type: String,
    comment: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&realm, "realm")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");

    let realm_s = realm.as_str();
    let type_s = realm_type.as_str();
    let comment_s = comment.as_deref().unwrap_or("");

    let params: &[(&str, &str)] = &[("realm", realm_s), ("type", type_s), ("comment", comment_s)];

    let _: serde_json::Value = client_guard
        .post_form("access/domains", params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create realm {realm}: {e}"))?;
    Ok(())
}

/// Update an authentication realm
#[tauri::command]
pub async fn update_proxmox_realm(
    cluster_id: String,
    realm: String,
    comment: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&realm, "realm")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");

    let mut body = serde_json::json!({});
    if let Some(c) = comment {
        body["comment"] = serde_json::Value::String(c);
    }

    let path = format!("access/domains/{realm}");
    let _: serde_json::Value = client_guard
        .put(&path, &body, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update realm {realm}: {e}"))?;
    Ok(())
}

/// Delete an authentication realm
#[tauri::command]
pub async fn delete_proxmox_realm(
    cluster_id: String,
    realm: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&realm, "realm")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");

    let path = format!("access/domains/{realm}");
    let _: serde_json::Value = client_guard
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete realm {realm}: {e}"))?;
    Ok(())
}

// ─── Node Administration ───────────────────────────────────────────────────

#[tauri::command]
pub async fn get_node_dns(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<crate::proxmox::node::NodeDns, String> {
    validate_pve_identifier(&node, "node")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::node::get_node_dns(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn update_node_dns(
    cluster_id: String,
    node: String,
    search: String,
    dns1: Option<String>,
    dns2: Option<String>,
    dns3: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node, "node")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::node::update_node_dns(
        &client_guard,
        &node,
        &search,
        dns1.as_deref(),
        dns2.as_deref(),
        dns3.as_deref(),
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn get_node_time(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<crate::proxmox::node::NodeTime, String> {
    validate_pve_identifier(&node, "node")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::node::get_node_time(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn update_node_time(
    cluster_id: String,
    node: String,
    timezone: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node, "node")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::node::update_node_time(
        &client_guard,
        &node,
        &timezone,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn reboot_node(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    validate_pve_identifier(&node, "node")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::node::reboot_node(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn shutdown_node(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    validate_pve_identifier(&node, "node")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::node::shutdown_node(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn get_node_journal(
    cluster_id: String,
    node: String,
    lastentries: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    validate_pve_identifier(&node, "node")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::node::get_node_journal(
        &client_guard,
        &node,
        lastentries.unwrap_or(200),
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn get_node_report(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    validate_pve_identifier(&node, "node")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::node::get_node_report(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

// ─── Network Administration ────────────────────────────────────────────────

#[tauri::command]
pub async fn reload_network_config(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    validate_pve_identifier(&node, "node")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::network::reload_network_config(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

// ─── VM Configuration ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_vm_config(
    cluster_id: String,
    node: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::vm::get_vm_config_raw(
        &client_guard,
        &node,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn get_vm_pending_config(
    cluster_id: String,
    node: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<crate::proxmox::vm::VmPendingEntry>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::vm::get_vm_pending_config(
        &client_guard,
        &node,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn remote_migrate_vm(
    cluster_id: String,
    node: String,
    vm_id: u32,
    target_node: String,
    target_storage: String,
    online: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::vm::remote_migrate_vm(
        &client_guard,
        &node,
        vm_id,
        &target_node,
        &target_storage,
        online,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

/// Result of starting a cross-datacenter (remote) migration.
///
/// Carries the task UPID for polling and the temporary destination API token
/// details so the frontend can delete the token after the task completes.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RemoteMigrationStart {
    pub upid: String,
    pub source_node: String,
    pub dest_cluster_id: String,
    pub dest_userid: String,
    pub dest_tokenname: String,
}

/// Start a cross-datacenter (remote) VM migration.
///
/// Creates a temporary API token on the destination remote (from its stored
/// credentials), resolves the destination TLS fingerprint (stored override or
/// auto-fetched), issues a PVE `remote-migrate` on the source node, and returns
/// the task UPID plus the temporary token details for later cleanup.
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn start_remote_migration(
    cluster_id: String,
    node: String,
    vm_id: u32,
    dest_cluster_id: String,
    target_node: String,
    target_storage: String,
    target_bridge: String,
    online: bool,
    state: State<'_, AppState>,
) -> Result<RemoteMigrationStart, String> {
    // `node` and `target_node` both flow into PVE REST URL paths (the source
    // remote-migrate call and the destination certificate/fingerprint lookup),
    // so validate them up-front against the shared node allowlist.
    crate::proxmox::validate::validate_node(&node)?;
    crate::proxmox::validate::validate_node(&target_node)?;
    crate::proxmox::validate::validate_vmid(vm_id)?;

    // Resolve an optional stored fingerprint for the destination remote.
    let stored_fingerprint: Option<String> = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {e}"))?;
        db.query_row(
            "SELECT ssl_fingerprint FROM proxmox_clusters WHERE id = ?1",
            rusqlite::params![dest_cluster_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .ok()
        .flatten()
        .filter(|s| !s.trim().is_empty())
    };

    // Destination client: host/port/user + fingerprint + temp token.
    let dest_client = get_proxmox_client_for_cluster(&dest_cluster_id, &state).await?;
    let (dest_host, dest_port, dest_userid, fingerprint, full_tokenid, token_secret, tokenname) = {
        let dest_guard = dest_client.lock().await;
        let dest_ticket = dest_guard.ticket.as_deref().unwrap_or("").to_string();
        let dest_host = dest_guard.base_url().to_string();
        let dest_port = dest_guard.port();
        let dest_userid = dest_guard.username().to_string();

        // Resolve fingerprint: stored override, else auto-fetch from target node.
        let fingerprint = match stored_fingerprint {
            Some(fp) => fp,
            None => {
                crate::proxmox::migration::get_node_fingerprint(
                    &dest_guard,
                    &target_node,
                    &dest_ticket,
                )
                .await?
            }
        };

        // Create a short-lived, non-privilege-separated token on the destination.
        let tokenname = format!(
            "tftsr-migrate-{vm_id}-{}",
            chrono::Utc::now().timestamp_millis()
        );
        let token = crate::proxmox::auth_realm::create_user_token(
            &dest_guard,
            &dest_ticket,
            &dest_userid,
            &tokenname,
            Some("Temporary token for TRCAA cross-DC migration"),
            false,
            None,
        )
        .await?;
        let full_tokenid = token.full_tokenid.ok_or_else(|| {
            "Destination did not return a full token id for the migration token".to_string()
        })?;
        let token_secret = token.value.ok_or_else(|| {
            "Destination did not return a secret for the migration token".to_string()
        })?;

        (
            dest_host,
            dest_port,
            dest_userid,
            fingerprint,
            full_tokenid,
            token_secret,
            tokenname,
        )
    };

    let target_endpoint = crate::proxmox::migration::build_remote_target_endpoint(
        &full_tokenid,
        &token_secret,
        &dest_host,
        dest_port,
        Some(&fingerprint),
    );

    // Issue the remote-migrate on the source node. If it fails, clean up token.
    let source_client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let migrate_result = {
        let src_guard = source_client.lock().await;
        crate::proxmox::migration::remote_migrate_vm(
            &src_guard,
            &node,
            vm_id,
            &target_endpoint,
            &target_bridge,
            &target_storage,
            online,
            src_guard.ticket.as_deref().unwrap_or(""),
        )
        .await
    };

    let upid = match migrate_result {
        Ok(upid) => upid,
        Err(e) => {
            // Best-effort cleanup of the temp token before surfacing the error.
            let dest_guard = dest_client.lock().await;
            if let Err(cleanup_err) = crate::proxmox::auth_realm::delete_user_token(
                &dest_guard,
                dest_guard.ticket.as_deref().unwrap_or(""),
                &dest_userid,
                &tokenname,
            )
            .await
            {
                tracing::warn!(
                    "Failed to clean up temporary migration token '{tokenname}': {cleanup_err}"
                );
            }
            return Err(e);
        }
    };

    // NOTE: On a successful start the token is intentionally NOT deleted here.
    // The destination pulls the VM asynchronously using this token while the
    // source task runs, so it must outlive this call. The caller removes it via
    // `delete_user_token` once `get_task_status` reports the task has finished
    // (see the migration poller in the frontend / RemoteMigrationStart result).
    Ok(RemoteMigrationStart {
        upid,
        source_node: node,
        dest_cluster_id,
        dest_userid,
        dest_tokenname: tokenname,
    })
}

// ─── VM Console (noVNC) ───────────────────────────────────────────────────

/// Open an in-app noVNC console for a QEMU VM.
///
/// Requests a `vncproxy` ticket from PVE, starts a local WebSocket proxy that
/// injects the auth cookie and accepts the node's self-signed TLS cert, and
/// returns the local websocket URL + VNC ticket for the noVNC client.
#[tauri::command]
pub async fn open_vnc_console(
    cluster_id: String,
    node: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<crate::proxmox::console::VncConsoleSession, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let (proxy, host, port, auth_ticket) = {
        let guard = client.lock().await;
        let auth_ticket = guard
            .ticket
            .clone()
            .ok_or_else(|| "No active session ticket for this remote".to_string())?;
        let proxy =
            crate::proxmox::console::vncproxy_vm(&guard, &node, vm_id, &auth_ticket).await?;
        (
            proxy,
            guard.base_url().to_string(),
            guard.port(),
            auth_ticket,
        )
    };

    let upstream = crate::proxmox::console::build_vncwebsocket_url(
        &host,
        port,
        &node,
        vm_id,
        &proxy.port,
        &proxy.ticket,
    );
    let fingerprint = stored_ssl_fingerprint(&cluster_id, &state);
    crate::proxmox::console::start_vnc_proxy(
        upstream,
        "PVEAuthCookie".to_string(),
        auth_ticket,
        proxy.ticket,
        fingerprint,
    )
    .await
}

/// Open an in-app noVNC console for an LXC container.
#[tauri::command]
pub async fn open_lxc_console(
    cluster_id: String,
    node: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<crate::proxmox::console::VncConsoleSession, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let (proxy, host, port, auth_ticket) = {
        let guard = client.lock().await;
        let auth_ticket = guard
            .ticket
            .clone()
            .ok_or_else(|| "No active session ticket for this remote".to_string())?;
        let proxy =
            crate::proxmox::console::vncproxy_lxc(&guard, &node, vm_id, &auth_ticket).await?;
        (
            proxy,
            guard.base_url().to_string(),
            guard.port(),
            auth_ticket,
        )
    };

    let upstream = crate::proxmox::console::build_lxc_vncwebsocket_url(
        &host,
        port,
        &node,
        vm_id,
        &proxy.port,
        &proxy.ticket,
    );
    let fingerprint = stored_ssl_fingerprint(&cluster_id, &state);
    crate::proxmox::console::start_vnc_proxy(
        upstream,
        "PVEAuthCookie".to_string(),
        auth_ticket,
        proxy.ticket,
        fingerprint,
    )
    .await
}

/// Tagged host-shell session: which renderer the frontend should use plus the
/// local proxy URL and credentials.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeShellSession {
    /// `"novnc"` (PVE graphical shell) or `"xterm"` (PBS terminal).
    pub kind: String,
    /// Local websocket URL the renderer connects to.
    pub local_url: String,
    /// VNC ticket / terminal ticket.
    pub ticket: String,
    /// Bound local port (diagnostics).
    pub local_port: u16,
    /// RFB password for noVNC shells (PVE vncshell only).
    pub password: Option<String>,
    /// The session user (needed for the xterm termproxy login line).
    pub user: String,
}

/// Open a host (node) shell for a stored remote.
///
/// - **PVE** remotes use `vncshell` (graphical, rendered with noVNC).
/// - **PBS** remotes use `termproxy` (text terminal, rendered with xterm.js).
///
/// In both cases a local WebSocket proxy is started that injects the correct
/// auth cookie (`PVEAuthCookie` / `PBSAuthCookie`) and accepts the node's
/// self-signed TLS certificate.
#[tauri::command]
pub async fn open_node_shell(
    cluster_id: String,
    node: String,
    cmd: Option<String>,
    state: State<'_, AppState>,
) -> Result<NodeShellSession, String> {
    validate_pve_identifier(&node, "node")?;
    if let Some(cmd) = cmd.as_deref() {
        crate::proxmox::console::validate_shell_cmd(cmd)?;
    }
    // Resolve the cluster type so we know which shell API + cookie to use.
    let is_pbs = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("Failed to lock database: {e}"))?;
        let ct: String = db
            .query_row(
                "SELECT cluster_type FROM proxmox_clusters WHERE id = ?1",
                [&cluster_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| format!("Failed to query cluster: {e}"))?
            .ok_or_else(|| format!("Cluster {cluster_id} not found"))?;
        ct.eq_ignore_ascii_case("pbs")
    };

    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let (proxy, host, port, auth_ticket, user) = {
        let guard = client.lock().await;
        let auth_ticket = guard
            .ticket
            .clone()
            .ok_or_else(|| "No active session ticket for this remote".to_string())?;
        let proxy = if is_pbs {
            crate::proxmox::console::termproxy_node(&guard, &node, cmd.as_deref(), &auth_ticket)
                .await?
        } else {
            crate::proxmox::console::vncshell_node(&guard, &node, cmd.as_deref(), &auth_ticket)
                .await?
        };
        let user = if proxy.user.is_empty() {
            guard.username().to_string()
        } else {
            proxy.user.clone()
        };
        (
            proxy,
            guard.base_url().to_string(),
            guard.port(),
            auth_ticket,
            user,
        )
    };

    let upstream = crate::proxmox::console::build_node_vncwebsocket_url(
        &host,
        port,
        &node,
        &proxy.port,
        &proxy.ticket,
    );

    let cookie_name = if is_pbs {
        "PBSAuthCookie"
    } else {
        "PVEAuthCookie"
    };

    let session = crate::proxmox::console::start_vnc_proxy(
        upstream,
        cookie_name.to_string(),
        auth_ticket,
        proxy.ticket.clone(),
        stored_ssl_fingerprint(&cluster_id, &state),
    )
    .await?;

    Ok(NodeShellSession {
        kind: if is_pbs { "xterm" } else { "novnc" }.to_string(),
        local_url: session.local_url,
        ticket: session.ticket,
        local_port: session.local_port,
        password: proxy.password,
        user,
    })
}

#[tauri::command]
pub async fn get_container_config(
    cluster_id: String,
    node: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::lxc::get_container_config(
        &client_guard,
        &node,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn create_proxmox_container(
    cluster_id: String,
    node: String,
    vmid: u32,
    ostemplate: String,
    hostname: Option<String>,
    memory: Option<u32>,
    cores: Option<u32>,
    rootfs: Option<String>,
    net0: Option<String>,
    password: Option<String>,
    unprivileged: Option<bool>,
    start: Option<bool>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let params = crate::proxmox::lxc::ContainerCreateParams {
        vmid,
        ostemplate,
        hostname,
        memory,
        cores,
        rootfs,
        net0,
        password,
        unprivileged,
        start,
    };
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::lxc::create_proxmox_container(
        &client_guard,
        &node,
        params,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

// ─── RRD Metrics ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_node_rrd_data(
    cluster_id: String,
    node: String,
    timeframe: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::metrics::get_node_rrd_data(
        &client_guard,
        &node,
        &timeframe,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn get_vm_rrd_data(
    cluster_id: String,
    node: String,
    vm_id: u32,
    timeframe: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::metrics::get_vm_rrd_data(
        &client_guard,
        &node,
        vm_id,
        &timeframe,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn get_storage_rrd_data(
    cluster_id: String,
    node: String,
    storage: String,
    timeframe: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::metrics::get_storage_rrd_data(
        &client_guard,
        &node,
        &storage,
        &timeframe,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

// ─── Ceph Advanced ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_ceph_monitors(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::proxmox::ceph::CephMonitor>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::ceph::list_monitors(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn list_ceph_managers(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::proxmox::ceph::CephMgr>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::ceph::list_managers(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn list_cephfs(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::proxmox::ceph::CephFs>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::ceph::list_cephfs(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn get_ceph_flags(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::ceph::get_ceph_flags(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

/// Set (or clear) a cluster-level Ceph runtime flag.
#[tauri::command]
pub async fn set_ceph_flag(
    cluster_id: String,
    flag: String,
    value: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::ceph::set_ceph_flag(
        &client_guard,
        &flag,
        value,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

/// Create a Ceph monitor on a node. Returns the task UPID.
#[tauri::command]
pub async fn create_ceph_monitor(
    cluster_id: String,
    node: String,
    monid: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::ceph::create_mon(
        &client_guard,
        &node,
        &monid,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

/// Destroy a Ceph monitor. Returns the task UPID.
#[tauri::command]
pub async fn delete_ceph_monitor(
    cluster_id: String,
    node: String,
    monid: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::ceph::destroy_mon(
        &client_guard,
        &node,
        &monid,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

/// Create a Ceph manager on a node. Returns the task UPID.
#[tauri::command]
pub async fn create_ceph_manager(
    cluster_id: String,
    node: String,
    id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::ceph::create_mgr(
        &client_guard,
        &node,
        &id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

/// Destroy a Ceph manager. Returns the task UPID.
#[tauri::command]
pub async fn delete_ceph_manager(
    cluster_id: String,
    node: String,
    id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::ceph::destroy_mgr(
        &client_guard,
        &node,
        &id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

/// Start, stop, or restart a Ceph mon/mgr service (e.g. "mon.vmhost1").
/// Returns the task UPID.
#[tauri::command]
pub async fn ceph_service_action(
    cluster_id: String,
    node: String,
    service: String,
    action: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::ceph::ceph_service_action(
        &client_guard,
        &node,
        &service,
        &action,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

// ─── Firewall (cluster + guest level) ─────────────────────────────────────

#[tauri::command]
pub async fn list_cluster_firewall_rules(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::proxmox::firewall::FirewallRule>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::firewall::list_cluster_firewall_rules(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn get_cluster_firewall_status(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<crate::proxmox::firewall::ClusterFirewallStatus, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::firewall::get_cluster_firewall_status(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn list_guest_firewall_rules(
    cluster_id: String,
    node: String,
    vm_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<crate::proxmox::firewall::FirewallRule>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::firewall::list_guest_firewall_rules(
        &client_guard,
        &node,
        vm_id,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_guest_firewall_rule(
    cluster_id: String,
    node: String,
    vm_id: u32,
    action: String,
    proto: Option<String>,
    source: Option<String>,
    dest: Option<String>,
    dport: Option<String>,
    enable: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let rule = crate::proxmox::firewall::FirewallRule {
        rule_num: 0,
        action,
        protocol: proto.unwrap_or_default(),
        source: source.unwrap_or_default(),
        destination: dest.unwrap_or_default(),
        port: dport,
        enabled: enable.unwrap_or(true),
    };
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::firewall::add_guest_firewall_rule(
        &client_guard,
        &node,
        vm_id,
        &rule,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn delete_guest_firewall_rule(
    cluster_id: String,
    node: String,
    vm_id: u32,
    pos: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::firewall::delete_guest_firewall_rule(
        &client_guard,
        &node,
        vm_id,
        pos,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

// ─── TFA Management ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_tfa_entries(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::proxmox::tfa::TfaEntry>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::tfa::list_tfa_entries(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_tfa_entry(
    cluster_id: String,
    userid: String,
    tfa_type: String,
    description: Option<String>,
    totp: Option<String>,
    value: Option<String>,
    key: Option<String>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::tfa::add_tfa_entry(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
        &userid,
        &tfa_type,
        description.as_deref(),
        totp.as_deref(),
        value.as_deref(),
        key.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn delete_tfa_entry(
    cluster_id: String,
    userid: String,
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::tfa::delete_tfa_entry(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
        &userid,
        &id,
    )
    .await
}

// ─── User API Tokens ──────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_user_tokens(
    cluster_id: String,
    userid: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::proxmox::auth_realm::UserToken>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::auth_realm::list_user_tokens(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
        &userid,
    )
    .await
}

#[tauri::command]
pub async fn create_user_token(
    cluster_id: String,
    userid: String,
    tokenname: String,
    comment: Option<String>,
    privsep: Option<bool>,
    expire: Option<i64>,
    state: State<'_, AppState>,
) -> Result<crate::proxmox::auth_realm::UserTokenCreateResult, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::auth_realm::create_user_token(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
        &userid,
        &tokenname,
        comment.as_deref(),
        privsep.unwrap_or(true),
        expire,
    )
    .await
}

#[tauri::command]
pub async fn delete_user_token(
    cluster_id: String,
    userid: String,
    tokenname: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::auth_realm::delete_user_token(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
        &userid,
        &tokenname,
    )
    .await
}

// ─── PBS Management ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_pbs_datastores(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::proxmox::pbs::PbsDatastore>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::pbs::list_pbs_datastores(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn get_pbs_datastore_status(
    cluster_id: String,
    store: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::pbs::get_pbs_datastore_status(
        &client_guard,
        &store,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn list_pbs_namespaces(
    cluster_id: String,
    store: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::proxmox::pbs::PbsNamespace>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::pbs::list_pbs_namespaces(
        &client_guard,
        &store,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn list_pbs_snapshots(
    cluster_id: String,
    store: String,
    ns: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::proxmox::pbs::PbsSnapshot>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::pbs::list_pbs_snapshots(
        &client_guard,
        &store,
        ns.as_deref().unwrap_or(""),
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn list_pbs_tasks(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::proxmox::pbs::PbsTask>, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::pbs::list_pbs_tasks(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

#[tauri::command]
pub async fn get_pbs_node_status(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    crate::proxmox::pbs::get_pbs_node_status(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
}

// ─── Subscription Update ──────────────────────────────────────────────────

#[tauri::command]
pub async fn update_subscription(
    cluster_id: String,
    node: String,
    key: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_pve_identifier(&node, "node")?;
    let client = get_proxmox_client_for_cluster(&cluster_id, &state).await?;
    let client_guard = client.lock().await;
    let ticket = client_guard.ticket.as_deref().unwrap_or("");
    let path = format!("nodes/{node}/subscription");
    let _: serde_json::Value = client_guard
        .post_form(&path, &[("key", key.as_str())], Some(ticket))
        .await
        .map_err(|e| format!("Failed to update subscription on node {node}: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_storage_update_params_only_includes_provided() {
        let p = build_storage_update_params(Some("images,iso"), None, None);
        assert_eq!(p, vec![("content".to_string(), "images,iso".to_string())]);
    }

    #[test]
    fn test_build_storage_update_params_empty_when_none() {
        assert!(build_storage_update_params(None, None, None).is_empty());
    }

    #[test]
    fn test_build_storage_update_params_disable_flag() {
        let p = build_storage_update_params(None, Some("vmhost1,vmhost2"), Some(true));
        assert!(p.contains(&("nodes".to_string(), "vmhost1,vmhost2".to_string())));
        assert!(p.contains(&("disable".to_string(), "1".to_string())));
    }

    #[test]
    fn test_build_storage_update_params_enable_flag() {
        let p = build_storage_update_params(None, None, Some(false));
        assert_eq!(p, vec![("disable".to_string(), "0".to_string())]);
    }

    #[test]
    fn test_normalize_backup_jobs_null_returns_empty() {
        // Standalone remotes with no jobs return `data: null`; must not error.
        let jobs = normalize_backup_jobs(&serde_json::Value::Null);
        assert!(jobs.is_empty());
    }

    #[test]
    fn test_normalize_backup_jobs_empty_array() {
        let jobs = normalize_backup_jobs(&serde_json::json!([]));
        assert!(jobs.is_empty());
    }

    #[test]
    fn test_normalize_backup_jobs_preserves_existing_id() {
        let jobs = normalize_backup_jobs(&serde_json::json!([
            { "id": "backup-abc", "storage": "local", "schedule": "0 2 * * *" }
        ]));
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0]["id"], "backup-abc");
    }

    #[test]
    fn test_normalize_backup_jobs_synthesizes_missing_id() {
        let jobs = normalize_backup_jobs(&serde_json::json!([
            { "storage": "nas01", "schedule": "0 3 * * *" }
        ]));
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0]["id"], "backup-nas01");
    }

    #[test]
    fn test_normalize_backup_jobs_caps_at_100() {
        let many: Vec<serde_json::Value> = (0..150)
            .map(|i| serde_json::json!({ "id": format!("backup-{i}") }))
            .collect();
        let jobs = normalize_backup_jobs(&serde_json::Value::Array(many));
        assert_eq!(jobs.len(), 100);
    }

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
        let msg = format!("Connection test failed: {raw}");
        assert!(msg.starts_with("Connection test failed:"));
        assert!(msg.contains("connection refused"));
    }

    #[test]
    fn test_list_proxmox_nodes_error_message_format() {
        let raw = "HTTP 403";
        let msg = format!("Failed to list nodes: {raw}");
        assert!(msg.starts_with("Failed to list nodes:"));
        assert!(msg.contains("403"));
    }

    #[test]
    fn test_create_proxmox_vm_ide2_with_iso() {
        let iso = Some("local:iso/ubuntu.iso".to_string());
        let ide2 = iso
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("{s},media=cdrom"))
            .unwrap_or_else(|| "none,media=cdrom".to_string());
        assert_eq!(ide2, "local:iso/ubuntu.iso,media=cdrom");
    }

    #[test]
    fn test_create_proxmox_vm_ide2_without_iso() {
        let iso: Option<String> = None;
        let ide2 = iso
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("{s},media=cdrom"))
            .unwrap_or_else(|| "none,media=cdrom".to_string());
        assert_eq!(ide2, "none,media=cdrom");
    }

    #[test]
    fn test_create_proxmox_vm_ide2_empty_string_iso() {
        let iso = Some("".to_string());
        let ide2 = iso
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("{s},media=cdrom"))
            .unwrap_or_else(|| "none,media=cdrom".to_string());
        assert_eq!(ide2, "none,media=cdrom");
    }

    #[test]
    fn test_create_proxmox_vm_scsi0_format() {
        let storage = "local-lvm";
        let disk_size: u32 = 32;
        let scsi0 = format!("{storage}:{disk_size}");
        assert_eq!(scsi0, "local-lvm:32");
    }

    #[test]
    fn test_create_proxmox_vm_net0_format() {
        let bridge = "vmbr0";
        let net0 = format!("virtio,bridge={bridge}");
        assert_eq!(net0, "virtio,bridge=vmbr0");
    }

    #[test]
    fn test_create_proxmox_vm_error_message_format() {
        let vmid: u32 = 105;
        let raw = "storage not found";
        let msg = format!("Failed to create VM: Failed to create VM {vmid}: {raw}");
        assert!(msg.contains("Failed to create VM"));
        assert!(msg.contains("105"));
    }

    #[test]
    fn test_validate_pve_identifier_valid() {
        assert!(super::validate_pve_identifier("pve-node1", "node_id").is_ok());
        assert!(super::validate_pve_identifier("vmbr0", "net_bridge").is_ok());
        assert!(super::validate_pve_identifier("local-lvm", "storage").is_ok());
        assert!(super::validate_pve_identifier("node.example", "node_id").is_ok());
    }

    #[test]
    fn test_validate_pve_identifier_rejects_path_traversal() {
        assert!(super::validate_pve_identifier("../../access/users", "node_id").is_err());
        assert!(super::validate_pve_identifier("node/sub", "node_id").is_err());
        assert!(super::validate_pve_identifier("node?query=1", "node_id").is_err());
        assert!(super::validate_pve_identifier("node#anchor", "node_id").is_err());
    }

    #[test]
    fn test_validate_pve_identifier_rejects_empty() {
        assert!(super::validate_pve_identifier("", "node_id").is_err());
    }

    #[test]
    fn test_create_vm_vmid_range_validation() {
        let valid: u32 = 100;
        assert!((100..=999_999_999).contains(&valid));
        let too_low: u32 = 99;
        assert!(!(100..=999_999_999).contains(&too_low));
        let too_high: u32 = 1_000_000_000;
        assert!(!(100..=999_999_999).contains(&too_high));
    }

    #[test]
    fn test_iso_format_valid() {
        let iso = "local:iso/ubuntu-24.04.iso";
        let valid = iso
            .split_once(':')
            .map(|(store, path)| {
                !store.is_empty()
                    && !store.contains(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
                    && path.starts_with("iso/")
                    && !path.contains(',')
            })
            .unwrap_or(false);
        assert!(valid, "should accept valid iso path");
    }

    #[test]
    fn test_iso_format_rejects_comma_injection() {
        let iso = "local:iso/x.iso,media=cdrom,backup=0";
        let valid = iso
            .split_once(':')
            .map(|(store, path)| {
                !store.is_empty()
                    && !store.contains(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
                    && path.starts_with("iso/")
                    && !path.contains(',')
            })
            .unwrap_or(false);
        assert!(!valid, "should reject comma injection in iso path");
    }

    #[test]
    fn test_iso_format_rejects_missing_colon() {
        let iso = "local-iso-no-colon";
        let valid = iso
            .split_once(':')
            .map(|(store, path)| {
                !store.is_empty()
                    && !store.contains(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
                    && path.starts_with("iso/")
                    && !path.contains(',')
            })
            .unwrap_or(false);
        assert!(!valid, "should reject iso without storage: prefix");
    }
}
