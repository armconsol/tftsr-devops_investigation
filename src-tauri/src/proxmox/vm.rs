// VM management module
// Provides operations for managing Proxmox VE virtual machines

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// VM information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmInfo {
    pub id: u32,
    pub name: Option<String>,
    pub status: String,
    pub cpu: f64,
    pub memory: u64,
    pub disk: u64,
    pub uptime: u64,
    pub node: String,
    pub template: Option<bool>,
    pub agent: Option<String>,
    pub mem: Option<u64>,
    pub max_mem: Option<u64>,
    pub max_disk: Option<u64>,
    pub netin: Option<u64>,
    pub netout: Option<u64>,
    pub diskread: Option<u64>,
    pub diskwrite: Option<u64>,
}

/// VM power state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VmState {
    Running,
    Stopped,
    Suspended,
    Paused,
}

/// Start a VM
pub async fn start_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu/{}/status/start", node, vmid);
    let _response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to start VM {}: {}", vmid, e))?;
    Ok(())
}

/// Stop a VM
pub async fn stop_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu/{}/status/stop", node, vmid);
    let _response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to stop VM {}: {}", vmid, e))?;
    Ok(())
}

/// Reboot a VM
pub async fn reboot_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu/{}/status/reboot", node, vmid);
    let _response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to reboot VM {}: {}", vmid, e))?;
    Ok(())
}

/// Shutdown a VM
pub async fn shutdown_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu/{}/status/shutdown", node, vmid);
    let _response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to shutdown VM {}: {}", vmid, e))?;
    Ok(())
}

/// Resume a suspended VM
pub async fn resume_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu/{}/status/resume", node, vmid);
    let _response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to resume VM {}: {}", vmid, e))?;
    Ok(())
}

/// Suspend a VM
pub async fn suspend_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu/{}/status/suspend", node, vmid);
    let _response: serde_json::Value = client
        .post_form(&path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to suspend VM {}: {}", vmid, e))?;
    Ok(())
}

/// List all VMs
pub async fn list_vms(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<VmInfo>, String> {
    // cluster/resources is GET-only; handle_response strips the {"data":[...]} envelope.
    let response: serde_json::Value = client
        .get("cluster/resources?type=vm", Some(ticket))
        .await
        .map_err(|e| format!("Failed to list VMs: {}", e))?;

    let resources = response
        .as_array()
        .ok_or_else(|| "Invalid response format".to_string())?;

    let vms: Vec<VmInfo> = resources
        .iter()
        .filter_map(|r| {
            let vmid = r.get("vmid")?.as_u64()?;
            let node = r.get("node")?.as_str()?.to_string();
            // Only include qemu VMs (not LXC containers which also appear in cluster/resources?type=vm)
            let resource_type = r.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if resource_type != "qemu" {
                return None;
            }
            let name = r
                .get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());
            let status = r
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown")
                .to_string();
            // cpu may be absent for stopped VMs
            let cpu = r.get("cpu").and_then(|c| c.as_f64()).unwrap_or(0.0);

            Some(VmInfo {
                id: vmid as u32,
                name,
                status,
                cpu,
                memory: r.get("mem").and_then(|m| m.as_u64()).unwrap_or(0),
                disk: r.get("disk").and_then(|d| d.as_u64()).unwrap_or(0),
                uptime: r.get("uptime").and_then(|u| u.as_u64()).unwrap_or(0),
                node,
                template: r.get("template").and_then(|t| t.as_bool()),
                agent: r
                    .get("agent")
                    .and_then(|a| a.as_str())
                    .map(|s| s.to_string()),
                mem: r.get("mem").and_then(|m| m.as_u64()),
                max_mem: r.get("maxmem").and_then(|m| m.as_u64()),
                max_disk: r.get("maxdisk").and_then(|d| d.as_u64()),
                netin: r.get("netin").and_then(|n| n.as_u64()),
                netout: r.get("netout").and_then(|n| n.as_u64()),
                diskread: r.get("diskread").and_then(|d| d.as_u64()),
                diskwrite: r.get("diskwrite").and_then(|d| d.as_u64()),
            })
        })
        .collect();

    Ok(vms)
}

/// Get VM details
pub async fn get_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<VmInfo, String> {
    let path = format!("nodes/{}/qemu/{}/config", node, vmid);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get VM {}: {}", vmid, e))?;

    let vm = &response;

    Ok(VmInfo {
        id: vmid,
        name: vm
            .get("name")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string()),
        status: vm
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string(),
        cpu: vm.get("cpu").and_then(|c| c.as_f64()).unwrap_or(0.0),
        memory: vm.get("memory").and_then(|m| m.as_u64()).unwrap_or(0),
        disk: vm.get("disk").and_then(|d| d.as_u64()).unwrap_or(0),
        uptime: vm.get("uptime").and_then(|u| u.as_u64()).unwrap_or(0),
        node: node.to_string(),
        template: vm.get("template").and_then(|t| t.as_bool()),
        agent: vm
            .get("agent")
            .and_then(|a| a.as_str())
            .map(|s| s.to_string()),
        mem: vm.get("mem").and_then(|m| m.as_u64()),
        max_mem: vm.get("maxmem").and_then(|m| m.as_u64()),
        max_disk: vm.get("maxdisk").and_then(|d| d.as_u64()),
        netin: vm.get("netin").and_then(|n| n.as_u64()),
        netout: vm.get("netout").and_then(|n| n.as_u64()),
        diskread: vm.get("diskread").and_then(|d| d.as_u64()),
        diskwrite: vm.get("diskwrite").and_then(|d| d.as_u64()),
    })
}

