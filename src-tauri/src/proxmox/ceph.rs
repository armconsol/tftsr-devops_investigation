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
    ticket: &str,
) -> Result<Vec<CephPool>, String> {
    let path = "cluster/ceph/pool";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
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
    ticket: &str,
) -> Result<Vec<CephOsd>, String> {
    let path = "cluster/ceph/osd";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
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

/// List Ceph monitors
pub async fn list_monitors(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<CephMonitor>, String> {
    let path = "cluster/ceph/mon";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
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
    ticket: &str,
) -> Result<CephHealth, String> {
    let path = "cluster/ceph/health";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get Ceph health: {}", e))?;

    // response IS the health data (handle_response already unwrapped the envelope)
    let health = &response;

    let details: Vec<String> = health
        .get("details")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|d| {
                    d.get("message")
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
}
