// Task Management module
// Provides operations for managing remote tasks

use serde::{Deserialize, Serialize};

/// Task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub task_id: String,
    pub node: String,
    pub vm_id: Option<u32>,
    pub user: String,
    pub status: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub progress: u32,
    pub exit_status: Option<String>,
    pub description: String,
}

/// Task log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

/// List tasks for a node
pub async fn list_tasks(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<TaskInfo>, String> {
    let path = format!("nodes/{}/tasks", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list tasks for node {}: {}", node, e))?;

    if let Some(tasks) = response.as_array() {
        let task_list: Vec<TaskInfo> = tasks
            .iter()
            .filter_map(|task| {
                let id = task.get("id")?.as_str()?.to_string();
                let node_name = task
                    .get("node")
                    .and_then(|n| n.as_str())
                    .unwrap_or(node)
                    .to_string();
                let vm_id = task.get("vmid").and_then(|v| v.as_u64()).map(|v| v as u32);
                let user = task
                    .get("user")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                let status = task
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let start_time = task
                    .get("starttime")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                let end_time = task
                    .get("endtime")
                    .and_then(|e| e.as_str())
                    .map(|e| e.to_string());
                let progress = task.get("progress").and_then(|p| p.as_u64()).unwrap_or(0) as u32;
                let exit_status = task
                    .get("exitstatus")
                    .and_then(|e| e.as_str())
                    .filter(|e| !e.is_empty())
                    .map(|e| e.to_string());
                let description = task
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();

                Some(TaskInfo {
                    task_id: id,
                    node: node_name,
                    vm_id,
                    user,
                    status,
                    start_time,
                    end_time,
                    progress,
                    exit_status,
                    description,
                })
            })
            .collect();

        Ok(task_list)
    } else {
        Ok(vec![])
    }
}

/// Get task status
pub async fn get_task_status(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    task_id: &str,
    ticket: &str,
) -> Result<TaskInfo, String> {
    let path = format!("nodes/{}/tasks/{}", node, task_id);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get task {}: {}", task_id, e))?;

    {
        let data = &response;
        let id = data
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let node_name = data
            .get("node")
            .and_then(|n| n.as_str())
            .unwrap_or(node)
            .to_string();
        let vm_id = data.get("vmid").and_then(|v| v.as_u64()).map(|v| v as u32);
        let user = data
            .get("user")
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();
        let status = data
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();
        let start_time = data
            .get("starttime")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let end_time = data
            .get("endtime")
            .and_then(|e| e.as_str())
            .map(|e| e.to_string());
        let progress = data.get("progress").and_then(|p| p.as_u64()).unwrap_or(0) as u32;
        let exit_status = data
            .get("exitstatus")
            .and_then(|e| e.as_str())
            .filter(|e| !e.is_empty())
            .map(|e| e.to_string());
        let description = data
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();

        Ok(TaskInfo {
            task_id: id,
            node: node_name,
            vm_id,
            user,
            status,
            start_time,
            end_time,
            progress,
            exit_status,
            description,
        })
    }
}

/// Stop/cancel task
pub async fn stop_task(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    task_id: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/tasks/{}", node, task_id);
    let config = serde_json::json!({
        "cancel": true
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to stop task {}: {}", task_id, e))?;
    Ok(())
}

/// Get task log
pub async fn get_task_log(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    task_id: &str,
    ticket: &str,
) -> Result<Vec<TaskLogEntry>, String> {
    let path = format!("nodes/{}/tasks/{}/log", node, task_id);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get task log for {}: {}", task_id, e))?;

    if let Some(log_entries) = response.as_array() {
        let log_list: Vec<TaskLogEntry> = log_entries
            .iter()
            .map(|entry| {
                let timestamp = entry
                    .get("t")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                let level = entry
                    .get("l")
                    .and_then(|l| l.as_str())
                    .unwrap_or("info")
                    .to_string();
                let message = entry
                    .get("m")
                    .and_then(|m| m.as_str())
                    .unwrap_or("")
                    .to_string();

                TaskLogEntry {
                    timestamp,
                    level,
                    message,
                }
            })
            .collect();

        Ok(log_list)
    } else {
        Ok(vec![])
    }
}

/// Forward task to remote
pub async fn forward_task(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    target_node: &str,
    task_id: &str,
    ticket: &str,
) -> Result<TaskInfo, String> {
    let path = format!("nodes/{}/tasks/{}/forward", node, task_id);
    let config = serde_json::json!({
        "target": target_node
    });

    let response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to forward task {}: {}", task_id, e))?;

    {
        let data = &response;
        let id = data
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let status = data
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("running")
            .to_string();
        let start_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        Ok(TaskInfo {
            task_id: id,
            node: node.to_string(),
            vm_id: None,
            user: "".to_string(),
            status,
            start_time,
            end_time: None,
            progress: 0,
            exit_status: None,
            description: format!("Forwarded to {}", target_node),
        })
    }
}