/// Get VM status
pub async fn get_vm_status(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    let path = format!("nodes/{}/qemu/{}/status/current", node, vmid);
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get VM status {}: {}", vmid, e))
}

/// Get VM current configuration
pub async fn get_vm_config(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    let path = format!("nodes/{}/qemu/{}/config", node, vmid);
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get VM config {}: {}", vmid, e))
}

/// Create a new VM
pub async fn create_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    config: &serde_json::Value,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu", node);

    // Convert JSON config to form-encoded params
    let mut params: Vec<(&str, &str)> = Vec::new();
    let mut string_values: Vec<String> = Vec::new();

    if let Some(obj) = config.as_object() {
        // First pass: collect all non-string values
        for (_key, value) in obj {
            if value.as_str().is_none() {
                string_values.push(value.to_string());
            }
        }

        // Second pass: build params
        let mut string_idx = 0;
        for (key, value) in obj {
            if let Some(str_val) = value.as_str() {
                params.push((key.as_str(), str_val));
            } else {
                params.push((key.as_str(), string_values[string_idx].as_str()));
                string_idx += 1;
            }
        }
    }

    let _response: serde_json::Value = client
        .post_form(&path, &params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create VM {}: {}", vmid, e))?;
    Ok(())
}

/// Delete a VM
pub async fn delete_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu/{}", node, vmid);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete VM {}: {}", vmid, e))?;
    Ok(())
}

/// Clone a VM
pub async fn clone_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    new_vmid: u32,
    name: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu/{}/clone", node, vmid);
    let newid_str = new_vmid.to_string();
    let params = vec![("newid", newid_str.as_str()), ("name", name), ("full", "1")];

    let _response: serde_json::Value = client
        .post_form(&path, &params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to clone VM {} to {}: {}", vmid, new_vmid, e))?;
    Ok(())
}

/// Migrate a VM
pub async fn migrate_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    source_node: &str,
    vmid: u32,
    target_node: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu/{}/migrate", source_node, vmid);
    let params = vec![("target", target_node), ("online", "1")];

    let _response: serde_json::Value = client
        .post_form(&path, &params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to migrate VM {} to {}: {}", vmid, target_node, e))?;
    Ok(())
}

/// Create a snapshot
pub async fn create_snapshot(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    snapshot_name: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu/{}/snapshot", node, vmid);
    let params = vec![("snapname", snapshot_name)];

    let _response: serde_json::Value = client
        .post_form(&path, &params, Some(ticket))
        .await
        .map_err(|e| {
            format!(
                "Failed to create snapshot {} for VM {}: {}",
                snapshot_name, vmid, e
            )
        })?;
    Ok(())
}

/// Delete a snapshot
pub async fn delete_snapshot(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    snapshot_name: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/qemu/{}/snapshot/{}", node, vmid, snapshot_name);
    let _response: serde_json::Value = client.delete(&path, Some(ticket)).await.map_err(|e| {
        format!(
            "Failed to delete snapshot {} for VM {}: {}",
            snapshot_name, vmid, e
        )
    })?;
    Ok(())
}

/// Rollback to a snapshot
pub async fn rollback_snapshot(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    snapshot_name: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!(
        "nodes/{}/qemu/{}/snapshot/{}/rollback",
        node, vmid, snapshot_name
    );
    let _response: serde_json::Value =
        client
            .post_form(&path, &[], Some(ticket))
            .await
            .map_err(|e| {
                format!(
                    "Failed to rollback VM {} to snapshot {}: {}",
                    vmid, snapshot_name, e
                )
            })?;
    Ok(())
}

/// List snapshots
pub async fn list_snapshots(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let path = format!("nodes/{}/qemu/{}/snapshot", node, vmid);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list snapshots for VM {}: {}", vmid, e))?;

    response
        .as_array()
        .map(|arr| arr.to_vec())
        .ok_or_else(|| "Invalid response format".to_string())
}

use crate::proxmox::validate::{validate_node, validate_vmid};

/// Pending config entry returned by /nodes/{node}/qemu/{vmid}/pending
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmPendingEntry {
    pub key: String,
    pub value: Option<serde_json::Value>,
    pub pending: Option<serde_json::Value>,
    pub delete: Option<i64>,
}

