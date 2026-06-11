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
        .post(&path, &serde_json::json!({}), Some(ticket))
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
        .post(&path, &serde_json::json!({}), Some(ticket))
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
        .post(&path, &serde_json::json!({}), Some(ticket))
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
        .post(&path, &serde_json::json!({}), Some(ticket))
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
        .post(&path, &serde_json::json!({}), Some(ticket))
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
        .post(&path, &serde_json::json!({}), Some(ticket))
        .await
        .map_err(|e| format!("Failed to suspend VM {}: {}", vmid, e))?;
    Ok(())
}

/// List all VMs
pub async fn list_vms(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<VmInfo>, String> {
    let path = "cluster/resources";
    let params = serde_json::json!({
        "type": "qemu"
    });

    let response: serde_json::Value = client
        .post(path, &params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list VMs: {}", e))?;

    // Parse the response to extract VM info
    // The API returns a list of resources in the "data" field
    if let Some(resources) = response.get("data").and_then(|d| d.as_array()) {
        let vms: Vec<VmInfo> = resources
            .iter()
            .filter_map(|r| {
                let vmid = r.get("vmid")?.as_u64()?;
                let node = r.get("node")?.as_str()?.to_string();
                let name = r.get("name")?.as_str().map(|s| s.to_string());
                let status = r.get("status")?.as_str()?.to_string();
                let cpu = r.get("cpu")?.as_f64()?;

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
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
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

    // Parse the response to extract VM info
    let vm = response.get("data").ok_or("Invalid response format")?;

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
    let _response: serde_json::Value = client
        .post(&path, config, Some(ticket))
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
    let config = serde_json::json!({
        "newid": new_vmid,
        "name": name,
        "full": 1
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
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
    let config = serde_json::json!({
        "target": target_node,
        "online": true
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
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
    let config = serde_json::json!({
        "snapname": snapshot_name
    });

    let _response: serde_json::Value =
        client
            .post(&path, &config, Some(ticket))
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
    let _response: serde_json::Value = client
        .post(&path, &serde_json::json!({}), Some(ticket))
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

    if let Some(snapshots) = response.get("data").and_then(|d| d.as_array()) {
        Ok(snapshots.to_vec())
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
