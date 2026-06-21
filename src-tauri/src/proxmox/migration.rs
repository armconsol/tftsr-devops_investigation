// Remote Migration module
// Provides operations for cross-cluster VM migration

use serde::{Deserialize, Serialize};

/// Migration task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationTask {
    pub task_id: String,
    pub vm_id: u32,
    pub source_node: String,
    pub target_node: String,
    pub source_cluster: String,
    pub target_cluster: String,
    pub status: String,
    pub progress: u32,
    pub start_time: String,
    pub end_time: Option<String>,
    pub error: Option<String>,
}

/// Migration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    pub task_id: String,
    pub status: String,
    pub progress: u32,
    pub bytes_transferred: u64,
    pub bytes_remaining: u64,
    pub downtime: u64,
}

/// Migrate VM to remote cluster
pub async fn migrate_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vm_id: u32,
    target_node: &str,
    target_cluster: &str,
    ticket: &str,
) -> Result<MigrationTask, String> {
    let path = format!("nodes/{}/qemu/{}/migrate", node, vm_id);
    let config = serde_json::json!({
        "target": target_node,
        "targetcluster": target_cluster,
        "targetstorage": "",
        "online": true,
        "force": false
    });

    let response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to migrate VM {}: {}", vm_id, e))?;

    {
        let data = &response;
        let task_id = data
            .get("taskid")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        let status = data
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("running")
            .to_string();
        let progress = data.get("progress").and_then(|p| p.as_u64()).unwrap_or(0) as u32;
        let start_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        Ok(MigrationTask {
            task_id,
            vm_id,
            source_node: node.to_string(),
            target_node: target_node.to_string(),
            source_cluster: client.base_url().to_string(),
            target_cluster: target_cluster.to_string(),
            status,
            progress,
            start_time,
            end_time: None,
            error: None,
        })
    }
}

/// List migration tasks
pub async fn list_migration_status(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<MigrationTask>, String> {
    let path = format!("nodes/{}/tasks", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list migration tasks for node {}: {}", node, e))?;

    if let Some(tasks) = response.as_array() {
        let task_list: Vec<MigrationTask> = tasks
            .iter()
            .filter_map(|task| {
                let id = task.get("id")?.as_str()?.to_string();
                let vm_id = task
                    .get("vmid")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)?;
                let status = task
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let progress = task.get("progress").and_then(|p| p.as_u64()).unwrap_or(0) as u32;
                let start_time = task
                    .get("starttime")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                let end_time = task
                    .get("endtime")
                    .and_then(|e| e.as_str())
                    .map(|e| e.to_string());
                let error = task
                    .get("exitstatus")
                    .and_then(|e| e.as_str())
                    .filter(|e| !e.is_empty())
                    .map(|e| e.to_string());

                Some(MigrationTask {
                    task_id: id,
                    vm_id,
                    source_node: node.to_string(),
                    target_node: "".to_string(),
                    source_cluster: "".to_string(),
                    target_cluster: "".to_string(),
                    status,
                    progress,
                    start_time,
                    end_time,
                    error,
                })
            })
            .collect();

        Ok(task_list)
    } else {
        Ok(vec![])
    }
}

/// Get migration task status
pub async fn get_migration_task_status(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    task_id: &str,
    ticket: &str,
) -> Result<MigrationStatus, String> {
    let path = format!("nodes/{}/tasks/{}", node, task_id);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get migration task {}: {}", task_id, e))?;

    {
        let data = &response;
        let status = data
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();
        let progress = data.get("progress").and_then(|p| p.as_u64()).unwrap_or(0) as u32;
        let bytes_transferred = data
            .get("bytes_transferred")
            .and_then(|b| b.as_u64())
            .unwrap_or(0);
        let bytes_remaining = data
            .get("bytes_remaining")
            .and_then(|b| b.as_u64())
            .unwrap_or(0);
        let downtime = data.get("downtime").and_then(|d| d.as_u64()).unwrap_or(0);

        Ok(MigrationStatus {
            task_id: task_id.to_string(),
            status,
            progress,
            bytes_transferred,
            bytes_remaining,
            downtime,
        })
    }
}

/// Cancel migration task
pub async fn cancel_migration(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vm_id: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu/{}/migrate", node, vm_id);
    let config = serde_json::json!({
        "cancel": true
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to cancel migration for VM {}: {}", vm_id, e))?;
    Ok(())
}
