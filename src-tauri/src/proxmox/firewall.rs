// Firewall management module
// Provides operations for managing Proxmox firewall

use serde::{Deserialize, Serialize};

/// Firewall rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub rule_num: u32,
    pub action: String,
    pub protocol: String,
    pub source: String,
    pub destination: String,
    pub port: Option<String>,
    pub enabled: bool,
}

/// Firewall status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallStatus {
    pub enabled: bool,
    pub rules: Vec<FirewallRule>,
    pub rule_count: u32,
}

/// List firewall rules
pub async fn list_firewall_rules(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<FirewallRule>, String> {
    validate_node(node)?;
    let path = format!("nodes/{}/firewall/rules", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list firewall rules: {}", e))?;

    if let Some(rules) = response.as_array() {
        let rule_list: Vec<FirewallRule> = rules
            .iter()
            .filter_map(|rule| {
                let rule_num = rule.get("pos")?.as_u64()? as u32;
                let action = rule.get("action")?.as_str()?.to_string();
                let protocol = rule.get("proto")?.as_str().unwrap_or("").to_string();
                let source = rule.get("source")?.as_str().unwrap_or("").to_string();
                let destination = rule.get("dest")?.as_str().unwrap_or("").to_string();
                let port = rule
                    .get("dport")
                    .or(rule.get("sport"))
                    .and_then(|p| p.as_str())
                    .map(|s| s.to_string());
                let enabled = rule
                    .get("enable")
                    .and_then(|e| e.as_i64())
                    .map(|n| n != 0)
                    .unwrap_or(true);

                Some(FirewallRule {
                    rule_num,
                    action,
                    protocol,
                    source,
                    destination,
                    port,
                    enabled,
                })
            })
            .collect();

        Ok(rule_list)
    } else {
        Err("Invalid response format".to_string())
    }
}

/// Add firewall rule
pub async fn add_rule(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    rule: &FirewallRule,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{}/firewall/rules", node);
    let mut config = serde_json::json!({
        "action": rule.action,
        "proto": rule.protocol,
        "source": rule.source,
        "dest": rule.destination,
        "enable": if rule.enabled { 1 } else { 0 }
    });
    if let Some(ref port) = rule.port {
        config["dport"] = serde_json::Value::String(port.clone());
    }

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to add firewall rule: {}", e))?;
    Ok(())
}

/// Delete firewall rule
pub async fn delete_rule(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    rule_num: u32,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{}/firewall/rules/{}", node, rule_num);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete firewall rule {}: {}", rule_num, e))?;
    Ok(())
}

