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

/// One line of a PVE task log. Entries only ever carry a line number `n` and
/// the full line text `t` — there is no separate level/message split.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskLogEntry {
    pub n: u64,
    pub t: String,
}

/// Validate a Proxmox task UPID (`UPID:<node>:...`). Rejects anything that
/// could be used for path traversal or injection when interpolated into a
/// request path.
pub fn validate_upid(upid: &str) -> Result<(), String> {
    if !upid.starts_with("UPID:") {
        return Err(format!(
            "Invalid task UPID '{upid}': must start with 'UPID:'"
        ));
    }
    if upid.len() > 256 {
        return Err("Invalid task UPID: too long".to_string());
    }
    let valid_chars = upid
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '-' | '_' | '.' | '@'));
    if !valid_chars {
        return Err(format!(
            "Invalid task UPID '{upid}': contains illegal characters"
        ));
    }
    Ok(())
}

/// Case-insensitive substring search over task log lines.
pub fn filter_log_lines(entries: &[TaskLogEntry], query: &str) -> Vec<TaskLogEntry> {
    let needle = query.to_lowercase();
    entries
        .iter()
        .filter(|e| e.t.to_lowercase().contains(&needle))
        .cloned()
        .collect()
}

/// List tasks for a node
pub async fn list_tasks(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<TaskInfo>, String> {
    let path = format!("nodes/{node}/tasks");
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list tasks for node {node}: {e}"))?;

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
    let path = format!("nodes/{node}/tasks/{task_id}");
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get task {task_id}: {e}"))?;

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
    let path = format!("nodes/{node}/tasks/{task_id}");
    let config = serde_json::json!({
        "cancel": true
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to stop task {task_id}: {e}"))?;
    Ok(())
}

/// Parse the (envelope-unwrapped) response of
/// GET /nodes/{node}/tasks/{upid}/log, an array of `{n, t}` objects.
pub fn parse_task_log_entries(response: &serde_json::Value) -> Result<Vec<TaskLogEntry>, String> {
    let arr = response
        .as_array()
        .ok_or_else(|| "Invalid task log response format".to_string())?;

    arr.iter()
        .map(|entry| {
            serde_json::from_value::<TaskLogEntry>(entry.clone())
                .map_err(|e| format!("Failed to parse task log entry: {e}"))
        })
        .collect()
}

/// Get task log
pub async fn get_task_log(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    task_id: &str,
    ticket: &str,
) -> Result<Vec<TaskLogEntry>, String> {
    validate_upid(task_id)?;
    let path = format!("nodes/{node}/tasks/{task_id}/log");
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get task log for {task_id}: {e}"))?;

    parse_task_log_entries(&response)
}

/// Forward task to remote
pub async fn forward_task(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    target_node: &str,
    task_id: &str,
    ticket: &str,
) -> Result<TaskInfo, String> {
    let path = format!("nodes/{node}/tasks/{task_id}/forward");
    let config = serde_json::json!({
        "target": target_node
    });

    let response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to forward task {task_id}: {e}"))?;

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
            description: format!("Forwarded to {target_node}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_upid_accepts_well_formed() {
        assert!(
            validate_upid("UPID:vmhost1:00001234:0000ABCD:00000000:aptupdate::root@pam:").is_ok()
        );
    }

    #[test]
    fn test_validate_upid_rejects_bad_prefix_and_traversal() {
        for upid in [
            "not-a-upid",
            "",
            "UPID:vmhost1/../etc/passwd",
            "UPID:vmhost1; rm -rf /",
        ] {
            assert!(
                validate_upid(upid).is_err(),
                "upid {upid:?} must be rejected"
            );
        }
    }

    #[test]
    fn test_parse_task_log_entries_basic() {
        let response = serde_json::json!([
            {"n": 1, "t": "starting update"},
            {"n": 2, "t": "update complete"}
        ]);
        let entries = parse_task_log_entries(&response).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].n, 1);
        assert_eq!(entries[1].t, "update complete");
    }

    #[test]
    fn test_parse_task_log_entries_rejects_non_array() {
        assert!(parse_task_log_entries(&serde_json::json!({})).is_err());
    }

    #[test]
    fn test_parse_task_log_entries_missing_field_errors() {
        assert!(parse_task_log_entries(&serde_json::json!([{"n": 1}])).is_err());
    }

    #[test]
    fn test_filter_log_lines_case_insensitive_substring() {
        let entries = vec![
            TaskLogEntry {
                n: 1,
                t: "Starting APT update".to_string(),
            },
            TaskLogEntry {
                n: 2,
                t: "Nothing to do".to_string(),
            },
            TaskLogEntry {
                n: 3,
                t: "update complete".to_string(),
            },
        ];
        let matches = filter_log_lines(&entries, "UPDATE");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].n, 1);
        assert_eq!(matches[1].n, 3);
    }

    #[test]
    fn test_filter_log_lines_no_match() {
        let entries = vec![TaskLogEntry {
            n: 1,
            t: "hello".to_string(),
        }];
        assert!(filter_log_lines(&entries, "xyz").is_empty());
    }

    #[test]
    fn test_filter_log_lines_unicode() {
        let entries = vec![TaskLogEntry {
            n: 1,
            t: "café is running".to_string(),
        }];
        assert_eq!(filter_log_lines(&entries, "CAFÉ").len(), 1);
    }
}
