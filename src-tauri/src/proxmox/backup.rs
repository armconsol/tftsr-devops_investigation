// Backup management module
// Provides operations for managing Proxmox Backup Server

use serde::{Deserialize, Serialize};

/// Backup job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupJob {
    pub job_id: u32,
    pub name: String,
    pub schedule: String,
    pub enabled: bool,
    pub datastore: String,
    pub source: String,
    pub retention: String,
}

/// Datastore information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatastoreInfo {
    pub datastore: String,
    pub node: String,
    pub size: u64,
    pub used: u64,
    pub available: u64,
    pub status: String,
}

/// List backup jobs
pub async fn list_backup_jobs(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<BackupJob>, String> {
    let path = "cluster/backup";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list backup jobs: {}", e))?;

    if let Some(jobs) = response.as_array() {
        let backup_jobs: Vec<BackupJob> = jobs
            .iter()
            .filter_map(|job| {
                let job_id = job.get("jobid")?.as_u64()?;
                let name = job.get("name")?.as_str()?.to_string();
                let schedule = job.get("schedule")?.as_str()?.to_string();
                let enabled = job.get("enabled")?.as_bool()?;
                let datastore = job.get("datastore")?.as_str()?.to_string();
                let source = job.get("source")?.as_str()?.to_string();
                let retention = job.get("retention")?.as_str().unwrap_or("").to_string();

                Some(BackupJob {
                    job_id: job_id as u32,
                    name,
                    schedule,
                    enabled,
                    datastore,
                    source,
                    retention,
                })
            })
            .collect();

        Ok(backup_jobs)
    } else {
        Err("Invalid response format".to_string())
    }
}

/// Create backup job
pub async fn create_backup_job(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    job: &BackupJob,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/backup/jobs", node);
    let config = serde_json::json!({
        "jobid": job.job_id,
        "name": job.name,
        "schedule": job.schedule,
        "enabled": job.enabled,
        "datastore": job.datastore,
        "source": job.source,
        "retention": job.retention
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create backup job {}: {}", job.job_id, e))?;
    Ok(())
}

/// Update backup job
pub async fn update_backup_job(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    job_id: u32,
    job: &BackupJob,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/backup/jobs/{}", node, job_id);
    let config = serde_json::json!({
        "name": job.name,
        "schedule": job.schedule,
        "enabled": job.enabled,
        "datastore": job.datastore,
        "source": job.source,
        "retention": job.retention
    });

    let _response: serde_json::Value = client
        .put(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update backup job {}: {}", job_id, e))?;
    Ok(())
}

/// Delete backup job
pub async fn delete_backup_job(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    job_id: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/backup/jobs/{}", node, job_id);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete backup job {}: {}", job_id, e))?;
    Ok(())
}

/// Build `vzdump` form parameters from a `cluster/backup` job config object.
///
/// PVE has no "run job by id" REST endpoint; the web UI's "Run now" reads the
/// job's vzdump options and POSTs them to `nodes/{node}/vzdump`. We mirror that:
/// translate the stored job config into vzdump params. When the job targets all
/// guests (`vmid` empty or `"all"`) we send `all=1`; otherwise the explicit
/// `vmid` list. Common optional fields are forwarded when present.
pub fn build_vzdump_params(job: &serde_json::Value) -> Vec<(String, String)> {
    let mut params: Vec<(String, String)> = Vec::new();

    if let Some(storage) = job.get("storage").and_then(|v| v.as_str()) {
        if !storage.is_empty() {
            params.push(("storage".to_string(), storage.to_string()));
        }
    }
    if let Some(mode) = job.get("mode").and_then(|v| v.as_str()) {
        if !mode.is_empty() {
            params.push(("mode".to_string(), mode.to_string()));
        }
    }

    let vmid = job
        .get("vmid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    if vmid.is_empty() || vmid == "all" {
        params.push(("all".to_string(), "1".to_string()));
    } else {
        params.push(("vmid".to_string(), vmid.to_string()));
    }

    // Forward common optional fields when the job defines them.
    for key in ["compress", "exclude", "mailnotification", "notes-template"] {
        if let Some(val) = job.get(key).and_then(|v| v.as_str()) {
            if !val.is_empty() {
                params.push((key.to_string(), val.to_string()));
            }
        }
    }

    params
}

/// Pick the node to run a manual backup on: the job's configured node (first
/// entry if comma-separated) if set, otherwise the first available cluster node.
pub fn select_backup_node(job: &serde_json::Value, cluster_nodes: &[String]) -> Option<String> {
    if let Some(node) = job.get("node").and_then(|v| v.as_str()) {
        if let Some(first) = node.split(',').map(str::trim).find(|n| !n.is_empty()) {
            return Some(first.to_string());
        }
    }
    cluster_nodes.first().cloned()
}

/// List datastores
pub async fn list_datastores(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<DatastoreInfo>, String> {
    let path = "datastore";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list datastores: {}", e))?;

    if let Some(datastores) = response.as_array() {
        let datastore_list: Vec<DatastoreInfo> = datastores
            .iter()
            .filter_map(|ds| {
                let datastore = ds.get("datastore")?.as_str()?.to_string();
                let node = ds.get("node")?.as_str()?.to_string();
                let size = ds.get("size")?.as_u64()?;
                let used = ds.get("used")?.as_u64()?;
                let available = ds.get("available")?.as_u64()?;
                let status = ds.get("status")?.as_str()?.to_string();

                Some(DatastoreInfo {
                    datastore,
                    node,
                    size,
                    used,
                    available,
                    status,
                })
            })
            .collect();

        Ok(datastore_list)
    } else {
        Err("Invalid response format".to_string())
    }
}

/// Get datastore status
pub async fn get_datastore_status(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    datastore: &str,
    ticket: &str,
) -> Result<DatastoreInfo, String> {
    let path = format!("nodes/{}/backup/status?datastore={}", node, datastore);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get datastore status: {}", e))?;

    // response IS already the data (handle_response already unwrapped the envelope)
    let ds = &response;

    Ok(DatastoreInfo {
        datastore: datastore.to_string(),
        node: node.to_string(),
        size: ds.get("size").and_then(|s| s.as_u64()).unwrap_or(0),
        used: ds.get("used").and_then(|u| u.as_u64()).unwrap_or(0),
        available: ds.get("available").and_then(|a| a.as_u64()).unwrap_or(0),
        status: ds
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string(),
    })
}

/// List backup snapshots
pub async fn list_backup_snapshots(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    datastore: &str,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let path = format!("nodes/{}/backup/snapshots?datastore={}", node, datastore);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list backup snapshots: {}", e))?;

    if let Some(snapshots) = response.as_array() {
        Ok(snapshots.to_vec())
    } else {
        Err("Invalid response format".to_string())
    }
}

/// Restore backup
pub async fn restore_backup(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    datastore: &str,
    backup_id: &str,
    target_node: &str,
    target_vmid: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/backup/restore", node);
    let config = serde_json::json!({
        "datastore": datastore,
        "backup": backup_id,
        "target-node": target_node,
        "target-vmid": target_vmid
    });

    let _response: serde_json::Value =
        client
            .post(&path, &config, Some(ticket))
            .await
            .map_err(|e| {
                format!(
                    "Failed to restore backup {} to VM {}: {}",
                    backup_id, target_vmid, e
                )
            })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_job_serialization() {
        let job = BackupJob {
            job_id: 1,
            name: "daily-backup".to_string(),
            schedule: "0 2 * * *".to_string(),
            enabled: true,
            datastore: "pbs-datastore".to_string(),
            source: "/data".to_string(),
            retention: "30d".to_string(),
        };

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: BackupJob = serde_json::from_str(&json).unwrap();

        assert_eq!(job.job_id, deserialized.job_id);
        assert_eq!(job.name, deserialized.name);
        assert_eq!(job.enabled, deserialized.enabled);
    }

    #[test]
    fn test_datastore_info_serialization() {
        let ds = DatastoreInfo {
            datastore: "local".to_string(),
            node: "pbs-node-1".to_string(),
            size: 1000000000000,
            used: 300000000000,
            available: 700000000000,
            status: "available".to_string(),
        };

        let json = serde_json::to_string(&ds).unwrap();
        let deserialized: DatastoreInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(ds.datastore, deserialized.datastore);
        assert_eq!(ds.status, "available");
    }

    #[test]
    fn test_build_vzdump_params_all_guests() {
        let job = serde_json::json!({
            "id": "backup-local",
            "storage": "local",
            "mode": "snapshot",
            "vmid": "all"
        });
        let params = build_vzdump_params(&job);
        assert!(params.contains(&("storage".to_string(), "local".to_string())));
        assert!(params.contains(&("mode".to_string(), "snapshot".to_string())));
        assert!(params.contains(&("all".to_string(), "1".to_string())));
        // Must not also send an explicit vmid when backing up all guests.
        assert!(!params.iter().any(|(k, _)| k == "vmid"));
    }

    #[test]
    fn test_build_vzdump_params_explicit_vmids() {
        let job = serde_json::json!({
            "storage": "pbs",
            "mode": "stop",
            "vmid": "100,101"
        });
        let params = build_vzdump_params(&job);
        assert!(params.contains(&("vmid".to_string(), "100,101".to_string())));
        assert!(!params.iter().any(|(k, _)| k == "all"));
    }

    #[test]
    fn test_build_vzdump_params_missing_vmid_defaults_to_all() {
        let job = serde_json::json!({ "storage": "local" });
        let params = build_vzdump_params(&job);
        assert!(params.contains(&("all".to_string(), "1".to_string())));
    }

    #[test]
    fn test_build_vzdump_params_forwards_optional_fields() {
        let job = serde_json::json!({
            "storage": "local",
            "vmid": "100",
            "compress": "zstd",
            "exclude": "200"
        });
        let params = build_vzdump_params(&job);
        assert!(params.contains(&("compress".to_string(), "zstd".to_string())));
        assert!(params.contains(&("exclude".to_string(), "200".to_string())));
    }

    #[test]
    fn test_select_backup_node_prefers_job_node() {
        let job = serde_json::json!({ "node": "pve2" });
        let nodes = vec!["pve1".to_string(), "pve2".to_string()];
        assert_eq!(select_backup_node(&job, &nodes), Some("pve2".to_string()));
    }

    #[test]
    fn test_select_backup_node_takes_first_of_list() {
        let job = serde_json::json!({ "node": "pve2,pve3" });
        let nodes = vec!["pve1".to_string()];
        assert_eq!(select_backup_node(&job, &nodes), Some("pve2".to_string()));
    }

    #[test]
    fn test_select_backup_node_falls_back_to_cluster_node() {
        let job = serde_json::json!({ "storage": "local" });
        let nodes = vec!["pve1".to_string(), "pve2".to_string()];
        assert_eq!(select_backup_node(&job, &nodes), Some("pve1".to_string()));
    }

    #[test]
    fn test_select_backup_node_none_when_no_nodes() {
        let job = serde_json::json!({ "storage": "local" });
        assert_eq!(select_backup_node(&job, &[]), None);
    }
}