/// Update firewall rule
pub async fn update_rule(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    rule_num: u32,
    rule: &FirewallRule,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{}/firewall/rules/{}", node, rule_num);
    let mut config = serde_json::json!({
        "action": rule.action,
        "proto": rule.protocol,
        "source": rule.source,
        "dest": rule.destination,
        "enable": if rule.enabled { 1 } else { 0 }
    });
    if let Some(ref port) = rule.port {
        config["dport"] = serde_json::Value::String(port.clone());
    }

    let _response: serde_json::Value = client
        .put(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update firewall rule {}: {}", rule_num, e))?;
    Ok(())
}

/// Enable firewall
pub async fn enable_firewall(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{}/firewall/options", node);
    let config = serde_json::json!({
        "enable": 1
    });

    let _response: serde_json::Value = client
        .put(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to enable firewall: {}", e))?;
    Ok(())
}

/// Disable firewall
pub async fn disable_firewall(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    let path = format!("nodes/{}/firewall/options", node);
    let config = serde_json::json!({
        "enable": 0
    });

    let _response: serde_json::Value = client
        .put(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to disable firewall: {}", e))?;
    Ok(())
}

/// Get firewall status
pub async fn get_firewall_status(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<FirewallStatus, String> {
    validate_node(node)?;
    let path = format!("nodes/{}/firewall/rules", node);
    let rules_response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get firewall rules: {}", e))?;

    let enabled_path = format!("nodes/{}/firewall/options", node);
    let options_response: serde_json::Value = client
        .get(&enabled_path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get firewall options: {}", e))?;

    let enabled = options_response
        .get("enable")
        .and_then(|e| e.as_i64())
        .map(|n| n != 0)
        .unwrap_or(false);

    let empty = Vec::new();
    let rules: Vec<FirewallRule> = rules_response
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter_map(|rule| {
            let rule_num = rule.get("pos")?.as_u64()? as u32;
            let action = rule.get("action")?.as_str()?.to_string();
            let protocol = rule.get("proto")?.as_str().unwrap_or("").to_string();
            let source = rule.get("source")?.as_str().unwrap_or("").to_string();
            let destination = rule.get("dest")?.as_str().unwrap_or("").to_string();
            let port = rule
                .get("dport")
                .or(rule.get("sport"))
                .and_then(|p| p.as_str())
                .map(|s| s.to_string());
            let enabled = rule
                .get("enable")
                .and_then(|e| e.as_i64())
                .map(|n| n != 0)
                .unwrap_or(true);

            Some(FirewallRule {
                rule_num,
                action,
                protocol,
                source,
                destination,
                port,
                enabled,
            })
        })
        .collect();

    let rule_count = rules.len() as u32;

    Ok(FirewallStatus {
        enabled,
        rules,
        rule_count,
    })
}

fn validate_node(node: &str) -> Result<(), String> {
    if node.is_empty() || node.len() > 64 {
        return Err("Node name must be between 1 and 64 characters".to_string());
    }
    if !node.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(format!(
            "Invalid node name '{}': only alphanumeric characters and hyphens are allowed",
            node
        ));
    }
    Ok(())
}

fn validate_vmid(vmid: u32) -> Result<(), String> {
    if !(100..=999_999_999).contains(&vmid) {
        return Err(format!(
            "Invalid vmid {}: must be in range 100..=999999999",
            vmid
        ));
    }
    Ok(())
}

fn validate_firewall_zone(zone: &str) -> Result<(), String> {
    if zone.is_empty() || zone.len() > 64 {
        return Err("Firewall zone must be between 1 and 64 characters".to_string());
    }
    if !zone
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!(
            "Invalid firewall zone '{}': only alphanumeric characters, hyphens, and underscores are allowed",
            zone
        ));
    }
    Ok(())
}

/// Cluster-level firewall status (distinct from per-node FirewallStatus)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterFirewallStatus {
    pub enable: Option<i64>,
    pub policy_in: Option<String>,
    pub policy_out: Option<String>,
}

/// Parse a PVE firewall rules array into `Vec<FirewallRule>`
fn parse_firewall_rules(response: serde_json::Value) -> Result<Vec<FirewallRule>, String> {
    let rules = response
        .as_array()
        .ok_or_else(|| "Invalid response format".to_string())?;

    let rule_list = rules
        .iter()
        .filter_map(|rule| {
            let rule_num = rule.get("pos")?.as_u64()? as u32;
            let action = rule.get("action")?.as_str()?.to_string();
            let protocol = rule.get("proto")?.as_str().unwrap_or("").to_string();
            let source = rule.get("source")?.as_str().unwrap_or("").to_string();
            let destination = rule.get("dest")?.as_str().unwrap_or("").to_string();
            let port = rule
                .get("dport")
                .or(rule.get("sport"))
                .and_then(|p| p.as_str())
                .map(|s| s.to_string());
            let enabled = rule
                .get("enable")
                .and_then(|e| e.as_i64())
                .map(|n| n != 0)
                .unwrap_or(true);
            Some(FirewallRule {
                rule_num,
                action,
                protocol,
                source,
                destination,
                port,
                enabled,
            })
        })
        .collect();

    Ok(rule_list)
}

/// List cluster-level firewall rules
pub async fn list_cluster_firewall_rules(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<FirewallRule>, String> {
    let response: serde_json::Value = client
        .get("cluster/firewall/rules", Some(ticket))
        .await
        .map_err(|e| format!("Failed to list cluster firewall rules: {}", e))?;
    parse_firewall_rules(response)
}

/// Get cluster-level firewall options (enable flag, default policies)
pub async fn get_cluster_firewall_status(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<ClusterFirewallStatus, String> {
    let response: serde_json::Value = client
        .get("cluster/firewall/options", Some(ticket))
        .await
        .map_err(|e| format!("Failed to get cluster firewall options: {}", e))?;

    let enable = response.get("enable").and_then(|v| v.as_i64());
    let policy_in = response
        .get("policy_in")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let policy_out = response
        .get("policy_out")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(ClusterFirewallStatus {
        enable,
        policy_in,
        policy_out,
    })
}

/// List firewall rules for a guest VM
pub async fn list_guest_firewall_rules(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<Vec<FirewallRule>, String> {
    validate_node(node)?;
    validate_vmid(vmid)?;
    let path = format!("nodes/{}/qemu/{}/firewall/rules", node, vmid);
    let response: serde_json::Value = client.get(&path, Some(ticket)).await.map_err(|e| {
        format!(
            "Failed to list guest firewall rules for vmid {}: {}",
            vmid, e
        )
    })?;
    parse_firewall_rules(response)
}

/// Add a firewall rule to a guest VM
pub async fn add_guest_firewall_rule(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    rule: &FirewallRule,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    validate_vmid(vmid)?;
    let path = format!("nodes/{}/qemu/{}/firewall/rules", node, vmid);
    let mut config = serde_json::json!({
        "action": rule.action,
        "proto": rule.protocol,
        "source": rule.source,
        "dest": rule.destination,
        "enable": if rule.enabled { 1 } else { 0 }
    });
    if let Some(ref port) = rule.port {
        config["dport"] = serde_json::Value::String(port.clone());
    }
    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to add guest firewall rule for vmid {}: {}", vmid, e))?;
    Ok(())
}

/// Delete a firewall rule from a guest VM by position
pub async fn delete_guest_firewall_rule(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    pos: u32,
    ticket: &str,
) -> Result<(), String> {
    validate_node(node)?;
    validate_vmid(vmid)?;
    let path = format!("nodes/{}/qemu/{}/firewall/rules/{}", node, vmid, pos);
    let _response: serde_json::Value = client.delete(&path, Some(ticket)).await.map_err(|e| {
        format!(
            "Failed to delete guest firewall rule at pos {} for vmid {}: {}",
            pos, vmid, e
        )
    })?;
    Ok(())
}

/// Get firewall zone configuration
pub async fn get_firewall_zone(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    zone: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    validate_node(node)?;
    validate_firewall_zone(zone)?;
    let path = format!(
        "nodes/{}/firewall/zones/{}",
        node,
        urlencoding::encode(zone)
    );
    client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get firewall zone {}: {}", zone, e))
}

/// List firewall zones
pub async fn list_firewall_zones(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    validate_node(node)?;
    let path = format!("nodes/{}/firewall/zones", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list firewall zones: {}", e))?;

    if let Some(zones) = response.as_array() {
        Ok(zones.to_vec())
    } else {
        Err("Invalid response format".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firewall_rule_serialization() {
        let rule = FirewallRule {
            rule_num: 1,
            action: "ACCEPT".to_string(),
            protocol: "tcp".to_string(),
            source: "10.0.0.0/8".to_string(),
            destination: "any".to_string(),
            port: Some("443".to_string()),
            enabled: true,
        };

        let json = serde_json::to_string(&rule).unwrap();
        let deserialized: FirewallRule = serde_json::from_str(&json).unwrap();

        assert_eq!(rule.action, deserialized.action);
        assert_eq!(rule.enabled, deserialized.enabled);
    }

    #[test]
    fn test_firewall_status_serialization() {
        let status = FirewallStatus {
            enabled: true,
            rules: vec![],
            rule_count: 0,
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: FirewallStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(status.enabled, deserialized.enabled);
    }

    #[test]
    fn test_firewall_enable_integer_1_is_enabled() {
        // PVE API uses integer 1/0 for enable, not boolean
        let pve_rule =
            serde_json::json!({"pos": 1, "action": "ACCEPT", "proto": "tcp", "enable": 1});
        let enabled = pve_rule
            .get("enable")
            .and_then(|e| e.as_i64())
            .map(|n| n != 0)
            .unwrap_or(true);
        assert!(enabled, "enable=1 must parse as true");
    }

    #[test]
    fn test_firewall_enable_integer_0_is_disabled() {
        let pve_rule = serde_json::json!({"pos": 2, "action": "DROP", "enable": 0});
        let enabled = pve_rule
            .get("enable")
            .and_then(|e| e.as_i64())
            .map(|n| n != 0)
            .unwrap_or(true);
        assert!(!enabled, "enable=0 must parse as false");
    }

    #[test]
    fn test_firewall_pve_uses_proto_not_protocol() {
        // PVE API field is "proto", not "protocol"
        let pve_rule = serde_json::json!({"pos": 1, "action": "ACCEPT", "proto": "udp"});
        let proto = pve_rule.get("proto").and_then(|p| p.as_str()).unwrap_or("");
        assert_eq!(proto, "udp");
        assert!(
            pve_rule.get("protocol").is_none(),
            "PVE uses 'proto' not 'protocol'"
        );
    }

    #[test]
    fn test_firewall_pve_uses_pos_not_rule_num() {
        // PVE API field is "pos", not "rule_num"
        let pve_rule = serde_json::json!({"pos": 5, "action": "ACCEPT"});
        let pos = pve_rule.get("pos").and_then(|p| p.as_u64()).unwrap();
        assert_eq!(pos, 5);
        assert!(
            pve_rule.get("rule_num").is_none(),
            "PVE uses 'pos' not 'rule_num'"
        );
    }

    #[test]
    fn test_cluster_firewall_status_deserialization() {
        let fixture = serde_json::json!({
            "enable": 1,
            "policy_in": "DROP",
            "policy_out": "ACCEPT"
        });

        let enable = fixture.get("enable").and_then(|v| v.as_i64());
        let policy_in = fixture
            .get("policy_in")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let policy_out = fixture
            .get("policy_out")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let status = ClusterFirewallStatus {
            enable,
            policy_in,
            policy_out,
        };

        assert_eq!(status.enable, Some(1));
        assert_eq!(status.policy_in.as_deref(), Some("DROP"));
        assert_eq!(status.policy_out.as_deref(), Some("ACCEPT"));
    }

    #[test]
    fn test_cluster_firewall_status_partial_fixture() {
        // policy fields are optional — cluster firewall may only have enable set
        let fixture = serde_json::json!({"enable": 0});
        let status = ClusterFirewallStatus {
            enable: fixture.get("enable").and_then(|v| v.as_i64()),
            policy_in: fixture
                .get("policy_in")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            policy_out: fixture
                .get("policy_out")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        assert_eq!(status.enable, Some(0));
        assert!(status.policy_in.is_none());
        assert!(status.policy_out.is_none());
    }

    #[test]
    fn test_cluster_firewall_status_roundtrip() {
        let status = ClusterFirewallStatus {
            enable: Some(1),
            policy_in: Some("REJECT".to_string()),
            policy_out: Some("ACCEPT".to_string()),
        };
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: ClusterFirewallStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status.enable, deserialized.enable);
        assert_eq!(status.policy_in, deserialized.policy_in);
        assert_eq!(status.policy_out, deserialized.policy_out);
    }

    #[test]
    fn test_guest_firewall_path_building() {
        let node = "pve-node-01";
        let vmid: u32 = 100;
        let pos: u32 = 3;
        let rules_path = format!("nodes/{}/qemu/{}/firewall/rules", node, vmid);
        let delete_path = format!("nodes/{}/qemu/{}/firewall/rules/{}", node, vmid, pos);
        assert_eq!(rules_path, "nodes/pve-node-01/qemu/100/firewall/rules");
        assert_eq!(delete_path, "nodes/pve-node-01/qemu/100/firewall/rules/3");
    }

    #[test]
    fn test_guest_firewall_vmid_boundary_99_rejected() {
        assert!(
            validate_vmid(99).is_err(),
            "vmid 99 is below the minimum of 100"
        );
    }

    #[test]
    fn test_guest_firewall_vmid_boundary_100_accepted() {
        assert!(
            validate_vmid(100).is_ok(),
            "vmid 100 is the minimum valid vmid"
        );
    }

    #[test]
    fn test_guest_firewall_vmid_boundary_999999999_accepted() {
        assert!(
            validate_vmid(999_999_999).is_ok(),
            "vmid 999999999 is the maximum valid vmid"
        );
    }

    #[test]
    fn test_guest_firewall_vmid_boundary_1000000000_rejected() {
        assert!(
            validate_vmid(1_000_000_000).is_err(),
            "vmid 1000000000 exceeds the maximum of 999999999"
        );
    }

    #[test]
    fn test_validate_node_rejects_path_traversal() {
        assert!(validate_node("../etc").is_err());
        assert!(validate_node("node/bad").is_err());
        assert!(validate_node("node\\bad").is_err());
        assert!(validate_node("node with space").is_err());
    }

    #[test]
    fn test_validate_node_accepts_valid_names() {
        assert!(validate_node("pve-node-01").is_ok());
        assert!(validate_node("pve1").is_ok());
    }

    #[test]
    fn test_validate_node_rejects_empty_and_too_long() {
        assert!(validate_node("").is_err());
        assert!(validate_node(&"a".repeat(65)).is_err());
        assert!(validate_node(&"a".repeat(64)).is_ok());
    }

    #[test]
    fn test_validate_firewall_zone_valid() {
        assert!(validate_firewall_zone("trusted").is_ok());
        assert!(validate_firewall_zone("dmz-1").is_ok());
        assert!(validate_firewall_zone("zone_a").is_ok());
    }

    #[test]
    fn test_validate_firewall_zone_rejects_injection() {
        assert!(validate_firewall_zone("").is_err());
        assert!(validate_firewall_zone(&"z".repeat(65)).is_err());
        assert!(validate_firewall_zone("../etc").is_err());
        assert!(validate_firewall_zone("zone/sub").is_err());
        assert!(validate_firewall_zone("zone name").is_err());
    }
}
