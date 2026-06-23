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
    node: &str,
    ticket: &str,
) -> Result<Vec<BackupJob>, String> {
    let path = format!("nodes/{}/backup/jobs", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list backup jobs: {}", e))?;

    if let Some(jobs) = response.get("data").and_then(|d| d.as_array()) {
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
        Err("Invalid response format: missing 'data' field".to_string())
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

/// Trigger backup job manually
pub async fn trigger_backup_job(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    job_id: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/backup/jobs/{}/run", node, job_id);
    let _response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to trigger backup job {}: {}", job_id, e))?;
    Ok(())
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

    if let Some(datastores) = response.get("data").and_then(|d| d.as_array()) {
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
        Err("Invalid response format: missing 'data' field".to_string())
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

    let ds = response.get("data").ok_or("Invalid response format")?;

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

    if let Some(snapshots) = response.get("data").and_then(|d| d.as_array()) {
        Ok(snapshots.to_vec())
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
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
}
