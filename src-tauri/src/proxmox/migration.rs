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
    let body = serde_json::json!({
        "target": target_node,
        "online": 1,
        "force": 0,
    });

    let response: serde_json::Value = client
        .post::<serde_json::Value, _>(&path, &body, Some(ticket))
        .await
        .map_err(|e| format!("Failed to migrate VM {}: {}", vm_id, e))?;

    // handle_response unwraps the "data" envelope; migrate returns the task UPID as a string.
    let task_id = response.as_str().unwrap_or("").to_string();
    let start_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    Ok(MigrationTask {
        task_id,
        vm_id,
        source_node: node.to_string(),
        target_node: target_node.to_string(),
        source_cluster: client.base_url().to_string(),
        target_cluster: target_cluster.to_string(),
        status: "running".to_string(),
        progress: 0,
        start_time,
        end_time: None,
        error: None,
    })
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

/// Outcome of interpreting a migration task's status/exitstatus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationOutcome {
    /// Task is still running.
    Running,
    /// Task finished successfully.
    Success,
    /// Task finished with an error (carries the Proxmox error string).
    Error(String),
}

/// Interpret a task's `status` + `exitstatus` fields into a migration outcome.
///
/// Proxmox tasks report `status` = "running" | "stopped". When stopped, the
/// `exitstatus` is "OK" on success, otherwise it carries the failure reason
/// (e.g. "migration aborted"). A missing/empty exitstatus on a stopped task is
/// treated as an unknown error rather than a false success.
pub fn interpret_task_exitstatus(status: &str, exitstatus: Option<&str>) -> MigrationOutcome {
    if status != "stopped" {
        return MigrationOutcome::Running;
    }
    match exitstatus.map(|s| s.trim()).filter(|s| !s.is_empty()) {
        Some("OK") => MigrationOutcome::Success,
        Some(err) => MigrationOutcome::Error(err.to_string()),
        None => MigrationOutcome::Error("Task stopped without an exit status".to_string()),
    }
}

/// Normalise a TLS fingerprint to Proxmox's canonical form: upper-case hex
/// pairs separated by colons, no surrounding whitespace.
pub fn normalize_fingerprint(raw: &str) -> String {
    raw.trim().to_uppercase()
}

/// Build the `target-endpoint` property string for a PVE `remote-migrate`
/// request using an API token for authentication.
///
/// Format:
/// `apitoken=PVEAPIToken=<full_tokenid>=<secret>,host=<host>,port=<port>[,fingerprint=<fp>]`
pub fn build_remote_target_endpoint(
    full_tokenid: &str,
    secret: &str,
    host: &str,
    port: u16,
    fingerprint: Option<&str>,
) -> String {
    let mut endpoint = format!(
        "apitoken=PVEAPIToken={}={},host={},port={}",
        full_tokenid, secret, host, port
    );
    if let Some(fp) = fingerprint.map(|f| f.trim()).filter(|f| !f.is_empty()) {
        endpoint.push_str(&format!(",fingerprint={}", normalize_fingerprint(fp)));
    }
    endpoint
}

/// Extract the active node TLS fingerprint from the response of
/// `GET /nodes/{node}/certificates/info` (an array of certificate objects).
///
/// Prefers the `pveproxy-ssl.pem` certificate; falls back to the first cert
/// that carries a non-empty `fingerprint`.
pub fn extract_node_fingerprint(certs: &serde_json::Value) -> Option<String> {
    let arr = certs.as_array()?;
    // Prefer the pveproxy cert (the one serving the API/console).
    let preferred = arr.iter().find(|c| {
        c.get("filename")
            .and_then(|f| f.as_str())
            .map(|f| f.contains("pveproxy"))
            .unwrap_or(false)
    });
    let chosen = preferred.or_else(|| {
        arr.iter().find(|c| {
            c.get("fingerprint")
                .and_then(|f| f.as_str())
                .map(|f| !f.trim().is_empty())
                .unwrap_or(false)
        })
    })?;
    chosen
        .get("fingerprint")
        .and_then(|f| f.as_str())
        .map(|f| f.trim())
        .filter(|f| !f.is_empty())
        .map(normalize_fingerprint)
}

