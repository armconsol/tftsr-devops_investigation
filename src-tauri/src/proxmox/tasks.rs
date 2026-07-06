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
///
/// `:` and `@` are intentionally allowed unencoded: a real PVE UPID always
/// contains both (e.g. `UPID:vmhost1:...::root@pam:`), and per RFC 3986's
/// path-segment grammar (`pchar = unreserved / pct-encoded / sub-delims /
/// ":" / "@"`) neither requires percent-encoding to appear literally in a
/// URL path segment — only `/` can introduce an unintended segment
/// boundary, and that (along with whitespace and shell metacharacters) is
/// excluded below.
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

/// Extract the task UPID from a PVE task object. PVE's task-list and
/// task-status responses key this as `upid`; `id` on those same objects is a
/// *different* field (the affected object's id, e.g. a VMID for a qemu
/// task) and must not be used here — every downstream call
/// (`get_task_status`, `stop_task`, `get_task_log`) interpolates this value
/// directly into `nodes/{node}/tasks/{task_id}`, which only accepts a UPID.
/// Falls back to `id` only if `upid` is absent, to tolerate any endpoint
/// variant that genuinely omits it.
fn extract_task_upid(value: &serde_json::Value) -> String {
    value
        .get("upid")
        .or_else(|| value.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
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
                let id = extract_task_upid(task);
                if id.is_empty() {
                    return None;
                }
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
        let id = extract_task_upid(data);
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
        let id = extract_task_upid(data);
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
    fn test_validate_upid_still_rejects_a_slash_which_would_add_a_path_segment() {
        // ':' and '@' are allowed (real UPIDs require them and neither can
        // introduce a new path segment per RFC 3986), but '/' must remain
        // rejected since it can.
        assert!(validate_upid("UPID:vmhost1:aaa:bbb::task::root@pam:/extra").is_err());
    }

    #[test]
    fn test_extract_task_upid_prefers_upid_over_id() {
        // PVE task-list/status entries carry both `id` (the affected
        // object's id, e.g. a VMID) and `upid` (the actual task identifier).
        // Downstream calls need `upid`.
        let entry = serde_json::json!({
            "id": "104",
            "upid": "UPID:vmhost1:00001234:0000ABCD:00000000:qmstart:104:root@pam:",
            "node": "vmhost1"
        });
        assert_eq!(
            extract_task_upid(&entry),
            "UPID:vmhost1:00001234:0000ABCD:00000000:qmstart:104:root@pam:"
        );
    }

    #[test]
    fn test_extract_task_upid_falls_back_to_id_when_upid_absent() {
        let entry = serde_json::json!({ "id": "UPID:vmhost1:x:y:z:task::root@pam:" });
        assert_eq!(
            extract_task_upid(&entry),
            "UPID:vmhost1:x:y:z:task::root@pam:"
        );
    }

    #[test]
    fn test_extract_task_upid_missing_both_returns_empty() {
        assert_eq!(extract_task_upid(&serde_json::json!({})), "");
    }

    #[test]
    fn test_list_tasks_field_mapping_uses_upid_not_id() {
        // Regression test for the id/upid field mismatch: list_tasks must
        // populate TaskInfo.task_id from "upid", not the object-id "id"
        // field, since get_task_status/stop_task/get_task_log all
        // interpolate task_id directly into nodes/{node}/tasks/{task_id}.
        let pve_response = serde_json::json!([
            {
                "id": "104",
                "upid": "UPID:vmhost1:00001234:0000ABCD:00000000:qmstart:104:root@pam:",
                "node": "vmhost1",
                "user": "root@pam",
                "status": "stopped",
                "starttime": "1700000000",
                "exitstatus": "OK"
            }
        ]);
        let arr = pve_response.as_array().unwrap();
        let task = &arr[0];
        assert_eq!(
            extract_task_upid(task),
            "UPID:vmhost1:00001234:0000ABCD:00000000:qmstart:104:root@pam:",
            "list_tasks must use the upid field, not the object id field"
        );
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
