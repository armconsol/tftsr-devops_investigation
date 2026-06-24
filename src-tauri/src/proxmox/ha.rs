// HA (High Availability) groups management module
// Provides operations for managing Proxmox HA groups

use serde::{Deserialize, Serialize};

/// HA group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaGroup {
    #[serde(rename = "id")]
    pub group: String,
    pub nodes: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restricted: Option<bool>,
    #[serde(rename = "nofailback", skip_serializing_if = "Option::is_none")]
    pub no_failback: Option<bool>,
}

/// HA resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaResource {
    pub sid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    pub state: String,
    #[serde(rename = "request_state", skip_serializing_if = "Option::is_none")]
    pub request_state: Option<String>,
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

    if let Some(groups) = response.as_array() {
        let group_list: Vec<HaGroup> = groups
            .iter()
            .filter_map(|group| {
                let name = group.get("group")?.as_str()?.to_string();
                let nodes = group
                    .get("nodes")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let comment = group
                    .get("comment")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string());
                let restricted = group
                    .get("restricted")
                    .and_then(|r| r.as_i64())
                    .map(|v| v != 0);
                let no_failback = group
                    .get("nofailback")
                    .and_then(|f| f.as_i64())
                    .map(|v| v != 0);

                Some(HaGroup {
                    group: name,
                    nodes,
                    comment,
                    restricted,
                    no_failback,
                })
            })
            .collect();

        Ok(group_list)
    } else {
        Err("Invalid response format: expected array".to_string())
    }
}

/// Create HA group
pub async fn create_ha_group(
    client: &crate::proxmox::client::ProxmoxClient,
    group: &str,
    nodes: &[String],
    ticket: &str,
) -> Result<(), String> {
    let path = "cluster/ha/groups";
    let config = serde_json::json!({
        "group": group,
        "nodes": nodes.join(",")
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
    ticket: &str,
) -> Result<(), String> {
    let path = format!("cluster/ha/groups/{}", group);
    let config = serde_json::json!({
        "nodes": nodes.join(",")
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

    if let Some(resources) = response.as_array() {
        let resource_list: Vec<HaResource> = resources
            .iter()
            .filter_map(|resource| {
                let sid = resource.get("sid")?.as_str()?.to_string();
                let group = resource
                    .get("group")
                    .and_then(|g| g.as_str())
                    .map(|s| s.to_string());
                let node = resource
                    .get("node")
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string());
                let state = resource
                    .get("state")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let request_state = resource
                    .get("request_state")
                    .and_then(|r| r.as_str())
                    .map(|s| s.to_string());

                Some(HaResource {
                    sid,
                    group,
                    node,
                    state,
                    request_state,
                })
            })
            .collect();

        Ok(resource_list)
    } else {
        Err("Invalid response format: expected array".to_string())
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
        .post_form(&path, &[], Some(ticket))
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
        .post_form(&path, &[], Some(ticket))
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
        .post_form(&path, &[], Some(ticket))
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
            nodes: "pve-node-1,pve-node-2".to_string(),
            comment: None,
            restricted: Some(false),
            no_failback: Some(false),
        };

        let json = serde_json::to_string(&group).unwrap();
        let deserialized: HaGroup = serde_json::from_str(&json).unwrap();

        assert_eq!(group.group, deserialized.group);
        assert_eq!(group.nodes, deserialized.nodes);
    }

    #[test]
    fn test_ha_resource_serialization() {
        let resource = HaResource {
            sid: "vm:100".to_string(),
            group: Some("primary".to_string()),
            node: Some("pve-node-1".to_string()),
            state: "started".to_string(),
            request_state: None,
        };

        let json = serde_json::to_string(&resource).unwrap();
        let deserialized: HaResource = serde_json::from_str(&json).unwrap();

        assert_eq!(resource.sid, deserialized.sid);
        assert_eq!(resource.state, deserialized.state);
    }

    #[test]
    fn test_ha_group_nodes_is_comma_separated_string() {
        // PVE API returns nodes as a comma-separated string, not an array
        let pve_response = serde_json::json!({
            "group": "Even",
            "nodes": "vmhost2,vmhost4",
            "restricted": 0,
            "nofailback": 0,
            "type": "group"
        });

        let name = pve_response.get("group").and_then(|n| n.as_str()).unwrap();
        let nodes = pve_response.get("nodes").and_then(|n| n.as_str()).unwrap();

        assert_eq!(name, "Even");
        assert_eq!(nodes, "vmhost2,vmhost4");
        assert!(
            nodes.contains(','),
            "nodes must be a comma-separated string"
        );
    }

    #[test]
    fn test_ha_resource_uses_sid_not_resource() {
        // PVE API uses "sid" field, not "resource"
        let pve_response = serde_json::json!({
            "sid": "vm:100",
            "group": "primary",
            "state": "started",
            "node": "pve1"
        });

        let sid = pve_response.get("sid").and_then(|s| s.as_str()).unwrap();
        assert_eq!(sid, "vm:100");
        assert!(
            pve_response.get("resource").is_none(),
            "PVE API uses sid not resource"
        );
    }

    #[test]
    fn test_ha_group_serialized_id_field() {
        // Frontend expects "id" field due to #[serde(rename = "id")] on group field
        let group = HaGroup {
            group: "Odd".to_string(),
            nodes: "vmhost1,vmhost3".to_string(),
            comment: None,
            restricted: None,
            no_failback: None,
        };
        let json = serde_json::to_string(&group).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(
            v.get("id").is_some(),
            "serialized JSON must have 'id' field for frontend"
        );
        assert!(
            v.get("group").is_none(),
            "serialized JSON must not have 'group' (renamed to id)"
        );
    }
}
