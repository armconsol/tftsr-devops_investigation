// Ceph Cluster Management module
// Provides operations for managing Ceph clusters

use serde::{Deserialize, Serialize};

/// Ceph cluster information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CephCluster {
    pub cluster_id: String,
    pub name: String,
    pub status: String,
    pub health: String,
    pub monitors: Vec<String>,
    pub managers: Vec<String>,
    pub masters: Vec<String>,
    pub osd_count: u32,
    pub osd_up: u32,
    pub osd_in: u32,
    pub pg_total: u32,
    pub pg_active: u32,
    pub pg_clean: u32,
    pub bytes_total: u64,
    pub bytes_used: u64,
    pub bytes_avail: u64,
}

/// Ceph cluster status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CephClusterStatus {
    pub cluster_id: String,
    pub health: String,
    pub last_updated: String,
    pub osd_map: serde_json::Value,
    pub pg_map: serde_json::Value,
}

/// List Ceph clusters
pub async fn list_ceph_clusters(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<CephCluster>, String> {
    let path = "ceph/clusters";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list Ceph clusters: {}", e))?;

    if let Some(clusters) = response.as_array() {
        let cluster_list: Vec<CephCluster> = clusters
            .iter()
            .filter_map(|cluster| {
                let id = cluster.get("cluster_id")?.as_str()?.to_string();
                let name = cluster
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let status = cluster
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let health = cluster
                    .get("health")
                    .and_then(|h| h.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let monitors: Vec<String> = cluster
                    .get("monitors")
                    .and_then(|m| m.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|m| m.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                let managers: Vec<String> = cluster
                    .get("managers")
                    .and_then(|m| m.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|m| m.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                let masters: Vec<String> = cluster
                    .get("masters")
                    .and_then(|m| m.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|m| m.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                let osd_count = cluster
                    .get("osd_count")
                    .and_then(|o| o.as_u64())
                    .unwrap_or(0) as u32;
                let osd_up = cluster.get("osd_up").and_then(|o| o.as_u64()).unwrap_or(0) as u32;
                let osd_in = cluster.get("osd_in").and_then(|o| o.as_u64()).unwrap_or(0) as u32;
                let pg_total = cluster
                    .get("pg_total")
                    .and_then(|p| p.as_u64())
                    .unwrap_or(0) as u32;
                let pg_active = cluster
                    .get("pg_active")
                    .and_then(|p| p.as_u64())
                    .unwrap_or(0) as u32;
                let pg_clean = cluster
                    .get("pg_clean")
                    .and_then(|p| p.as_u64())
                    .unwrap_or(0) as u32;
                let bytes_total = cluster
                    .get("bytes_total")
                    .and_then(|b| b.as_u64())
                    .unwrap_or(0);
                let bytes_used = cluster
                    .get("bytes_used")
                    .and_then(|b| b.as_u64())
                    .unwrap_or(0);
                let bytes_avail = cluster
                    .get("bytes_avail")
                    .and_then(|b| b.as_u64())
                    .unwrap_or(0);

                Some(CephCluster {
                    cluster_id: id,
                    name,
                    status,
                    health,
                    monitors,
                    managers,
                    masters,
                    osd_count,
                    osd_up,
                    osd_in,
                    pg_total,
                    pg_active,
                    pg_clean,
                    bytes_total,
                    bytes_used,
                    bytes_avail,
                })
            })
            .collect();

        Ok(cluster_list)
    } else {
        Err("Invalid response format".to_string())
    }
}

/// Get Ceph cluster status
pub async fn get_ceph_cluster_status(
    client: &crate::proxmox::client::ProxmoxClient,
    cluster_id: &str,
    ticket: &str,
) -> Result<CephClusterStatus, String> {
    let path = format!("ceph/clusters/{}/status", cluster_id);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get Ceph cluster {} status: {}", cluster_id, e))?;

    {
        let data = &response;
        let id = data
            .get("cluster_id")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let health = data
            .get("health")
            .and_then(|h| h.as_str())
            .unwrap_or("unknown")
            .to_string();
        let last_updated = data
            .get("last_updated")
            .and_then(|l| l.as_str())
            .unwrap_or("")
            .to_string();
        let osd_map = data
            .get("osd_map")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        let pg_map = data.get("pg_map").cloned().unwrap_or(serde_json::json!({}));

        Ok(CephClusterStatus {
            cluster_id: id,
            health,
            last_updated,
            osd_map,
            pg_map,
        })
    }
}

/// Add Ceph cluster
pub async fn add_ceph_cluster(
    client: &crate::proxmox::client::ProxmoxClient,
    cluster_id: &str,
    name: &str,
    mon_host: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = "ceph/clusters";
    let config = serde_json::json!({
        "cluster_id": cluster_id,
        "name": name,
        "mon_host": mon_host
    });

    let _response: serde_json::Value = client
        .post(path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to add Ceph cluster {}: {}", cluster_id, e))?;
    Ok(())
}

/// Remove Ceph cluster
pub async fn remove_ceph_cluster(
    client: &crate::proxmox::client::ProxmoxClient,
    cluster_id: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("ceph/clusters/{}", cluster_id);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to remove Ceph cluster {}: {}", cluster_id, e))?;
    Ok(())
}

/// Get Ceph cluster configuration
pub async fn get_ceph_cluster_config(
    client: &crate::proxmox::client::ProxmoxClient,
    cluster_id: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    let path = format!("ceph/clusters/{}/config", cluster_id);
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get Ceph cluster {} config: {}", cluster_id, e))
}

/// Sync Ceph cluster
pub async fn sync_ceph_cluster(
    client: &crate::proxmox::client::ProxmoxClient,
    cluster_id: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("ceph/clusters/{}/sync", cluster_id);
    let _response: serde_json::Value = client
        .post(&path, &serde_json::json!({}), Some(ticket))
        .await
        .map_err(|e| format!("Failed to sync Ceph cluster {}: {}", cluster_id, e))?;
    Ok(())
}