/// Fetch a node's TLS fingerprint from its certificates API.
/// GET /nodes/{node}/certificates/info
pub async fn get_node_fingerprint(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<String, String> {
    let path = format!("nodes/{}/certificates/info", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to fetch certificate info for node {}: {}", node, e))?;
    extract_node_fingerprint(&response)
        .ok_or_else(|| format!("No TLS fingerprint found for node {}", node))
}

/// Perform a cross-cluster (remote) VM migration.
/// POST /nodes/{node}/qemu/{vmid}/remote-migrate
///
/// `target_endpoint` is the property string built by
/// [`build_remote_target_endpoint`]. Returns the UPID task string.
#[allow(clippy::too_many_arguments)]
pub async fn remote_migrate_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    target_endpoint: &str,
    target_bridge: &str,
    target_storage: &str,
    online: bool,
    ticket: &str,
) -> Result<String, String> {
    let online_str = if online { "1" } else { "0" };
    let vmid_str = vmid.to_string();
    let path = format!("nodes/{}/qemu/{}/remote-migrate", node, vmid);
    let params: &[(&str, &str)] = &[
        ("target-endpoint", target_endpoint),
        ("target-vmid", &vmid_str),
        ("target-bridge", target_bridge),
        ("target-storage", target_storage),
        ("online", online_str),
    ];

    let response: serde_json::Value = client
        .post_form(&path, params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to remote-migrate VM {}: {}", vmid, e))?;

    let upid = response
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| response.to_string());
    Ok(upid)
}

/// Cancel migration task
pub async fn cancel_migration(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vm_id: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu/{}/migrate", node, vm_id);
    let params = vec![("cancel", "1")];

    let _response: serde_json::Value = client
        .post_form(&path, &params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to cancel migration for VM {}: {}", vm_id, e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_interpret_running() {
        assert_eq!(
            interpret_task_exitstatus("running", None),
            MigrationOutcome::Running
        );
        assert_eq!(
            interpret_task_exitstatus("running", Some("")),
            MigrationOutcome::Running
        );
    }

    #[test]
    fn test_interpret_success() {
        assert_eq!(
            interpret_task_exitstatus("stopped", Some("OK")),
            MigrationOutcome::Success
        );
    }

    #[test]
    fn test_interpret_error() {
        assert_eq!(
            interpret_task_exitstatus("stopped", Some("migration aborted")),
            MigrationOutcome::Error("migration aborted".to_string())
        );
    }

    #[test]
    fn test_interpret_stopped_without_status_is_error() {
        assert_eq!(
            interpret_task_exitstatus("stopped", None),
            MigrationOutcome::Error("Task stopped without an exit status".to_string())
        );
        assert_eq!(
            interpret_task_exitstatus("stopped", Some("  ")),
            MigrationOutcome::Error("Task stopped without an exit status".to_string())
        );
    }

    #[test]
    fn test_normalize_fingerprint() {
        assert_eq!(
            normalize_fingerprint("  ab:cd:ef  "),
            "AB:CD:EF".to_string()
        );
    }

    #[test]
    fn test_build_target_endpoint_with_fingerprint() {
        let ep = build_remote_target_endpoint(
            "root@pam!tftsr-migrate",
            "secretvalue",
            "172.0.0.21",
            8006,
            Some("ab:cd:ef"),
        );
        assert_eq!(
            ep,
            "apitoken=PVEAPIToken=root@pam!tftsr-migrate=secretvalue,host=172.0.0.21,port=8006,fingerprint=AB:CD:EF"
        );
    }

    #[test]
    fn test_build_target_endpoint_without_fingerprint() {
        let ep = build_remote_target_endpoint(
            "root@pam!tftsr-migrate",
            "secretvalue",
            "172.0.0.21",
            8006,
            None,
        );
        assert_eq!(
            ep,
            "apitoken=PVEAPIToken=root@pam!tftsr-migrate=secretvalue,host=172.0.0.21,port=8006"
        );
        // Blank fingerprint should be treated as None.
        let ep2 = build_remote_target_endpoint(
            "root@pam!tftsr-migrate",
            "secretvalue",
            "172.0.0.21",
            8006,
            Some("   "),
        );
        assert_eq!(ep, ep2);
    }

    #[test]
    fn test_extract_node_fingerprint_prefers_pveproxy() {
        let certs = json!([
            {"filename": "pve-ssl.pem", "fingerprint": "11:22"},
            {"filename": "pveproxy-ssl.pem", "fingerprint": "aa:bb:cc"}
        ]);
        assert_eq!(
            extract_node_fingerprint(&certs),
            Some("AA:BB:CC".to_string())
        );
    }

    #[test]
    fn test_extract_node_fingerprint_fallback_first_nonempty() {
        let certs = json!([
            {"filename": "other.pem", "fingerprint": ""},
            {"filename": "x.pem", "fingerprint": "de:ad:be:ef"}
        ]);
        assert_eq!(
            extract_node_fingerprint(&certs),
            Some("DE:AD:BE:EF".to_string())
        );
    }

    #[test]
    fn test_extract_node_fingerprint_none() {
        let certs = json!([{"filename": "x.pem"}]);
        assert_eq!(extract_node_fingerprint(&certs), None);
        assert_eq!(extract_node_fingerprint(&json!({})), None);
    }
}
