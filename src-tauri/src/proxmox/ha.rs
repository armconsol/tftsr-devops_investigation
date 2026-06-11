// HA (High Availability) groups management module
// Provides operations for managing Proxmox HA groups

use serde::{Deserialize, Serialize};

/// HA group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaGroup {
    pub group: String,
    pub nodes: Vec<String>,
    pub max_failures: u32,
    pub max_relocate: u32,
    pub state: String,
}

/// HA resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaResource {
    pub resource: String,
    pub group: Option<String>,
    pub node: Option<String>,
    pub state: String,
    pub enabled: bool,
}

/// List HA groups
pub async fn list_ha_groups(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<HaGroup>, String> {
    let path = "cluster/ha/groups";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list HA groups: {}", e))?;

    if let Some(groups) = response.get("data").and_then(|d| d.as_array()) {
        let group_list: Vec<HaGroup> = groups
            .iter()
            .filter_map(|group| {
                let name = group.get("group")?.as_str()?.to_string();
                let nodes: Vec<String> = group
                    .get("nodes")
                    .and_then(|n| n.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|n| n.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let max_failures = group.get("max_failures")?.as_u64()? as u32;
                let max_relocate = group.get("max_relocate")?.as_u64()? as u32;
                let state = group
                    .get("state")?
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();

                Some(HaGroup {
                    group: name,
                    nodes,
                    max_failures,
                    max_relocate,
                    state,
                })
            })
            .collect();

        Ok(group_list)
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// Create HA group
pub async fn create_ha_group(
    client: &crate::proxmox::client::ProxmoxClient,
    group: &str,
    nodes: &[String],
    max_failures: u32,
    max_relocate: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = "cluster/ha/groups";
    let config = serde_json::json!({
        "group": group,
        "nodes": nodes,
        "max_failures": max_failures,
        "max_relocate": max_relocate
    });

    let _response: serde_json::Value = client
        .post(path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to create HA group {}: {}", group, e))?;
    Ok(())
}

/// Update HA group
pub async fn update_ha_group(
    client: &crate::proxmox::client::ProxmoxClient,
    group: &str,
    nodes: &[String],
    max_failures: u32,
    max_relocate: u32,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ha/groups/{}", group);
    let config = serde_json::json!({
        "nodes": nodes,
        "max_failures": max_failures,
        "max_relocate": max_relocate
    });

    let _response: serde_json::Value = client
        .put(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update HA group {}: {}", group, e))?;
    Ok(())
}

/// Delete HA group
pub async fn delete_ha_group(
    client: &crate::proxmox::client::ProxmoxClient,
    group: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ha/groups/{}", group);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete HA group {}: {}", group, e))?;
    Ok(())
}

/// List HA resources
pub async fn list_ha_resources(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<HaResource>, String> {
    let path = "cluster/ha/resources";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list HA resources: {}", e))?;

    if let Some(resources) = response.get("data").and_then(|d| d.as_array()) {
        let resource_list: Vec<HaResource> = resources
            .iter()
            .filter_map(|resource| {
                let res = resource.get("resource")?.as_str()?.to_string();
                let group = resource
                    .get("group")
                    .and_then(|g| g.as_str())
                    .map(|s| s.to_string());
                let node = resource
                    .get("node")
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string());
                let state = resource
                    .get("state")?
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();
                let enabled = resource
                    .get("enabled")
                    .and_then(|e| e.as_bool())
                    .unwrap_or(true);

                Some(HaResource {
                    resource: res,
                    group,
                    node,
                    state,
                    enabled,
                })
            })
            .collect();

        Ok(resource_list)
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// Enable HA resource
pub async fn enable_ha_resource(
    client: &crate::proxmox::client::ProxmoxClient,
    resource: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ha/resources/{}/enable", resource);
    let _response: serde_json::Value = client
        .post(&path, &serde_json::json!({}), Some(ticket))
        .await
        .map_err(|e| format!("Failed to enable HA resource {}: {}", resource, e))?;
    Ok(())
}

/// Disable HA resource
pub async fn disable_ha_resource(
    client: &crate::proxmox::client::ProxmoxClient,
    resource: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ha/resources/{}/disable", resource);
    let _response: serde_json::Value = client
        .post(&path, &serde_json::json!({}), Some(ticket))
        .await
        .map_err(|e| format!("Failed to disable HA resource {}: {}", resource, e))?;
    Ok(())
}

/// Manage HA resource
pub async fn manage_ha_resource(
    client: &crate::proxmox::client::ProxmoxClient,
    resource: &str,
    action: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ha/resources/{}/{}", resource, action);
    let _response: serde_json::Value = client
        .post(&path, &serde_json::json!({}), Some(ticket))
        .await
        .map_err(|e| format!("Failed to manage HA resource {}: {}", resource, e))?;
    Ok(())
}

/// Get HA group status
pub async fn get_ha_group_status(
    client: &crate::proxmox::client::ProxmoxClient,
    group: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    let path = format!("cluster/ha/groups/{}/status", group);
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get HA group {}: {}", group, e))
}

/// Get HA resource status
pub async fn get_ha_resource_status(
    client: &crate::proxmox::client::ProxmoxClient,
    resource: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    let path = format!("cluster/ha/resources/{}/status", resource);
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get HA resource {}: {}", resource, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ha_group_serialization() {
        let group = HaGroup {
            group: "primary".to_string(),
            nodes: vec!["pve-node-1".to_string(), "pve-node-2".to_string()],
            max_failures: 2,
            max_relocate: 1,
            state: "enabled".to_string(),
        };

        let json = serde_json::to_string(&group).unwrap();
        let deserialized: HaGroup = serde_json::from_str(&json).unwrap();

        assert_eq!(group.group, deserialized.group);
        assert_eq!(group.state, "enabled");
    }

    #[test]
    fn test_ha_resource_serialization() {
        let resource = HaResource {
            resource: "vm:100".to_string(),
            group: Some("primary".to_string()),
            node: Some("pve-node-1".to_string()),
            state: "started".to_string(),
            enabled: true,
        };

        let json = serde_json::to_string(&resource).unwrap();
        let deserialized: HaResource = serde_json::from_str(&json).unwrap();

        assert_eq!(resource.resource, deserialized.resource);
        assert_eq!(resource.enabled, deserialized.enabled);
    }
}
