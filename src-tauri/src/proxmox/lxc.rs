// LXC container management module
// Provides operations for managing Proxmox VE Linux containers

use crate::proxmox::validate::{validate_node, validate_vmid};
use serde::{Deserialize, Serialize};

/// Parameters for creating a new LXC container
#[derive(Clone, Serialize, Deserialize)]
pub struct ContainerCreateParams {
    pub vmid: u32,
    pub ostemplate: String,
    pub hostname: Option<String>,
    pub memory: Option<u32>,
    pub cores: Option<u32>,
    pub rootfs: Option<String>,
    pub net0: Option<String>,
    pub password: Option<String>,
    pub unprivileged: Option<bool>,
    pub start: Option<bool>,
}

impl std::fmt::Debug for ContainerCreateParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContainerCreateParams")
            .field("vmid", &self.vmid)
            .field("ostemplate", &self.ostemplate)
            .field("hostname", &self.hostname)
            .field("memory", &self.memory)
            .field("cores", &self.cores)
            .field("rootfs", &self.rootfs)
            .field("net0", &self.net0)
            .field("password", &self.password.as_ref().map(|_| "[REDACTED]"))
            .field("unprivileged", &self.unprivileged)
            .field("start", &self.start)
            .finish()
    }
}

/// Get the current container configuration (raw polymorphic object)
/// GET /nodes/{node}/lxc/{vmid}/config
pub async fn get_container_config(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    validate_node(node)?;
    validate_vmid(vmid)?;
    let path = format!("nodes/{node}/lxc/{vmid}/config");
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get container config for vmid {vmid}: {e}"))
}

/// Create a new LXC container
/// POST /nodes/{node}/lxc
/// Returns the UPID task string.
pub async fn create_proxmox_container(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    params: ContainerCreateParams,
    ticket: &str,
) -> Result<String, String> {
    validate_node(node)?;
    validate_vmid(params.vmid)?;

    let vmid_str = params.vmid.to_string();
    let memory_str = params.memory.map(|m| m.to_string());
    let cores_str = params.cores.map(|c| c.to_string());

    let mut form: Vec<(&str, &str)> = vec![
        ("vmid", vmid_str.as_str()),
        ("ostemplate", params.ostemplate.as_str()),
    ];

    if let Some(ref hostname) = params.hostname {
        form.push(("hostname", hostname.as_str()));
    }
    if let Some(ref s) = memory_str {
        form.push(("memory", s.as_str()));
    }
    if let Some(ref s) = cores_str {
        form.push(("cores", s.as_str()));
    }
    if let Some(ref rootfs) = params.rootfs {
        form.push(("rootfs", rootfs.as_str()));
    }
    if let Some(ref net0) = params.net0 {
        form.push(("net0", net0.as_str()));
    }
    if let Some(ref password) = params.password {
        form.push(("password", password.as_str()));
    }
    if let Some(unprivileged) = params.unprivileged {
        form.push(("unprivileged", if unprivileged { "1" } else { "0" }));
    }
    if let Some(start) = params.start {
        form.push(("start", if start { "1" } else { "0" }));
    }

    let path = format!("nodes/{node}/lxc");
    let response: serde_json::Value = client
        .post_form(&path, &form, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create container on node {node}: {e}"))?;

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
    fn test_container_create_params_construction() {
        let params = ContainerCreateParams {
            vmid: 110,
            ostemplate: "local:vztmpl/ubuntu-22.04-standard.tar.zst".to_string(),
            hostname: Some("web-ct".to_string()),
            memory: Some(512),
            cores: Some(2),
            rootfs: Some("local-lvm:8".to_string()),
            net0: Some("name=eth0,bridge=vmbr0,ip=dhcp".to_string()),
            password: Some("secret".to_string()),
            unprivileged: Some(true),
            start: Some(false),
        };

        assert_eq!(params.vmid, 110);
        assert_eq!(
            params.ostemplate,
            "local:vztmpl/ubuntu-22.04-standard.tar.zst"
        );
        assert_eq!(params.hostname.as_deref(), Some("web-ct"));
        assert_eq!(params.memory, Some(512));
        assert_eq!(params.cores, Some(2));
        assert_eq!(params.unprivileged, Some(true));
        assert_eq!(params.start, Some(false));
    }

    #[test]
    fn test_container_create_params_minimal() {
        let params = ContainerCreateParams {
            vmid: 200,
            ostemplate: "local:vztmpl/debian-12.tar.zst".to_string(),
            hostname: None,
            memory: None,
            cores: None,
            rootfs: None,
            net0: None,
            password: None,
            unprivileged: None,
            start: None,
        };

        assert_eq!(params.vmid, 200);
        assert!(params.hostname.is_none());
        assert!(params.memory.is_none());
        assert!(params.unprivileged.is_none());
    }

    #[test]
    fn test_container_config_path_building() {
        let node = "pve-node1";
        let vmid = 110_u32;
        let path = format!("nodes/{node}/lxc/{vmid}/config");
        assert_eq!(path, "nodes/pve-node1/lxc/110/config");
    }

    #[test]
    fn test_vmid_range_valid() {
        assert!(validate_vmid(100).is_ok());
        assert!(validate_vmid(110).is_ok());
        assert!(validate_vmid(999_999_999).is_ok());
    }

    #[test]
    fn test_vmid_range_invalid() {
        assert!(validate_vmid(0).is_err());
        assert!(validate_vmid(99).is_err());
        assert!(validate_vmid(1_000_000_000).is_err());
    }

    #[test]
    fn test_node_name_valid() {
        assert!(validate_node("pve-node1").is_ok());
        assert!(validate_node("proxmox").is_ok());
        assert!(validate_node("node-01").is_ok());
    }

    #[test]
    fn test_node_name_invalid() {
        assert!(validate_node("").is_err());
        assert!(validate_node("node/evil").is_err());
        assert!(validate_node("node.evil").is_err());
        assert!(validate_node("node evil").is_err());
        assert!(validate_node(&"n".repeat(65)).is_err());
    }

    #[test]
    fn test_container_create_params_serialization() {
        let params = ContainerCreateParams {
            vmid: 150,
            ostemplate: "local:vztmpl/alpine-3.18.tar.zst".to_string(),
            hostname: Some("alpine-ct".to_string()),
            memory: Some(256),
            cores: Some(1),
            rootfs: Some("local-lvm:4".to_string()),
            net0: None,
            password: None,
            unprivileged: Some(false),
            start: Some(true),
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: ContainerCreateParams = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.vmid, 150);
        assert_eq!(deserialized.hostname.as_deref(), Some("alpine-ct"));
        assert_eq!(deserialized.unprivileged, Some(false));
        assert_eq!(deserialized.start, Some(true));
    }
}
