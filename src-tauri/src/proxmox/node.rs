// Node management module
// Provides operations for managing Proxmox nodes

use crate::proxmox::client::ProxmoxClient;
use serde::{Deserialize, Serialize};

/// Node information (kept for compatibility — list_nodes and get_node_status
/// are implemented directly in commands/proxmox.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node: String,
    pub cpu: f64,
    pub memory: f64,
    pub disk: f64,
    pub load: f64,
    pub uptime: u64,
    pub version: String,
    pub status: String,
}

/// DNS resolver configuration for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDns {
    pub search: String,
    pub dns1: Option<String>,
    pub dns2: Option<String>,
    pub dns3: Option<String>,
}

/// Time and timezone information for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTime {
    pub localtime: i64,
    pub time: i64,
    pub timezone: String,
}

/// Validate a node name: alphanumeric and hyphens only, max 64 chars.
fn validate_node_name(node: &str) -> Result<(), String> {
    if node.is_empty() || node.len() > 64 {
        return Err("Invalid node name".to_string());
    }
    if !node.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("Invalid node name".to_string());
    }
    Ok(())
}

/// List all nodes (stub — implemented in commands/proxmox.rs)
pub async fn list_nodes(_client: &ProxmoxClient, _ticket: &str) -> Result<Vec<NodeInfo>, String> {
    Err("Not implemented yet".to_string())
}

/// Get node status (stub — implemented in commands/proxmox.rs)
pub async fn get_node_status(
    _client: &ProxmoxClient,
    _node: &str,
    _ticket: &str,
) -> Result<NodeInfo, String> {
    Err("Not implemented yet".to_string())
}

