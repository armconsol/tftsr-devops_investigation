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

/// List firewall rules — returns normalized Vec<FirewallRule> using correct PVE field names.
/// PVE uses: pos (position), proto (protocol), enable (0/1 integer), src (source), dest (destination).
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

    let rules = response.as_array().ok_or("Invalid response format")?;

    let rule_list: Vec<FirewallRule> = rules
        .iter()
        .filter_map(|rule| {
            // PVE uses "pos" for the rule position number
            let rule_num = rule.get("pos").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let action = rule.get("action").and_then(|v| v.as_str())?.to_string();
            // PVE uses "proto" not "protocol"
            let protocol = rule.get("proto").and_then(|v| v.as_str()).unwrap_or("").to_string();
            // source and dest are optional fields
            let source = rule.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let destination = rule.get("dest").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let port = rule
                .get("dport")
                .or_else(|| rule.get("sport"))
                .and_then(|p| p.as_str())
                .map(|s| s.to_string());
            // PVE uses "enable" as integer (1=enabled, 0=disabled), not "enabled" bool
            let enabled = rule
                .get("enable")
                .and_then(|e| {
                    e.as_i64()
                        .map(|n| n != 0)
                        .or_else(|| e.as_bool())
                })
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

/// Add firewall rule — uses correct PVE API field names (proto, enable, dest).
pub async fn add_rule(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    rule: &FirewallRule,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/firewall/rules", node);

    let mut config = serde_json::json!({
        "action": rule.action,
        "type": "in",
        "enable": if rule.enabled { 1 } else { 0 }
    });

    // Only include optional fields when non-empty
    if !rule.protocol.is_empty() {
        config["proto"] = serde_json::Value::String(rule.protocol.clone());
    }
    if !rule.source.is_empty() {
        config["source"] = serde_json::Value::String(rule.source.clone());
    }
    if !rule.destination.is_empty() {
        config["dest"] = serde_json::Value::String(rule.destination.clone());
    }
    if let Some(ref port) = rule.port {
        if !port.is_empty() {
            config["dport"] = serde_json::Value::String(port.clone());
        }
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
    let config = serde_json::json!({
        "action": rule.action,
        "protocol": rule.protocol,
        "source": rule.source,
        "dest": rule.destination,
        "dport": rule.port,
        "enabled": rule.enabled
    });

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
        "enabled": true
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
        "enabled": false
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
        .get("enabled")
        .and_then(|e| e.as_bool())
        .unwrap_or(false);

    let rules: Vec<FirewallRule> = rules_response
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|rule| {
            let rule_num = rule.get("pos").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let action = rule.get("action").and_then(|v| v.as_str())?.to_string();
            let protocol = rule.get("proto").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let source = rule.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let destination = rule.get("dest").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let port = rule
                .get("dport")
                .or_else(|| rule.get("sport"))
                .and_then(|p| p.as_str())
                .map(|s| s.to_string());
            let enabled = rule
                .get("enable")
                .and_then(|e| {
                    e.as_i64()
                        .map(|n| n != 0)
                        .or_else(|| e.as_bool())
                })
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
}