/// Get the current VM configuration (raw polymorphic object)
/// GET /nodes/{node}/qemu/{vmid}/config
pub async fn get_vm_config_raw(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    validate_node(node)?;
    validate_vmid(vmid)?;
    let path = format!("nodes/{}/qemu/{}/config", node, vmid);
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get VM config for vmid {}: {}", vmid, e))
}

/// Get pending config changes for a VM
/// GET /nodes/{node}/qemu/{vmid}/pending
pub async fn get_vm_pending_config(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<Vec<VmPendingEntry>, String> {
    validate_node(node)?;
    validate_vmid(vmid)?;
    let path = format!("nodes/{}/qemu/{}/pending", node, vmid);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get VM pending config for vmid {}: {}", vmid, e))?;

    let entries: Vec<VmPendingEntry> = serde_json::from_value(response)
        .map_err(|e| format!("Failed to deserialize pending config entries: {}", e))?;
    Ok(entries)
}

/// Migrate a VM to a remote cluster
/// POST /nodes/{node}/qemu/{vmid}/remote_migrate
/// Returns the UPID task string.
pub async fn remote_migrate_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    target_node: &str,
    target_storage: &str,
    online: bool,
    ticket: &str,
) -> Result<String, String> {
    validate_node(node)?;
    validate_vmid(vmid)?;

    let online_str = if online { "1" } else { "0" };
    let path = crate::proxmox::migration::remote_migrate_path(node, vmid);
    let params: &[(&str, &str)] = &[
        ("target", target_node),
        ("targetstorage", target_storage),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_migrate_uses_rest_underscore_path() {
        // vm.rs delegates to the shared helper; assert it yields the REST
        // (underscore) form, not the dashed CLI name that 501s.
        assert_eq!(
            crate::proxmox::migration::remote_migrate_path("vmhost3", 104),
            "nodes/vmhost3/qemu/104/remote_migrate"
        );
    }

    #[test]
    fn test_vm_info_serialization() {
        let vm = VmInfo {
            id: 100,
            name: Some("web-server".to_string()),
            status: "running".to_string(),
            cpu: 2.5,
            memory: 4096,
            disk: 50000,
            uptime: 86400,
            node: "pve-node-1".to_string(),
            template: Some(false),
            agent: Some("1".to_string()),
            mem: Some(4096),
            max_mem: Some(8192),
            max_disk: Some(100000),
            netin: Some(1000000),
            netout: Some(2000000),
            diskread: Some(5000000),
            diskwrite: Some(3000000),
        };

        let json = serde_json::to_string(&vm).unwrap();
        let deserialized: VmInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(vm.id, deserialized.id);
        assert_eq!(vm.name, deserialized.name);
        assert_eq!(vm.status, "running");
    }

    #[test]
    fn test_vm_state_serialization() {
        let json = serde_json::to_string(&VmState::Running).unwrap();
        assert_eq!(json, "\"running\"");

        let running: VmState = serde_json::from_str("\"running\"").unwrap();
        assert_eq!(running, VmState::Running);
    }

    #[test]
    fn test_vm_pending_entry_deserialization() {
        let fixture = r#"[
            {"key": "startup", "value": "order=2"},
            {"key": "ostype", "value": "l26"},
            {"key": "memory", "value": 2048, "pending": 4096},
            {"key": "net0", "delete": 1}
        ]"#;
        let entries: Vec<VmPendingEntry> = serde_json::from_str(fixture).unwrap();
        assert_eq!(entries.len(), 4);

        assert_eq!(entries[0].key, "startup");
        assert_eq!(
            entries[0].value.as_ref().unwrap().as_str().unwrap(),
            "order=2"
        );
        assert!(entries[0].pending.is_none());
        assert!(entries[0].delete.is_none());

        assert_eq!(entries[2].key, "memory");
        assert_eq!(entries[2].value.as_ref().unwrap().as_u64().unwrap(), 2048);
        assert_eq!(entries[2].pending.as_ref().unwrap().as_u64().unwrap(), 4096);

        assert_eq!(entries[3].key, "net0");
        assert_eq!(entries[3].delete.unwrap(), 1);
        assert!(entries[3].value.is_none());
    }

    #[test]
    fn test_validate_node_valid() {
        assert!(validate_node("pve-node-1").is_ok());
        assert!(validate_node("node01").is_ok());
        assert!(validate_node("a").is_ok());
    }

    #[test]
    fn test_validate_node_invalid() {
        assert!(validate_node("").is_err());
        assert!(validate_node("node/evil").is_err());
        assert!(validate_node("node..evil").is_err());
        assert!(validate_node(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_validate_vmid_valid() {
        assert!(validate_vmid(100).is_ok());
        assert!(validate_vmid(200).is_ok());
        assert!(validate_vmid(999_999_999).is_ok());
    }

    #[test]
    fn test_validate_vmid_invalid() {
        assert!(validate_vmid(0).is_err());
        assert!(validate_vmid(99).is_err());
        assert!(validate_vmid(1_000_000_000).is_err());
    }
}