/// Get DNS configuration for a node
pub async fn get_node_dns(
    client: &ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<NodeDns, String> {
    validate_node_name(node)?;
    let path = format!("nodes/{}/dns", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get DNS config for node {}: {}", node, e))?;

    serde_json::from_value(response).map_err(|e| format!("Failed to deserialize DNS config: {}", e))
}

/// Update DNS configuration for a node
pub async fn update_node_dns(
    client: &ProxmoxClient,
    node: &str,
    search: &str,
    dns1: Option<&str>,
    dns2: Option<&str>,
    dns3: Option<&str>,
    ticket: &str,
) -> Result<(), String> {
    validate_node_name(node)?;
    let path = format!("nodes/{}/dns", node);

    let mut body = serde_json::json!({ "search": search });
    if let Some(v) = dns1 {
        body["dns1"] = serde_json::Value::String(v.to_string());
    }
    if let Some(v) = dns2 {
        body["dns2"] = serde_json::Value::String(v.to_string());
    }
    if let Some(v) = dns3 {
        body["dns3"] = serde_json::Value::String(v.to_string());
    }

    let _response: serde_json::Value = client
        .put(&path, &body, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update DNS config for node {}: {}", node, e))?;

    Ok(())
}

/// Get time and timezone information for a node
pub async fn get_node_time(
    client: &ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<NodeTime, String> {
    validate_node_name(node)?;
    let path = format!("nodes/{}/time", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get time for node {}: {}", node, e))?;

    serde_json::from_value(response)
        .map_err(|e| format!("Failed to deserialize time response: {}", e))
}

/// Update the timezone for a node
pub async fn update_node_time(
    client: &ProxmoxClient,
    node: &str,
    timezone: &str,
    ticket: &str,
) -> Result<(), String> {
    validate_node_name(node)?;
    let path = format!("nodes/{}/time", node);
    let body = serde_json::json!({ "timezone": timezone });

    let _response: serde_json::Value = client
        .put(&path, &body, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update timezone for node {}: {}", node, e))?;

    Ok(())
}

/// Reboot a node. Returns the task UPID.
pub async fn reboot_node(
    client: &ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<String, String> {
    validate_node_name(node)?;
    let path = format!("nodes/{}/status", node);
    let response: serde_json::Value = client
        .post_form(&path, &[("command", "reboot")], Some(ticket))
        .await
        .map_err(|e| format!("Failed to reboot node {}: {}", node, e))?;

    response
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Unexpected response format for reboot UPID".to_string())
}

/// Shut down a node. Returns the task UPID.
pub async fn shutdown_node(
    client: &ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<String, String> {
    validate_node_name(node)?;
    let path = format!("nodes/{}/status", node);
    let response: serde_json::Value = client
        .post_form(&path, &[("command", "shutdown")], Some(ticket))
        .await
        .map_err(|e| format!("Failed to shut down node {}: {}", node, e))?;

    response
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Unexpected response format for shutdown UPID".to_string())
}

/// Get journal log entries for a node
pub async fn get_node_journal(
    client: &ProxmoxClient,
    node: &str,
    lastentries: u32,
    ticket: &str,
) -> Result<Vec<String>, String> {
    validate_node_name(node)?;
    let path = format!("nodes/{}/journal?lastentries={}", node, lastentries);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get journal for node {}: {}", node, e))?;

    serde_json::from_value::<Vec<String>>(response)
        .map_err(|e| format!("Failed to deserialize journal entries: {}", e))
}

/// Get a full diagnostic report for a node
pub async fn get_node_report(
    client: &ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<String, String> {
    validate_node_name(node)?;
    let path = format!("nodes/{}/report", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get report for node {}: {}", node, e))?;

    response
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Unexpected response format for node report".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── NodeInfo (existing struct) ───────────────────────────────────────────

    #[test]
    fn test_node_info_serialization() {
        let node = NodeInfo {
            node: "pve-node-1".to_string(),
            cpu: 0.42,
            memory: 0.65,
            disk: 0.30,
            load: 2.5,
            uptime: 86400,
            version: "7.4-15".to_string(),
            status: "online".to_string(),
        };

        let json = serde_json::to_string(&node).unwrap();
        let deserialized: NodeInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(node.node, deserialized.node);
        assert_eq!(node.status, "online");
    }

    // ── NodeDns ─────────────────────────────────────────────────────────────

    #[test]
    fn test_node_dns_deserialize_full() {
        let json = r#"{
            "search": "tftsr.com",
            "dns1": "172.0.0.1",
            "dns2": "8.8.8.8",
            "dns3": "1.1.1.1"
        }"#;
        let dns: NodeDns = serde_json::from_str(json).unwrap();
        assert_eq!(dns.search, "tftsr.com");
        assert_eq!(dns.dns1.as_deref(), Some("172.0.0.1"));
        assert_eq!(dns.dns2.as_deref(), Some("8.8.8.8"));
        assert_eq!(dns.dns3.as_deref(), Some("1.1.1.1"));
    }

    #[test]
    fn test_node_dns_deserialize_partial() {
        // Only dns1 present — dns2 and dns3 must be None, not an error.
        let json = r#"{"search": "home.lab", "dns1": "192.168.1.1"}"#;
        let dns: NodeDns = serde_json::from_str(json).unwrap();
        assert_eq!(dns.search, "home.lab");
        assert_eq!(dns.dns1.as_deref(), Some("192.168.1.1"));
        assert!(dns.dns2.is_none());
        assert!(dns.dns3.is_none());
    }

    #[test]
    fn test_node_dns_roundtrip() {
        let original = NodeDns {
            search: "example.com".to_string(),
            dns1: Some("1.1.1.1".to_string()),
            dns2: None,
            dns3: None,
        };
        let json = serde_json::to_string(&original).unwrap();
        let roundtripped: NodeDns = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtripped.search, original.search);
        assert_eq!(roundtripped.dns1, original.dns1);
    }

    // ── NodeTime ─────────────────────────────────────────────────────────────

    #[test]
    fn test_node_time_deserialize() {
        let json = r#"{
            "localtime": 1782248412,
            "time": 1782266412,
            "timezone": "America/Chicago"
        }"#;
        let time: NodeTime = serde_json::from_str(json).unwrap();
        assert_eq!(time.localtime, 1782248412);
        assert_eq!(time.time, 1782266412);
        assert_eq!(time.timezone, "America/Chicago");
    }

    #[test]
    fn test_node_time_roundtrip() {
        let original = NodeTime {
            localtime: 1000000,
            time: 1000100,
            timezone: "UTC".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let roundtripped: NodeTime = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtripped.timezone, original.timezone);
        assert_eq!(roundtripped.time, original.time);
    }

    // ── Node name validation ─────────────────────────────────────────────────

    #[test]
    fn test_validate_node_name_accepts_valid() {
        assert!(validate_node_name("pve").is_ok());
        assert!(validate_node_name("pve-node-1").is_ok());
        assert!(validate_node_name("NODE01").is_ok());
        assert!(validate_node_name("a").is_ok());
        // Exactly 64 chars
        assert!(validate_node_name(&"a".repeat(64)).is_ok());
    }

    #[test]
    fn test_validate_node_name_rejects_empty() {
        assert_eq!(validate_node_name(""), Err("Invalid node name".to_string()));
    }

    #[test]
    fn test_validate_node_name_rejects_too_long() {
        assert_eq!(
            validate_node_name(&"a".repeat(65)),
            Err("Invalid node name".to_string())
        );
    }

    #[test]
    fn test_validate_node_name_rejects_illegal_chars() {
        assert_eq!(
            validate_node_name("pve.node"),
            Err("Invalid node name".to_string())
        );
        assert_eq!(
            validate_node_name("node/1"),
            Err("Invalid node name".to_string())
        );
        assert_eq!(
            validate_node_name("node 1"),
            Err("Invalid node name".to_string())
        );
        assert_eq!(
            validate_node_name("../etc"),
            Err("Invalid node name".to_string())
        );
    }

    // ── Path construction ────────────────────────────────────────────────────

    #[test]
    fn test_dns_path_format() {
        let node = "pve-node-1";
        let path = format!("nodes/{}/dns", node);
        assert_eq!(path, "nodes/pve-node-1/dns");
    }

    #[test]
    fn test_time_path_format() {
        let node = "pve01";
        let path = format!("nodes/{}/time", node);
        assert_eq!(path, "nodes/pve01/time");
    }

    #[test]
    fn test_journal_path_includes_lastentries() {
        let node = "pve01";
        let lastentries: u32 = 100;
        let path = format!("nodes/{}/journal?lastentries={}", node, lastentries);
        assert_eq!(path, "nodes/pve01/journal?lastentries=100");
    }

    #[test]
    fn test_status_path_format() {
        let node = "pve01";
        let path = format!("nodes/{}/status", node);
        assert_eq!(path, "nodes/pve01/status");
    }

    #[test]
    fn test_report_path_format() {
        let node = "pve01";
        let path = format!("nodes/{}/report", node);
        assert_eq!(path, "nodes/pve01/report");
    }

    // ── Journal/report response deserialization ──────────────────────────────

    #[test]
    fn test_journal_deserialize_array_of_strings() {
        let json = serde_json::json!([
            "Jun 01 12:00:00 pve01 kernel: line one",
            "Jun 01 12:00:01 pve01 kernel: line two"
        ]);
        let entries: Vec<String> = serde_json::from_value(json).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries[0].contains("line one"));
    }

    #[test]
    fn test_report_deserialize_plain_string() {
        let response = serde_json::Value::String("full report text here".to_string());
        let report = response
            .as_str()
            .map(|s| s.to_string())
            .expect("should be a string");
        assert_eq!(report, "full report text here");
    }
}
