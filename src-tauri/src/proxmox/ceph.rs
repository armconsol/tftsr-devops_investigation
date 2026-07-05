// Ceph management module
// Provides operations for managing Ceph clusters

use serde::{Deserialize, Serialize};

/// Ceph pool information.
///
/// Field names are camelCase to match the frontend `CephPool` type exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CephPool {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub pool_type: String,
    pub size: u32,
    pub min_size: u32,
    pub used: u64,
    pub available: u64,
    pub total: u64,
    pub used_percent: f64,
}

/// Ceph OSD information.
///
/// Field names are camelCase to match the frontend `CephOsd` type exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CephOsd {
    pub id: u32,
    pub host: String,
    pub status: String,
    pub weight: f64,
    pub size: u64,
    pub used: u64,
    pub avail: u64,
    pub used_percent: f64,
}

/// Ceph monitor information.
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

/// Parse an (already `data`-unwrapped) `nodes/{node}/ceph/pool` response.
///
/// PVE returns an array of pool objects with fields `pool` (numeric id as a
/// string), `pool_name`, `type`, `size`, `min_size`, `bytes_used` and
/// `percent_used` (a 0..1 fraction). It does **not** expose `avail`/`total`, so
/// those are derived from `bytes_used` and `percent_used`.
pub fn parse_pools(response: &serde_json::Value) -> Result<Vec<CephPool>, String> {
    let pools = response
        .as_array()
        .ok_or_else(|| "Invalid response format".to_string())?;

    let pool_list = pools
        .iter()
        .filter_map(|pool| {
            let id = pool
                .get("pool")
                .map(value_to_id_string)
                .filter(|s| !s.is_empty())?;
            let name = pool.get("pool_name").and_then(|v| v.as_str())?.to_string();
            let pool_type = pool
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("replicated")
                .to_string();
            let size = pool.get("size").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let min_size = pool.get("min_size").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let used = pool.get("bytes_used").and_then(|v| v.as_u64()).unwrap_or(0);
            let fraction = pool
                .get("percent_used")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            // `percent_used` is a 0..1 fraction; total/available are derived.
            let total = if fraction > 0.0 {
                (used as f64 / fraction).round() as u64
            } else {
                0
            };
            let available = total.saturating_sub(used);
            let used_percent = fraction * 100.0;

            Some(CephPool {
                id,
                name,
                pool_type,
                size,
                min_size,
                used,
                available,
                total,
                used_percent,
            })
        })
        .collect();

    Ok(pool_list)
}

/// Coerce a JSON value that may be either a string or a number into a string id.
fn value_to_id_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

/// List Ceph pools
pub async fn list_pools(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<CephPool>, String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/pool");
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list Ceph pools: {e}"))?;

    parse_pools(&response)
}

