// Network interface management for Proxmox
// Provides CRUD operations for network interfaces on Proxmox nodes

use crate::proxmox::client::ProxmoxClient;
use serde::{Deserialize, Serialize};

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    pub iface: String,
    pub r#type: String,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub netmask: Option<String>,
    #[serde(default)]
    pub gateway: Option<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub comments: Option<String>,
}

/// Network interface configuration for creation/update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterfaceConfig {
    pub iface: String,
    #[serde(rename = "type")]
    pub iface_type: String,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub netmask: Option<String>,
    #[serde(default)]
    pub gateway: Option<String>,
    #[serde(default, with = "serde_bool_as_int")]
    pub active: bool,
    #[serde(default, with = "serde_bool_as_int")]
    pub autostart: bool,
    #[serde(default)]
    pub comments: Option<String>,
}

/// Helper module for serde bool-as-int conversion (Proxmox API expects 0/1)
mod serde_bool_as_int {
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i8(if *value { 1 } else { 0 })
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BoolOrInt;

        impl<'de> serde::de::Visitor<'de> for BoolOrInt {
            type Value = bool;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("integer or boolean")
            }

            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<bool, E> {
                Ok(v)
            }

            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<bool, E> {
                Ok(v != 0)
            }

            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<bool, E> {
                Ok(v != 0)
            }
        }

        deserializer.deserialize_any(BoolOrInt)
    }
}

/// List network interfaces on a node
pub async fn list_network_interfaces(
    client: &ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<NetworkInterface>, String> {
    let path = format!("nodes/{}/network", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list network interfaces for node {}: {}", node, e))?;

    let interfaces: Vec<NetworkInterface> = response
        .as_array()
        .ok_or_else(|| "Invalid response format".to_string())?
        .iter()
        .filter_map(|iface| {
            serde_json::from_value(iface.clone())
                .map_err(|e| {
                    tracing::warn!("Failed to deserialize interface: {}", e);
                    e
                })
                .ok()
        })
        .collect();

    Ok(interfaces)
}

/// Create a network interface
pub async fn create_network_interface(
    client: &ProxmoxClient,
    node: &str,
    config: &NetworkInterfaceConfig,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/network", node);

    let mut body = serde_json::json!({
        "iface": config.iface,
        "type": config.iface_type,
    });

    if let Some(ref address) = config.address {
        body["address"] = serde_json::Value::String(address.clone());
    }
    if let Some(ref netmask) = config.netmask {
        body["netmask"] = serde_json::Value::String(netmask.clone());
    }
    if let Some(ref gateway) = config.gateway {
        body["gateway"] = serde_json::Value::String(gateway.clone());
    }
    if config.active {
        body["active"] = serde_json::Value::Number(1.into());
    }
    if config.autostart {
        body["autostart"] = serde_json::Value::Number(1.into());
    }
    if let Some(ref comments) = config.comments {
        body["comments"] = serde_json::Value::String(comments.clone());
    }

    let _response: serde_json::Value = client
        .post(&path, &body, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create network interface {}: {}", config.iface, e))?;

    Ok(())
}

/// Update a network interface
pub async fn update_network_interface(
    client: &ProxmoxClient,
    node: &str,
    iface: &str,
    config: &NetworkInterfaceConfig,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/network/{}", node, iface);

    let mut body = serde_json::json!({
        "iface": config.iface,
        "type": config.iface_type,
    });

    if let Some(ref address) = config.address {
        body["address"] = serde_json::Value::String(address.clone());
    }
    if let Some(ref netmask) = config.netmask {
        body["netmask"] = serde_json::Value::String(netmask.clone());
    }
    if let Some(ref gateway) = config.gateway {
        body["gateway"] = serde_json::Value::String(gateway.clone());
    }
    if config.active {
        body["active"] = serde_json::Value::Number(1.into());
    }
    if config.autostart {
        body["autostart"] = serde_json::Value::Number(1.into());
    }
    if let Some(ref comments) = config.comments {
        body["comments"] = serde_json::Value::String(comments.clone());
    }

    let _response: serde_json::Value = client
        .put(&path, &body, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update network interface {}: {}", iface, e))?;

    Ok(())
}

/// Apply pending network configuration changes on a node.
///
/// This is equivalent to running `ifreload -a` on the node. Returns a task UPID string
/// that can be polled via the tasks API for completion status.
pub async fn reload_network_config(
    client: &ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<String, String> {
    crate::proxmox::validate::validate_node(node)?;

    let path = format!("nodes/{}/network", node);
    client
        .put(&path, &serde_json::json!({}), Some(ticket))
        .await
        .map_err(|e| format!("Failed to reload network config on node {}: {}", node, e))
}

/// Delete a network interface
pub async fn delete_network_interface(
    client: &ProxmoxClient,
    node: &str,
    iface: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/network/{}", node, iface);

    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete network interface {}: {}", iface, e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxmox::validate::validate_node;

    #[test]
    fn test_network_interface_serialization() {
        let iface = NetworkInterface {
            iface: "eth0".to_string(),
            r#type: "eth".to_string(),
            address: Some("192.168.1.100".to_string()),
            netmask: Some("24".to_string()),
            gateway: Some("192.168.1.1".to_string()),
            active: true,
            autostart: true,
            comments: Some("Management interface".to_string()),
        };

        let json = serde_json::to_string_pretty(&iface).unwrap();
        assert!(json.contains("eth0"));
        assert!(json.contains("eth"));
    }

    #[test]
    fn test_network_interface_config_serialization() {
        let config = NetworkInterfaceConfig {
            iface: "eth0".to_string(),
            iface_type: "eth".to_string(),
            address: Some("192.168.1.100".to_string()),
            netmask: Some("24".to_string()),
            gateway: Some("192.168.1.1".to_string()),
            active: true,
            autostart: false,
            comments: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("eth0"));
        assert!(json.contains("\"active\":1"));
        assert!(json.contains("\"autostart\":0"));
    }

    #[test]
    fn test_reload_network_config_node_validation_empty() {
        assert!(validate_node("").is_err());
    }

    #[test]
    fn test_reload_network_config_node_validation_path_traversal() {
        assert!(validate_node("../evil").is_err());
        assert!(validate_node("node/evil").is_err());
    }

    #[test]
    fn test_reload_network_config_node_validation_too_long() {
        let node = "a".repeat(65);
        assert!(validate_node(&node).is_err());
    }

    #[test]
    fn test_reload_network_config_node_validation_valid_names() {
        for name in &["pve-node1", "pve-node-1", "node01", "my-cluster-node"] {
            assert!(
                validate_node(name).is_ok(),
                "expected '{}' to pass validation",
                name
            );
        }
    }
}
