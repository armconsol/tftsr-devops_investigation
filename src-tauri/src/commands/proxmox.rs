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
    // Create client (no live auth — credentials stored and used on first connect)
    let client = ProxmoxClient::new(&connection.url, connection.port, &username);

    // Encrypt raw password for storage; auth happens lazily on first API call
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

    // Store in memory connection pool (unauthenticated; ticket set on first use)
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

/// List all Proxmox VMs
#[tauri::command]
pub async fn list_proxmox_vms(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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

/// List Proxmox Backup Jobs
#[tauri::command]
pub async fn list_proxmox_backup_jobs(
    cluster_id: String,
    node: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let jobs = crate::proxmox::backup::list_backup_jobs(
        &client_guard,
        &node,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list backup jobs: {}", e))?;

    let json_jobs: Vec<serde_json::Value> = jobs
        .into_iter()
        .map(|job| {
            serde_json::to_value(job).map_err(|e| format!("Failed to serialize backup job: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_jobs)
}

/// List Proxmox Datastores
#[tauri::command]
pub async fn list_proxmox_datastores(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let datastores = crate::proxmox::backup::list_datastores(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list datastores: {}", e))?;

    let json_datastores: Vec<serde_json::Value> = datastores
        .into_iter()
        .map(|ds| {
            serde_json::to_value(ds).map_err(|e| format!("Failed to serialize datastore: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json_datastores)
}

/// Trigger Proxmox Backup Job
#[tauri::command]
pub async fn trigger_proxmox_backup_job(
    cluster_id: String,
    node: String,
    job_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let views = crate::proxmox::views::list_views(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list views: {}", e))?;

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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let certs = crate::proxmox::certificates::list_certificates(
        &client_guard,
        client_guard.ticket.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| format!("Failed to list certificates: {}", e))?;

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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let path = "access/acl";
    let response: serde_json::Value = client_guard
        .get(path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list ACLs: {}", e))?;

    response
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| arr.to_vec())
        .ok_or_else(|| "Invalid response format".to_string())
}

/// List users
#[tauri::command]
pub async fn list_users(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let path = "access/users";
    let response: serde_json::Value = client_guard
        .get(path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list users: {}", e))?;

    response
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| arr.to_vec())
        .ok_or_else(|| "Invalid response format".to_string())
}

/// List authentication realms (typed)
#[tauri::command]
pub async fn list_realms(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let path = "cluster/config";
    let response: serde_json::Value = client_guard
        .get(path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to get cluster notes: {}", e))?;

    Ok(response
        .get("data")
        .and_then(|d| d.get("notes"))
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let path = format!("cluster/resources?type=vm&search={}", query);
    let response: serde_json::Value = client_guard
        .get(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to search resources: {}", e))?;

    response
        .get("data")
        .and_then(|d| d.as_array())
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let path = format!("nodes/{}/status", node_id);
    let response: serde_json::Value = client_guard
        .get(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to get node status: {}", e))?;

    response
        .get("data")
        .cloned()
        .ok_or_else(|| "Invalid response format: missing data field".to_string())
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let limit_val = limit.unwrap_or(500);
    let path = format!("nodes/{}/syslog?limit={}", node_id, limit_val);
    let response: serde_json::Value = client_guard
        .get(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to get syslog: {}", e))?;

    response
        .get("data")
        .and_then(|d| d.as_array())
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let path = format!("nodes/{}/network", node_id);
    let response: serde_json::Value = client_guard
        .get(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list network interfaces: {}", e))?;

    response
        .get("data")
        .and_then(|d| d.as_array())
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
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
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let path = "nodes/localhost/subscription";
    let response: serde_json::Value = client_guard
        .get(path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to get subscription status: {}", e))?;

    response
        .get("data")
        .cloned()
        .ok_or_else(|| "Invalid response format: missing data field".to_string())
}

// ─── Phase 15 - Cluster Task Log ─────────────────────────────────────────────

/// List cluster-level tasks
#[tauri::command]
pub async fn list_cluster_tasks(
    cluster_id: String,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let limit_val = limit.unwrap_or(50);
    let path = format!("cluster/tasks?limit={}", limit_val);
    let response: serde_json::Value = client_guard
        .get(&path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list cluster tasks: {}", e))?;

    response
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| arr.to_vec())
        .ok_or_else(|| "Invalid response format".to_string())
}

/// List Proxmox LXC containers
#[tauri::command]
pub async fn list_proxmox_containers(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let clusters = state.proxmox_clusters.lock().await;
    let client = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
    let client_guard = client.lock().await;

    let path = "cluster/resources?type=lxc";
    let response: serde_json::Value = client_guard
        .get(path, Some(client_guard.ticket.as_deref().unwrap_or("")))
        .await
        .map_err(|e| format!("Failed to list containers: {}", e))?;

    response
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| arr.to_vec())
        .ok_or_else(|| "Invalid response format".to_string())
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
    fn test_list_proxmox_containers_error_message() {
        let err = format!("Cluster {} not found", "missing-id");
        assert_eq!(err, "Cluster missing-id not found");
    }

    #[test]
    fn test_list_proxmox_containers_invalid_response() {
        let response = serde_json::json!({"other": "field"});
        let result: Result<Vec<serde_json::Value>, String> = response
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| arr.to_vec())
            .ok_or_else(|| "Invalid response format".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid response format");
    }

    #[test]
    fn test_list_proxmox_containers_valid_response() {
        let response = serde_json::json!({
            "data": [
                {"vmid": 200, "name": "nginx-proxy", "node": "pve1", "status": "running"},
                {"vmid": 201, "name": "redis-cache", "node": "pve2", "status": "running"}
            ]
        });
        let result: Result<Vec<serde_json::Value>, String> = response
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| arr.to_vec())
            .ok_or_else(|| "Invalid response format".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}