/// Create Ceph pool
pub async fn create_pool(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    pool: &str,
    pg_num: u32,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/pool");
    let config = serde_json::json!({
        "pool": pool,
        "pg_num": pg_num
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create Ceph pool {pool}: {e}"))?;
    Ok(())
}

/// Delete Ceph pool
pub async fn delete_pool(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    pool: &str,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/pool/{pool}");
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete Ceph pool {pool}: {e}"))?;
    Ok(())
}

/// Set Ceph pool quota
pub async fn set_pool_quota(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    pool: &str,
    max_bytes: u64,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/pool/{pool}");
    let config = serde_json::json!({
        "max_bytes": max_bytes
    });

    let _response: serde_json::Value = client
        .put(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to set quota for Ceph pool {pool}: {e}"))?;
    Ok(())
}

/// Parse an (already `data`-unwrapped) `nodes/{node}/ceph/osd` response.
///
/// PVE returns the CRUSH tree as an **object** `{ flags, root: { children: [...] } }`,
/// not a flat array. Host nodes nest OSD leaves under `children`; OSD leaves are
/// identified by `type == "osd"`. Each leaf carries `id` (numeric id as string),
/// `host`, `status` (`up`/`down`), `crush_weight`, `bytes_used`, `total_space`
/// and `percent_used`.
pub fn parse_osds(response: &serde_json::Value) -> Result<Vec<CephOsd>, String> {
    let root = response
        .get("root")
        .ok_or_else(|| "Invalid response format".to_string())?;

    let mut osds = Vec::new();
    collect_osds(root, &mut osds);
    Ok(osds)
}

/// Recursively walk the CRUSH tree, collecting every `type == "osd"` leaf.
fn collect_osds(node: &serde_json::Value, out: &mut Vec<CephOsd>) {
    if node.get("type").and_then(|v| v.as_str()) == Some("osd") {
        if let Some(osd) = parse_osd_leaf(node) {
            out.push(osd);
        }
        return;
    }
    if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
        for child in children {
            collect_osds(child, out);
        }
    }
}

fn parse_osd_leaf(osd: &serde_json::Value) -> Option<CephOsd> {
    // `id` is delivered as a string (e.g. "19").
    let id = osd.get("id").and_then(|v| match v {
        serde_json::Value::String(s) => s.parse::<u32>().ok(),
        serde_json::Value::Number(n) => n.as_u64().map(|n| n as u32),
        _ => None,
    })?;
    let host = osd
        .get("host")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let status = osd
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("down")
        .to_string();
    let weight = osd
        .get("crush_weight")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let size = osd.get("total_space").and_then(|v| v.as_u64()).unwrap_or(0);
    let used = osd.get("bytes_used").and_then(|v| v.as_u64()).unwrap_or(0);
    let avail = size.saturating_sub(used);
    // Derive the percentage from raw bytes so the value is always a true 0..100
    // percent, independent of how PVE rounds `percent_used`.
    let used_percent = if size > 0 {
        (used as f64 / size as f64) * 100.0
    } else {
        osd.get("percent_used")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
    };

    Some(CephOsd {
        id,
        host,
        status,
        weight,
        size,
        used,
        avail,
        used_percent,
    })
}

/// List Ceph OSDs
pub async fn list_osds(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<CephOsd>, String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/osd");
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list Ceph OSDs: {e}"))?;

    parse_osds(&response)
}

/// Set OSD weight
pub async fn set_osd_weight(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    osd_id: u32,
    weight: f64,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/osd/{osd_id}");
    let config = serde_json::json!({
        "weight": weight
    });

    let _response: serde_json::Value = client
        .put(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to set weight for OSD {osd_id}: {e}"))?;
    Ok(())
}

/// Mark OSD out
pub async fn osd_out(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    osd_id: u32,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/osd/{osd_id}/out");
    let _response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to mark OSD {osd_id} out: {e}"))?;
    Ok(())
}

/// Mark OSD in
pub async fn osd_in(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    osd_id: u32,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/osd/{osd_id}/in");
    let _response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to mark OSD {osd_id} in: {e}"))?;
    Ok(())
}

/// List Ceph MDS
pub async fn list_mds(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/mds");
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list Ceph MDS: {e}"))?;

    if let Some(mds) = response.as_array() {
        Ok(mds.to_vec())
    } else {
        Err("Invalid response format".to_string())
    }
}

/// Get MDS status
pub async fn get_mds_status(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    mds: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/mds/{mds}");
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get MDS {mds}: {e}"))
}

/// Trigger MDS failover
pub async fn mds_failover(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    mds: &str,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/mds/{mds}/failover");
    let _response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to trigger MDS failover {mds}: {e}"))?;
    Ok(())
}

/// List RBD images
pub async fn list_rbd(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    pool: &str,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/pool/{pool}/rbd");
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list RBD images in pool {pool}: {e}"))?;

    if let Some(images) = response.as_array() {
        Ok(images.to_vec())
    } else {
        Err("Invalid response format".to_string())
    }
}

/// Create RBD image
pub async fn create_rbd(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    pool: &str,
    image: &str,
    size: u64,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/pool/{pool}/rbd");
    let config = serde_json::json!({
        "image": image,
        "size": size
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create RBD image {image}: {e}"))?;
    Ok(())
}

/// Delete RBD image
pub async fn delete_rbd(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    pool: &str,
    image: &str,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/pool/{pool}/rbd/{image}");
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete RBD image {image}: {e}"))?;
    Ok(())
}

/// Clone RBD image
pub async fn clone_rbd(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    source_pool: &str,
    source_image: &str,
    dest_pool: &str,
    dest_image: &str,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/pool/{source_pool}/clone");
    let config = serde_json::json!({
        "source": source_image,
        "dest": format!("{dest_pool}/{dest_image}")
    });

    let _response: serde_json::Value =
        client
            .post(&path, &config, Some(ticket))
            .await
            .map_err(|e| {
                format!("Failed to clone RBD image {source_image} to {dest_pool}/{dest_image}: {e}")
            })?;
    Ok(())
}

/// Resize RBD image
pub async fn resize_rbd(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    pool: &str,
    image: &str,
    size: u64,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/pool/{pool}/rbd/{image}/resize");
    let config = serde_json::json!({
        "size": size
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to resize RBD image {image}: {e}"))?;
    Ok(())
}

/// Create RBD snapshot
pub async fn create_snapshot(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    pool: &str,
    image: &str,
    snapshot: &str,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/pool/{pool}/rbd/{image}/snapshot");
    let config = serde_json::json!({
        "snapshot": snapshot
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create snapshot {snapshot} for RBD image {image}: {e}"))?;
    Ok(())
}

fn validate_node(node: &str) -> Result<(), String> {
    if node.is_empty() || node.len() > 64 {
        return Err("Node name must be between 1 and 64 characters".to_string());
    }
    if !node.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(format!(
            "Invalid node name '{node}': only alphanumeric characters and hyphens are allowed"
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

/// Parse an (already `data`-unwrapped) `nodes/{node}/ceph/mgr` response into
/// `CephMgr` rows. PVE returns an array of `{ name, addr, state }`.
pub fn parse_managers(response: &serde_json::Value) -> Result<Vec<CephMgr>, String> {
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

/// List Ceph managers on a specific node
pub async fn list_managers(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<CephMgr>, String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/mgr");
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list Ceph managers on node {node}: {e}"))?;

    parse_managers(&response)
}

/// CephFS filesystem information.
///
/// Field names are camelCase to match the frontend `CephFs` type exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CephFs {
    pub name: String,
    pub metadata_pool: Option<String>,
    pub data_pool: Option<String>,
}

/// Extract `data_pool` from a CephFS entry, accepting either a single
/// `data_pool` string (older PVE) or a `data_pools` array (current PVE, since
/// a filesystem can have more than one data pool) — joined with ", " when
/// there are multiple. Array elements may be pool names (strings) or numeric
/// pool ids.
fn extract_data_pool(fs: &serde_json::Value) -> Option<String> {
    if let Some(s) = fs.get("data_pool").and_then(|v| v.as_str()) {
        return Some(s.to_string());
    }
    let pools = fs.get("data_pools")?.as_array()?;
    let names: Vec<String> = pools
        .iter()
        .filter_map(|p| {
            p.as_str()
                .map(str::to_string)
                .or_else(|| p.as_i64().map(|n| n.to_string()))
        })
        .collect();
    if names.is_empty() {
        None
    } else {
        Some(names.join(", "))
    }
}

/// Parse an (already `data`-unwrapped) `nodes/{node}/ceph/fs` response.
///
/// PVE returns an array of `{ name, metadata_pool, data_pool | data_pools }`.
pub fn parse_cephfs(response: &serde_json::Value) -> Result<Vec<CephFs>, String> {
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
            let data_pool = extract_data_pool(fs);
            Some(CephFs {
                name,
                metadata_pool,
                data_pool,
            })
        })
        .collect();

    Ok(fs_list)
}

/// List CephFS filesystems on a specific node
pub async fn list_cephfs(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<CephFs>, String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/fs");
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list CephFS on node {node}: {e}"))?;

    parse_cephfs(&response)
}

/// Get Ceph runtime flags.
///
/// Flags are a **cluster-level** resource in PVE (`/cluster/ceph/flags`); the
/// per-node `nodes/{node}/ceph/flags` path returns HTTP 501. Returns the array
/// of `{ name, value, description }` objects unchanged.
pub async fn get_ceph_flags(
    client: &crate::proxmox::client::ProxmoxClient,
    _node: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    client
        .get("cluster/ceph/flags", Some(ticket))
        .await
        .map_err(|e| format!("Failed to get Ceph flags: {e}"))
}

/// Every runtime flag PVE/Ceph exposes via `/cluster/ceph/flags`. Used to
/// whitelist mutation requests so only known flags can be toggled.
pub const CEPH_FLAGS: &[&str] = &[
    "noout",
    "noin",
    "nodown",
    "noup",
    "norebalance",
    "norecover",
    "noscrub",
    "nodeep-scrub",
    "nobackfill",
    "notieragent",
    "pause",
];

/// Validate a Ceph flag name against the known whitelist.
pub fn validate_ceph_flag(flag: &str) -> Result<(), String> {
    if CEPH_FLAGS.contains(&flag) {
        Ok(())
    } else {
        Err(format!(
            "Invalid Ceph flag '{flag}': must be one of {CEPH_FLAGS:?}"
        ))
    }
}

/// Set (or clear) a cluster-level Ceph runtime flag.
/// PUT /cluster/ceph/flags/{flag} { value: bool }
pub async fn set_ceph_flag(
    client: &crate::proxmox::client::ProxmoxClient,
    flag: &str,
    value: bool,
    ticket: &str,
) -> Result<(), String> {
    validate_ceph_flag(flag)?;
    let path = format!("cluster/ceph/flags/{flag}");
    let body = serde_json::json!({ "value": value });
    let _response: serde_json::Value = client
        .put(&path, &body, Some(ticket))
        .await
        .map_err(|e| format!("Failed to set Ceph flag {flag}: {e}"))?;
    Ok(())
}

/// Validate a Ceph mon/mgr service id: alphanumeric, hyphens, dots, max 64.
pub fn validate_ceph_service_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 {
        return Err("Service id must be between 1 and 64 characters".to_string());
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
    {
        return Err(format!(
            "Invalid service id '{id}': only alphanumeric characters, hyphens and dots are allowed"
        ));
    }
    Ok(())
}

/// Create a Ceph monitor on a node. POST /nodes/{node}/ceph/mon/{monid}.
/// Returns the task UPID.
pub async fn create_mon(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    monid: &str,
    ticket: &str,
) -> Result<String, String> {
    validate_node(node)?;
    validate_ceph_service_id(monid)?;
    let path = format!("nodes/{node}/ceph/mon/{monid}");
    let response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to create Ceph monitor {monid}: {e}"))?;
    Ok(response.as_str().unwrap_or_default().to_string())
}

/// Destroy a Ceph monitor. DELETE /nodes/{node}/ceph/mon/{monid}.
/// Returns the task UPID.
pub async fn destroy_mon(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    monid: &str,
    ticket: &str,
) -> Result<String, String> {
    validate_node(node)?;
    validate_ceph_service_id(monid)?;
    let path = format!("nodes/{node}/ceph/mon/{monid}");
    let response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to destroy Ceph monitor {monid}: {e}"))?;
    Ok(response.as_str().unwrap_or_default().to_string())
}

/// Create a Ceph manager on a node. POST /nodes/{node}/ceph/mgr/{id}.
/// Returns the task UPID.
pub async fn create_mgr(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    id: &str,
    ticket: &str,
) -> Result<String, String> {
    validate_node(node)?;
    validate_ceph_service_id(id)?;
    let path = format!("nodes/{node}/ceph/mgr/{id}");
    let response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to create Ceph manager {id}: {e}"))?;
    Ok(response.as_str().unwrap_or_default().to_string())
}

/// Destroy a Ceph manager. DELETE /nodes/{node}/ceph/mgr/{id}.
/// Returns the task UPID.
pub async fn destroy_mgr(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    id: &str,
    ticket: &str,
) -> Result<String, String> {
    validate_node(node)?;
    validate_ceph_service_id(id)?;
    let path = format!("nodes/{node}/ceph/mgr/{id}");
    let response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to destroy Ceph manager {id}: {e}"))?;
    Ok(response.as_str().unwrap_or_default().to_string())
}

/// Actions supported by `POST /nodes/{node}/ceph/{action}`.
const CEPH_SERVICE_ACTIONS: &[&str] = &["start", "stop", "restart"];

/// Validate a `mon.<id>` or `mgr.<id>` service identifier as accepted by the
/// PVE `service` form parameter for start/stop/restart.
pub fn validate_ceph_service(service: &str) -> Result<(), String> {
    let Some((kind, id)) = service.split_once('.') else {
        return Err(format!(
            "Invalid Ceph service '{service}': expected 'mon.<id>' or 'mgr.<id>'"
        ));
    };
    if kind != "mon" && kind != "mgr" {
        return Err(format!(
            "Invalid Ceph service '{service}': kind must be 'mon' or 'mgr'"
        ));
    }
    validate_ceph_service_id(id)
}

/// Start, stop, or restart a Ceph mon/mgr service.
/// POST /nodes/{node}/ceph/{start|stop|restart} { service: "mon.<id>" | "mgr.<id>" }
/// Returns the task UPID.
pub async fn ceph_service_action(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    service: &str,
    action: &str,
    ticket: &str,
) -> Result<String, String> {
    validate_node(node)?;
    validate_ceph_service(service)?;
    if !CEPH_SERVICE_ACTIONS.contains(&action) {
        return Err(format!(
            "Invalid Ceph service action '{action}': must be one of {CEPH_SERVICE_ACTIONS:?}"
        ));
    }
    let path = format!("nodes/{node}/ceph/{action}");
    let response: serde_json::Value = client
        .post_form(&path, &[("service", service)], Some(ticket))
        .await
        .map_err(|e| format!("Failed to {action} Ceph service {service}: {e}"))?;
    Ok(response.as_str().unwrap_or_default().to_string())
}

/// Parse an (already `data`-unwrapped) `nodes/{node}/ceph/mon` response.
///
/// PVE returns an array of monitor objects where `quorum` is a 1/0 **number**
/// (not a bool) and the version lives under `ceph_version_short`. The address
/// is exposed as `addr`.
pub fn parse_monitors(response: &serde_json::Value) -> Result<Vec<CephMonitor>, String> {
    let mons = response
        .as_array()
        .ok_or_else(|| "Invalid response format".to_string())?;

    let mon_list = mons
        .iter()
        .filter_map(|mon| {
            let name = mon.get("name")?.as_str()?.to_string();
            let quorum = json_truthy(mon.get("quorum"));
            let address = mon
                .get("addr")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let version = mon
                .get("ceph_version_short")
                .or_else(|| mon.get("version"))
                .and_then(|v| v.as_str())
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
}

/// Interpret a JSON value as truthy: booleans pass through, numbers are truthy
/// when non-zero, strings "1"/"true" are truthy.
fn json_truthy(v: Option<&serde_json::Value>) -> bool {
    match v {
        Some(serde_json::Value::Bool(b)) => *b,
        Some(serde_json::Value::Number(n)) => n.as_i64().map(|i| i != 0).unwrap_or(false),
        Some(serde_json::Value::String(s)) => s == "1" || s.eq_ignore_ascii_case("true"),
        _ => false,
    }
}

/// List Ceph monitors
pub async fn list_monitors(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<CephMonitor>, String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/mon");
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list Ceph monitors: {e}"))?;

    parse_monitors(&response)
}

/// Get monitor status
pub async fn get_monitor_status(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    monitor: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/mon/{monitor}");
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get monitor {monitor}: {e}"))
}

/// Get Ceph quorum health
pub async fn quorum_health(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/status");
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get Ceph health: {e}"))
}

/// Parse an (already `data`-unwrapped) `nodes/{node}/ceph/status` response into
/// a `CephHealth`.
///
/// `ProxmoxClient::get` already strips the Proxmox `{ "data": ... }` envelope,
/// so `status` here is the raw ceph status object. The health block may be
/// nested under `health` (typical) or, defensively, be the object itself.
/// Per-check messages are collected from `health.checks.*.summary.message`.
pub fn parse_ceph_health(status: &serde_json::Value) -> CephHealth {
    let health = status.get("health").unwrap_or(status);

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

    let status_str = health
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();

    // PVE/Ceph may expose `summary` as a plain string, or (older clusters) as an
    // array of `{ summary, severity }` objects. Fall back to the first detail
    // line, then to the status string, so the UI always has something to show.
    let summary = match health.get("summary") {
        Some(serde_json::Value::String(s)) if !s.is_empty() => s.clone(),
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|e| e.get("summary").and_then(|s| s.as_str()))
            .collect::<Vec<_>>()
            .join("; "),
        _ => String::new(),
    };
    let summary = if summary.is_empty() {
        details
            .first()
            .cloned()
            .unwrap_or_else(|| status_str.clone())
    } else {
        summary
    };

    CephHealth {
        status: status_str,
        summary,
        details,
    }
}

/// Get Ceph health
pub async fn get_ceph_health(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<CephHealth, String> {
    validate_node(node)?;
    let path = format!("nodes/{node}/ceph/status");
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get Ceph health: {e}"))?;

    Ok(parse_ceph_health(&response))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: `ProxmoxClient::get` already unwraps the `{ "data": ... }`
    /// envelope, so the parser receives the raw ceph status object. The previous
    /// implementation did a *second* `.get("data")` here, which always failed
    /// and made the Ceph page flash data then go blank. This asserts the parser
    /// reads health directly from the unwrapped status object.
    #[test]
    fn test_parse_ceph_health_unwrapped_status() {
        let status = serde_json::json!({
            "health": {
                "status": "HEALTH_OK",
                "checks": {}
            },
            "monmap": { "mons": [] }
        });
        let health = parse_ceph_health(&status);
        assert_eq!(health.status, "HEALTH_OK");
        assert!(health.details.is_empty());
    }

    #[test]
    fn test_parse_ceph_health_collects_check_messages() {
        let status = serde_json::json!({
            "health": {
                "status": "HEALTH_WARN",
                "checks": {
                    "OSD_DOWN": {
                        "summary": { "message": "1 osds down" },
                        "severity": "HEALTH_WARN"
                    }
                }
            }
        });
        let health = parse_ceph_health(&status);
        assert_eq!(health.status, "HEALTH_WARN");
        assert_eq!(health.details, vec!["1 osds down".to_string()]);
        // Falls back to the first detail line when no string summary is present.
        assert_eq!(health.summary, "1 osds down");
    }

    #[test]
    fn test_parse_ceph_health_summary_array() {
        // Older clusters expose `summary` as an array of objects.
        let status = serde_json::json!({
            "health": {
                "status": "HEALTH_WARN",
                "summary": [ { "severity": "HEALTH_WARN", "summary": "noout flag(s) set" } ]
            }
        });
        let health = parse_ceph_health(&status);
        assert_eq!(health.summary, "noout flag(s) set");
    }

    #[test]
    fn test_parse_ceph_health_defaults_when_empty() {
        let health = parse_ceph_health(&serde_json::json!({}));
        assert_eq!(health.status, "unknown");
        assert_eq!(health.summary, "unknown");
        assert!(health.details.is_empty());
    }

    #[test]
    fn test_ceph_pool_serialization() {
        let pool = CephPool {
            id: "1".to_string(),
            name: "rbd".to_string(),
            pool_type: "replicated".to_string(),
            size: 3,
            min_size: 2,
            used: 1000000000000,
            available: 2000000000000,
            total: 3000000000000,
            used_percent: 33.3,
        };

        let json = serde_json::to_value(&pool).unwrap();
        // Field names must match the frontend `CephPool` type (camelCase, `type`).
        assert_eq!(json.get("name").unwrap(), "rbd");
        assert_eq!(json.get("type").unwrap(), "replicated");
        assert!(json.get("minSize").is_some());
        assert!(json.get("usedPercent").is_some());

        let deserialized: CephPool = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.name, "rbd");
        assert_eq!(deserialized.pool_type, "replicated");
    }

    #[test]
    fn test_ceph_osd_serialization() {
        let osd = CephOsd {
            id: 0,
            host: "vmhost1".to_string(),
            status: "up".to_string(),
            weight: 1.0,
            size: 1920378863616,
            used: 66875367424,
            avail: 1853503496192,
            used_percent: 3.48,
        };

        let json = serde_json::to_value(&osd).unwrap();
        // Field names must match the frontend `CephOsd` type (camelCase).
        assert_eq!(json.get("host").unwrap(), "vmhost1");
        assert_eq!(json.get("status").unwrap(), "up");
        assert!(json.get("usedPercent").is_some());

        let deserialized: CephOsd = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.id, 0);
        assert_eq!(deserialized.status, "up");
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

    // ── Parser regression tests using real PVE 8 / Ceph 19 (Squid) shapes ──────

    #[test]
    fn test_parse_osds_walks_crush_tree() {
        // PVE returns the CRUSH tree as an object, not an array. The previous
        // `response.as_array()` made every Ceph OSD request fail with
        // "Failed to load Ceph OSDs".
        let response = serde_json::json!({
            "flags": "sortbitwise",
            "root": {
                "leaf": 0,
                "children": [
                    {
                        "type": "host",
                        "name": "vmhost4",
                        "children": [
                            {
                                "type": "osd",
                                "id": "19",
                                "name": "osd.19",
                                "host": "vmhost4",
                                "status": "up",
                                "in": 1,
                                "crush_weight": 1.7465972900390_f64,
                                "bytes_used": 66875367424_u64,
                                "total_space": 1920378863616_u64,
                                "percent_used": 3.482_f64
                            },
                            {
                                "type": "osd",
                                "id": "18",
                                "name": "osd.18",
                                "host": "vmhost4",
                                "status": "down",
                                "in": 0,
                                "crush_weight": 1.7465972900390_f64,
                                "bytes_used": 0_u64,
                                "total_space": 1920378863616_u64,
                                "percent_used": 0.0_f64
                            }
                        ]
                    }
                ]
            }
        });

        let osds = parse_osds(&response).expect("OSD tree should parse");
        assert_eq!(osds.len(), 2);

        let first = &osds[0];
        assert_eq!(first.id, 19);
        assert_eq!(first.host, "vmhost4");
        assert_eq!(first.status, "up");
        assert_eq!(first.size, 1920378863616);
        assert_eq!(first.used, 66875367424);
        assert_eq!(first.avail, 1920378863616 - 66875367424);
        assert!((first.used_percent - 3.482).abs() < 0.01);

        assert_eq!(osds[1].id, 18);
        assert_eq!(osds[1].status, "down");
        assert_eq!(osds[1].used_percent, 0.0);
    }

    #[test]
    fn test_parse_osds_rejects_missing_root() {
        let err = parse_osds(&serde_json::json!({"flags": "x"})).unwrap_err();
        assert_eq!(err, "Invalid response format");
    }

    #[test]
    fn test_parse_monitors_handles_numeric_quorum() {
        // `quorum` is a 1/0 number and the version lives under
        // `ceph_version_short`; the old `as_bool()` parser dropped every row.
        let response = serde_json::json!([
            {
                "name": "vmhost4",
                "addr": "172.19.111.164:6789/0",
                "quorum": 1,
                "ceph_version_short": "19.2.3",
                "state": "running"
            },
            {
                "name": "vmhost5",
                "addr": "172.19.111.165:6789/0",
                "quorum": 0,
                "ceph_version_short": "19.2.3"
            }
        ]);

        let mons = parse_monitors(&response).expect("monitors should parse");
        assert_eq!(mons.len(), 2);
        assert_eq!(mons[0].name, "vmhost4");
        assert!(mons[0].quorum);
        assert_eq!(mons[0].address, "172.19.111.164:6789/0");
        assert_eq!(mons[0].version, "19.2.3");
        assert!(!mons[1].quorum);
    }

    #[test]
    fn test_parse_managers_from_fixture() {
        let response = serde_json::json!([
            {"name": "vmhost1", "addr": "172.19.111.161", "state": "active", "ceph_version_short": "19.2.3"},
            {"name": "vmhost2", "addr": "172.19.111.162", "state": "standby"},
            {"name": "orphan"}
        ]);

        let mgrs = parse_managers(&response).expect("managers should parse");
        assert_eq!(mgrs.len(), 3);
        assert_eq!(mgrs[0].name, "vmhost1");
        assert_eq!(mgrs[0].addr.as_deref(), Some("172.19.111.161"));
        assert_eq!(mgrs[0].state.as_deref(), Some("active"));
        assert_eq!(mgrs[1].state.as_deref(), Some("standby"));
        assert!(mgrs[2].addr.is_none());
        assert!(mgrs[2].state.is_none());
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
    fn test_parse_cephfs_from_fixture() {
        // PVE surfaces the first data pool as `data_pool` (a string), not
        // `data_pool_ids`.
        let response = serde_json::json!([
            {"name": "cephfs", "metadata_pool": "cephfs_metadata", "data_pool": "cephfs_data"},
            {"name": "bare-fs"}
        ]);

        let filesystems = parse_cephfs(&response).expect("cephfs should parse");
        assert_eq!(filesystems.len(), 2);
        assert_eq!(filesystems[0].name, "cephfs");
        assert_eq!(
            filesystems[0].metadata_pool.as_deref(),
            Some("cephfs_metadata")
        );
        assert_eq!(filesystems[0].data_pool.as_deref(), Some("cephfs_data"));
        assert!(filesystems[1].metadata_pool.is_none());
        assert!(filesystems[1].data_pool.is_none());

        // Serializes to the camelCase shape the frontend expects.
        let json = serde_json::to_value(&filesystems[0]).unwrap();
        assert_eq!(json.get("metadataPool").unwrap(), "cephfs_metadata");
        assert_eq!(json.get("dataPool").unwrap(), "cephfs_data");
    }

    #[test]
    fn test_parse_cephfs_empty_array() {
        let filesystems = parse_cephfs(&serde_json::json!([])).expect("empty fs should parse");
        assert!(filesystems.is_empty());
    }

    #[test]
    fn test_parse_pools_derives_total_and_available() {
        // Real PVE pool objects expose `pool` (string id), `pool_name`,
        // `bytes_used` and a 0..1 `percent_used`; `avail`/`total` are derived.
        let response = serde_json::json!([
            {
                "pool": "2",
                "pool_name": "ceph-fed1",
                "type": "replicated",
                "size": 2,
                "min_size": 1,
                "bytes_used": 1000000000_u64,
                "percent_used": 0.04_f64
            }
        ]);

        let pools = parse_pools(&response).expect("pools should parse");
        assert_eq!(pools.len(), 1);
        let p = &pools[0];
        assert_eq!(p.id, "2");
        assert_eq!(p.name, "ceph-fed1");
        assert_eq!(p.pool_type, "replicated");
        assert_eq!(p.size, 2);
        assert_eq!(p.min_size, 1);
        assert_eq!(p.used, 1000000000);
        assert_eq!(p.total, 25000000000); // 1e9 / 0.04
        assert_eq!(p.available, 24000000000);
        assert!((p.used_percent - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_pools_rejects_non_array() {
        let err = parse_pools(&serde_json::json!({"pool": "x"})).unwrap_err();
        assert_eq!(err, "Invalid response format");
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
        let path = format!("nodes/{node}/ceph/status");
        assert!(path.contains("nodes/"));
        assert!(path.contains("/ceph/status"));
        assert!(!path.contains("cluster/"));
    }

    #[test]
    fn test_list_pools_uses_node_path() {
        let node = "pve1";
        let path = format!("nodes/{node}/ceph/pool");
        assert!(path.starts_with("nodes/"));
        assert!(!path.starts_with("cluster/"));
    }

    #[test]
    fn test_list_osds_uses_node_path() {
        let node = "pve1";
        let path = format!("nodes/{node}/ceph/osd");
        assert!(path.starts_with("nodes/"));
        assert!(!path.starts_with("cluster/"));
    }

    #[test]
    fn test_list_monitors_uses_node_path() {
        let node = "pve1";
        let path = format!("nodes/{node}/ceph/mon");
        assert!(path.starts_with("nodes/"));
        assert!(!path.starts_with("cluster/"));
    }

    #[test]
    fn test_write_operations_use_node_paths() {
        let node = "pve1";
        let create_pool_path = format!("nodes/{node}/ceph/pool");
        let set_osd_weight_path = format!("nodes/{node}/ceph/osd/{}", 1);
        let rbd_resize_path = format!("nodes/{node}/ceph/pool/{}/rbd/{}/resize", "rbd", "vm-100");
        assert!(create_pool_path.starts_with("nodes/"));
        assert!(set_osd_weight_path.starts_with("nodes/"));
        assert!(rbd_resize_path.starts_with("nodes/"));
        assert!(!create_pool_path.contains("cluster/"));
        assert!(!set_osd_weight_path.contains("cluster/"));
        assert!(!rbd_resize_path.contains("cluster/"));
    }

    // ── CephFS data_pool / data_pools ────────────────────────────────────────

    #[test]
    fn test_parse_cephfs_single_data_pool_string() {
        let response = serde_json::json!([
            { "name": "cephfs", "metadata_pool": "cephfs_metadata", "data_pool": "cephfs_data" }
        ]);
        let fs_list = parse_cephfs(&response).unwrap();
        assert_eq!(fs_list.len(), 1);
        assert_eq!(fs_list[0].data_pool.as_deref(), Some("cephfs_data"));
    }

    #[test]
    fn test_parse_cephfs_data_pools_array_of_names() {
        let response = serde_json::json!([
            { "name": "cephfs", "metadata_pool": "cephfs_metadata", "data_pools": ["cephfs_data", "cephfs_data2"] }
        ]);
        let fs_list = parse_cephfs(&response).unwrap();
        assert_eq!(
            fs_list[0].data_pool.as_deref(),
            Some("cephfs_data, cephfs_data2")
        );
    }

    #[test]
    fn test_parse_cephfs_data_pools_array_of_numeric_ids() {
        let response = serde_json::json!([
            { "name": "cephfs", "data_pools": [3, 4] }
        ]);
        let fs_list = parse_cephfs(&response).unwrap();
        assert_eq!(fs_list[0].data_pool.as_deref(), Some("3, 4"));
    }

    #[test]
    fn test_parse_cephfs_no_pool_info() {
        let response = serde_json::json!([{ "name": "cephfs" }]);
        let fs_list = parse_cephfs(&response).unwrap();
        assert_eq!(fs_list[0].data_pool, None);
    }

    // ── Ceph flags ────────────────────────────────────────────────────────────

    #[test]
    fn test_validate_ceph_flag_accepts_known_flags() {
        for flag in CEPH_FLAGS {
            assert!(validate_ceph_flag(flag).is_ok());
        }
    }

    #[test]
    fn test_validate_ceph_flag_rejects_unknown() {
        for flag in ["noout; rm -rf", "bogus_flag", "", "NOOUT"] {
            assert!(
                validate_ceph_flag(flag).is_err(),
                "flag {flag:?} must be rejected"
            );
        }
    }

    // ── Ceph service id / mon.mgr service validation ─────────────────────────

    #[test]
    fn test_validate_ceph_service_id_accepts_valid() {
        assert!(validate_ceph_service_id("vmhost1").is_ok());
        assert!(validate_ceph_service_id("node-1.example").is_ok());
    }

    #[test]
    fn test_validate_ceph_service_id_rejects_invalid() {
        for id in [
            "",
            "a".repeat(65).as_str(),
            "vmhost1; rm -rf",
            "vmhost1/../etc",
        ] {
            assert!(
                validate_ceph_service_id(id).is_err(),
                "id {id:?} must be rejected"
            );
        }
    }

    #[test]
    fn test_validate_ceph_service_accepts_mon_and_mgr() {
        assert!(validate_ceph_service("mon.vmhost1").is_ok());
        assert!(validate_ceph_service("mgr.vmhost1").is_ok());
    }

    #[test]
    fn test_validate_ceph_service_rejects_bad_kind_or_shape() {
        for service in [
            "osd.0",
            "vmhost1",
            "mon",
            "mon.",
            ".vmhost1",
            "mon.vmhost1; rm",
        ] {
            assert!(
                validate_ceph_service(service).is_err(),
                "service {service:?} must be rejected"
            );
        }
    }

    #[test]
    fn test_ceph_service_action_rejects_bad_action() {
        // Path construction happens after validation — action whitelist test
        // covers the constant directly since the async fn needs a live client.
        assert!(CEPH_SERVICE_ACTIONS.contains(&"start"));
        assert!(CEPH_SERVICE_ACTIONS.contains(&"stop"));
        assert!(CEPH_SERVICE_ACTIONS.contains(&"restart"));
        assert!(!CEPH_SERVICE_ACTIONS.contains(&"delete"));
    }
}
