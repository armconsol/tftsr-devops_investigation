// Ceph management module
// Provides operations for managing Ceph clusters

use serde::{Deserialize, Serialize};

/// Ceph pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CephPool {
    pub pool: String,
    pub pool_id: u64,
    pub size: u32,
    pub min_size: u32,
    pub pg_num: u32,
    pub used: u64,
    pub avail: u64,
    pub status: String,
}

/// Ceph OSD information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CephOsd {
    pub osd: u32,
    pub up: bool,
    pub in_: bool,
    pub weight: f64,
    pub pg_num: u32,
    pub usage: f64,
}

/// Ceph monitor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CephMonitor {
    pub name: String,
    pub quorum: bool,
    pub address: String,
    pub version: String,
}

/// Ceph health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CephHealth {
    pub status: String,
    pub summary: String,
    pub details: Vec<String>,
}

/// List Ceph pools
pub async fn list_pools(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<CephPool>, String> {
    let path = format!("nodes/{}/ceph/pool", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list Ceph pools: {}", e))?;

    if let Some(pools) = response.as_array() {
        let pool_list: Vec<CephPool> = pools
            .iter()
            .filter_map(|pool| {
                let pool_name = pool.get("pool")?.as_str()?.to_string();
                let pool_id = pool.get("poolid")?.as_u64()?;
                let size = pool.get("size")?.as_u64()? as u32;
                let min_size = pool.get("min_size")?.as_u64()? as u32;
                let pg_num = pool.get("pg_num")?.as_u64()? as u32;
                let used = pool.get("used")?.as_u64()?;
                let avail = pool.get("avail")?.as_u64()?;
                let status = pool
                    .get("status")?
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();

                Some(CephPool {
                    pool: pool_name,
                    pool_id,
                    size,
                    min_size,
                    pg_num,
                    used,
                    avail,
                    status,
                })
            })
            .collect();

        Ok(pool_list)
    } else {
        Err("Invalid response format".to_string())
    }
}

/// Create Ceph pool
pub async fn create_pool(
    client: &crate::proxmox::client::ProxmoxClient,
    pool: &str,
    pg_num: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = "cluster/ceph/pool";
    let config = serde_json::json!({
        "pool": pool,
        "pg_num": pg_num
    });

    let _response: serde_json::Value = client
        .post(path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create Ceph pool {}: {}", pool, e))?;
    Ok(())
}

/// Delete Ceph pool
pub async fn delete_pool(
    client: &crate::proxmox::client::ProxmoxClient,
    pool: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ceph/pool/{}", pool);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete Ceph pool {}: {}", pool, e))?;
    Ok(())
}

/// Set Ceph pool quota
pub async fn set_pool_quota(
    client: &crate::proxmox::client::ProxmoxClient,
    pool: &str,
    max_bytes: u64,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ceph/pool/{}", pool);
    let config = serde_json::json!({
        "max_bytes": max_bytes
    });

    let _response: serde_json::Value = client
        .put(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to set quota for Ceph pool {}: {}", pool, e))?;
    Ok(())
}

/// List Ceph OSDs
pub async fn list_osds(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<CephOsd>, String> {
    let path = format!("nodes/{}/ceph/osd", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list Ceph OSDs: {}", e))?;

    if let Some(osds) = response.as_array() {
        let osd_list: Vec<CephOsd> = osds
            .iter()
            .filter_map(|osd| {
                let osd_id = osd.get("osd")?.as_u64()? as u32;
                let up = osd.get("up")?.as_bool()?;
                let in_ = osd.get("in")?.as_bool()?;
                let weight = osd.get("weight")?.as_f64()?;
                let pg_num = osd.get("pg_num")?.as_u64()? as u32;
                let usage = osd.get("kb_used")?.as_f64().unwrap_or(0.0);

                Some(CephOsd {
                    osd: osd_id,
                    up,
                    in_,
                    weight,
                    pg_num,
                    usage,
                })
            })
            .collect();

        Ok(osd_list)
    } else {
        Err("Invalid response format".to_string())
    }
}

/// Set OSD weight
pub async fn set_osd_weight(
    client: &crate::proxmox::client::ProxmoxClient,
    osd_id: u32,
    weight: f64,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ceph/osd/{}", osd_id);
    let config = serde_json::json!({
        "weight": weight
    });

    let _response: serde_json::Value = client
        .put(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to set weight for OSD {}: {}", osd_id, e))?;
    Ok(())
}

/// Mark OSD out
pub async fn osd_out(
    client: &crate::proxmox::client::ProxmoxClient,
    osd_id: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ceph/osd/{}/out", osd_id);
    let _response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to mark OSD {} out: {}", osd_id, e))?;
    Ok(())
}

/// Mark OSD in
pub async fn osd_in(
    client: &crate::proxmox::client::ProxmoxClient,
    osd_id: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ceph/osd/{}/in", osd_id);
    let _response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to mark OSD {} in: {}", osd_id, e))?;
    Ok(())
}

/// List Ceph MDS
pub async fn list_mds(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let path = "cluster/ceph/mds";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list Ceph MDS: {}", e))?;

    if let Some(mds) = response.as_array() {
        Ok(mds.to_vec())
    } else {
        Err("Invalid response format".to_string())
    }
}

/// Get MDS status
pub async fn get_mds_status(
    client: &crate::proxmox::client::ProxmoxClient,
    mds: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    let path = format!("cluster/ceph/mds/{}", mds);
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get MDS {}: {}", mds, e))
}

/// Trigger MDS failover
pub async fn mds_failover(
    client: &crate::proxmox::client::ProxmoxClient,
    mds: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ceph/mds/{}/failover", mds);
    let _response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to trigger MDS failover {}: {}", mds, e))?;
    Ok(())
}

/// List RBD images
pub async fn list_rbd(
    client: &crate::proxmox::client::ProxmoxClient,
    pool: &str,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let path = format!("cluster/ceph/pool/{}/rbd", pool);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list RBD images in pool {}: {}", pool, e))?;

    if let Some(images) = response.as_array() {
        Ok(images.to_vec())
    } else {
        Err("Invalid response format".to_string())
    }
}

/// Create RBD image
pub async fn create_rbd(
    client: &crate::proxmox::client::ProxmoxClient,
    pool: &str,
    image: &str,
    size: u64,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ceph/pool/{}/rbd", pool);
    let config = serde_json::json!({
        "image": image,
        "size": size
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create RBD image {}: {}", image, e))?;
    Ok(())
}

/// Delete RBD image
pub async fn delete_rbd(
    client: &crate::proxmox::client::ProxmoxClient,
    pool: &str,
    image: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ceph/pool/{}/rbd/{}", pool, image);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete RBD image {}: {}", image, e))?;
    Ok(())
}

/// Clone RBD image
pub async fn clone_rbd(
    client: &crate::proxmox::client::ProxmoxClient,
    source_pool: &str,
    source_image: &str,
    dest_pool: &str,
    dest_image: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ceph/pool/{}/clone", source_pool);
    let config = serde_json::json!({
        "source": source_image,
        "dest": format!("{}/{}", dest_pool, dest_image)
    });

    let _response: serde_json::Value =
        client
            .post(&path, &config, Some(ticket))
            .await
            .map_err(|e| {
                format!(
                    "Failed to clone RBD image {} to {}/{}: {}",
                    source_image, dest_pool, dest_image, e
                )
            })?;
    Ok(())
}

/// Resize RBD image
pub async fn resize_rbd(
    client: &crate::proxmox::client::ProxmoxClient,
    pool: &str,
    image: &str,
    size: u64,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ceph/pool/{}/rbd/{}/resize", pool, image);
    let config = serde_json::json!({
        "size": size
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to resize RBD image {}: {}", image, e))?;
    Ok(())
}

/// Create RBD snapshot
pub async fn create_snapshot(
    client: &crate::proxmox::client::ProxmoxClient,
    pool: &str,
    image: &str,
    snapshot: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ceph/pool/{}/rbd/{}/snapshot", pool, image);
    let config = serde_json::json!({
        "snapshot": snapshot
    });

    let _response: serde_json::Value =
        client
            .post(&path, &config, Some(ticket))
            .await
            .map_err(|e| {
                format!(
                    "Failed to create snapshot {} for RBD image {}: {}",
                    snapshot, image, e
                )
            })?;
    Ok(())
}

fn validate_node(node: &str) -> Result<(), String> {
    if node.is_empty() || node.len() > 64 {
        return Err("Node name must be between 1 and 64 characters".to_string());
    }
    if !node.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(format!(
            "Invalid node name '{}': only alphanumeric characters and hyphens are allowed",
            node
        ));
    }
    Ok(())
}

/// Ceph manager information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CephMgr {
    pub name: String,
    pub addr: Option<String>,
    pub state: Option<String>,
}

/// List Ceph managers on a specific node
pub async fn list_managers(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<CephMgr>, String> {
    validate_node(node)?;
    let path = format!("nodes/{}/ceph/mgr", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list Ceph managers on node {}: {}", node, e))?;

    let managers = response
        .as_array()
        .ok_or_else(|| "Invalid response format".to_string())?;

    let mgr_list = managers
        .iter()
        .filter_map(|mgr| {
            let name = mgr.get("name")?.as_str()?.to_string();
            let addr = mgr
                .get("addr")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let state = mgr
                .get("state")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Some(CephMgr { name, addr, state })
        })
        .collect();

    Ok(mgr_list)
}

/// CephFS filesystem information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CephFs {
    pub name: String,
    pub metadata_pool: Option<String>,
    pub data_pool_ids: Option<Vec<i64>>,
}

/// List CephFS filesystems on a specific node
pub async fn list_cephfs(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<CephFs>, String> {
    validate_node(node)?;
    let path = format!("nodes/{}/ceph/fs", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list CephFS on node {}: {}", node, e))?;

    let filesystems = response
        .as_array()
        .ok_or_else(|| "Invalid response format".to_string())?;

    let fs_list = filesystems
        .iter()
        .filter_map(|fs| {
            let name = fs.get("name")?.as_str()?.to_string();
            let metadata_pool = fs
                .get("metadata_pool")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let data_pool_ids = fs
                .get("data_pool_ids")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|id| id.as_i64())
                        .collect::<Vec<i64>>()
                });
            Some(CephFs {
                name,
                metadata_pool,
                data_pool_ids,
            })
        })
        .collect();

    Ok(fs_list)
}

/// Get Ceph runtime flags on a specific node
///
/// Returns a polymorphic object of flag states — kept as `Value` because the
/// set of flags varies with the Ceph release and cluster configuration.
pub async fn get_ceph_flags(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    validate_node(node)?;
    let path = format!("nodes/{}/ceph/flags", node);
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get Ceph flags on node {}: {}", node, e))
}

/// List Ceph monitors
pub async fn list_monitors(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<CephMonitor>, String> {
    let path = format!("nodes/{}/ceph/mon", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list Ceph monitors: {}", e))?;

    if let Some(mons) = response.as_array() {
        let mon_list: Vec<CephMonitor> = mons
            .iter()
            .filter_map(|mon| {
                let name = mon.get("name")?.as_str()?.to_string();
                let quorum = mon.get("quorum")?.as_bool()?;
                let address = mon.get("addr")?.as_str()?.to_string();
                let version = mon
                    .get("version")?
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();

                Some(CephMonitor {
                    name,
                    quorum,
                    address,
                    version,
                })
            })
            .collect();

        Ok(mon_list)
    } else {
        Err("Invalid response format".to_string())
    }
}

/// Get monitor status
pub async fn get_monitor_status(
    client: &crate::proxmox::client::ProxmoxClient,
    monitor: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    let path = format!("cluster/ceph/mon/{}", monitor);
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get monitor {}: {}", monitor, e))
}

/// Get Ceph quorum health
pub async fn quorum_health(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    let path = "cluster/ceph/health";
    client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get Ceph health: {}", e))
}

/// Get Ceph health
pub async fn get_ceph_health(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<CephHealth, String> {
    let path = format!("nodes/{}/ceph/status", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get Ceph health: {}", e))?;

    let data = response.get("data").ok_or("Invalid response format")?;
    let health = data.get("health").unwrap_or(data);

    let details: Vec<String> = health
        .get("checks")
        .and_then(|c| c.as_object())
        .map(|checks| {
            checks
                .values()
                .filter_map(|v| {
                    v.get("summary")
                        .and_then(|s| s.get("message"))
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(CephHealth {
        status: health
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string(),
        summary: health
            .get("summary")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string(),
        details,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ceph_pool_serialization() {
        let pool = CephPool {
            pool: "rbd".to_string(),
            pool_id: 1,
            size: 3,
            min_size: 2,
            pg_num: 128,
            used: 1000000000000,
            avail: 2000000000000,
            status: "healthy".to_string(),
        };

        let json = serde_json::to_string(&pool).unwrap();
        let deserialized: CephPool = serde_json::from_str(&json).unwrap();

        assert_eq!(pool.pool, deserialized.pool);
        assert_eq!(pool.status, "healthy");
    }

    #[test]
    fn test_ceph_osd_serialization() {
        let osd = CephOsd {
            osd: 0,
            up: true,
            in_: true,
            weight: 1.0,
            pg_num: 128,
            usage: 0.5,
        };

        let json = serde_json::to_string(&osd).unwrap();
        let deserialized: CephOsd = serde_json::from_str(&json).unwrap();

        assert_eq!(osd.osd, deserialized.osd);
        assert_eq!(osd.up, deserialized.up);
    }

    #[test]
    fn test_ceph_monitor_serialization() {
        let mon = CephMonitor {
            name: "pve-mon-1".to_string(),
            quorum: true,
            address: "10.0.0.1:6789".to_string(),
            version: "18.2.0".to_string(),
        };

        let json = serde_json::to_string(&mon).unwrap();
        let deserialized: CephMonitor = serde_json::from_str(&json).unwrap();

        assert_eq!(mon.name, deserialized.name);
        assert_eq!(mon.quorum, deserialized.quorum);
    }

    #[test]
    fn test_ceph_health_serialization() {
        let health = CephHealth {
            status: "HEALTH_OK".to_string(),
            summary: "Cluster is healthy".to_string(),
            details: vec!["All OSDs are up".to_string()],
        };

        let json = serde_json::to_string(&health).unwrap();
        let deserialized: CephHealth = serde_json::from_str(&json).unwrap();

        assert_eq!(health.status, deserialized.status);
        assert_eq!(health.summary, deserialized.summary);
    }

    #[test]
    fn test_ceph_mgr_deserialization_from_fixture() {
        let fixture = serde_json::json!([
            {"name": "pve-mgr-0", "addr": "10.0.0.1:6800", "state": "active"},
            {"name": "pve-mgr-1", "addr": "10.0.0.2:6800", "state": "standby"},
            {"name": "pve-mgr-orphan"}
        ]);

        let managers: Vec<CephMgr> = fixture
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|mgr| {
                let name = mgr.get("name")?.as_str()?.to_string();
                let addr = mgr
                    .get("addr")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let state = mgr
                    .get("state")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                Some(CephMgr { name, addr, state })
            })
            .collect();

        assert_eq!(managers.len(), 3);
        assert_eq!(managers[0].name, "pve-mgr-0");
        assert_eq!(managers[0].addr.as_deref(), Some("10.0.0.1:6800"));
        assert_eq!(managers[0].state.as_deref(), Some("active"));
        assert_eq!(managers[1].state.as_deref(), Some("standby"));
        assert!(
            managers[2].addr.is_none(),
            "orphan manager should have no addr"
        );
        assert!(
            managers[2].state.is_none(),
            "orphan manager should have no state"
        );
    }

    #[test]
    fn test_ceph_mgr_roundtrip_serialization() {
        let mgr = CephMgr {
            name: "pve-mgr-0".to_string(),
            addr: Some("10.0.0.1:6800".to_string()),
            state: Some("active".to_string()),
        };
        let json = serde_json::to_string(&mgr).unwrap();
        let deserialized: CephMgr = serde_json::from_str(&json).unwrap();
        assert_eq!(mgr.name, deserialized.name);
        assert_eq!(mgr.addr, deserialized.addr);
        assert_eq!(mgr.state, deserialized.state);
    }

    #[test]
    fn test_ceph_fs_deserialization_from_fixture() {
        let fixture = serde_json::json!([
            {"name": "cephfs", "metadata_pool": "cephfs_metadata", "data_pool_ids": [2, 3]},
            {"name": "bare-fs"}
        ]);

        let filesystems: Vec<CephFs> = fixture
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|fs| {
                let name = fs.get("name")?.as_str()?.to_string();
                let metadata_pool = fs
                    .get("metadata_pool")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let data_pool_ids = fs
                    .get("data_pool_ids")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|id| id.as_i64())
                            .collect::<Vec<i64>>()
                    });
                Some(CephFs {
                    name,
                    metadata_pool,
                    data_pool_ids,
                })
            })
            .collect();

        assert_eq!(filesystems.len(), 2);
        assert_eq!(filesystems[0].name, "cephfs");
        assert_eq!(
            filesystems[0].metadata_pool.as_deref(),
            Some("cephfs_metadata")
        );
        assert_eq!(
            filesystems[0].data_pool_ids.as_deref(),
            Some(vec![2i64, 3i64].as_slice())
        );
        assert!(filesystems[1].metadata_pool.is_none());
        assert!(filesystems[1].data_pool_ids.is_none());
    }

    #[test]
    fn test_validate_node_rejects_path_traversal() {
        assert!(validate_node("../etc").is_err());
        assert!(validate_node("node/bad").is_err());
        assert!(validate_node("node\\bad").is_err());
        assert!(validate_node("node with space").is_err());
    }

    #[test]
    fn test_validate_node_accepts_valid_names() {
        assert!(validate_node("pve-node-01").is_ok());
        assert!(validate_node("pve1").is_ok());
        assert!(validate_node("NODE").is_ok());
    }

    #[test]
    fn test_validate_node_rejects_empty_and_too_long() {
        assert!(validate_node("").is_err());
        assert!(validate_node(&"a".repeat(65)).is_err());
        assert!(validate_node(&"a".repeat(64)).is_ok());
    }

    #[test]
    fn test_ceph_health_uses_node_path() {
        let node = "pve1";
        let path = format!("nodes/{}/ceph/status", node);
        assert!(path.contains("nodes/"));
        assert!(path.contains("/ceph/status"));
        assert!(!path.contains("cluster/"));
    }

    #[test]
    fn test_list_pools_uses_node_path() {
        let node = "pve1";
        let path = format!("nodes/{}/ceph/pool", node);
        assert!(path.starts_with("nodes/"));
        assert!(!path.starts_with("cluster/"));
    }

    #[test]
    fn test_list_osds_uses_node_path() {
        let node = "pve1";
        let path = format!("nodes/{}/ceph/osd", node);
        assert!(path.starts_with("nodes/"));
        assert!(!path.starts_with("cluster/"));
    }

    #[test]
    fn test_list_monitors_uses_node_path() {
        let node = "pve1";
        let path = format!("nodes/{}/ceph/mon", node);
        assert!(path.starts_with("nodes/"));
        assert!(!path.starts_with("cluster/"));
    }
}
