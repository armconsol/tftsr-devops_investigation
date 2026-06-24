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

/// Get firewall zone configuration
pub async fn get_firewall_zone(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    zone: &str,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    let path = format!("nodes/{}/firewall/zones/{}", node, zone);
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
        let pve_rule = serde_json::json!({"pos": 1, "action": "ACCEPT", "proto": "tcp", "enable": 1});
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
        assert!(pve_rule.get("protocol").is_none(), "PVE uses 'proto' not 'protocol'");
    }

    #[test]
    fn test_firewall_pve_uses_pos_not_rule_num() {
        // PVE API field is "pos", not "rule_num"
        let pve_rule = serde_json::json!({"pos": 5, "action": "ACCEPT"});
        let pos = pve_rule.get("pos").and_then(|p| p.as_u64()).unwrap();
        assert_eq!(pos, 5);
        assert!(pve_rule.get("rule_num").is_none(), "PVE uses 'pos' not 'rule_num'");
    }
}
