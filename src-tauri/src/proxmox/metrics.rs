use serde::{Deserialize, Serialize};

/// Node metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub cpu: f64,     // CPU usage percentage
    pub memory: f64,  // Memory usage percentage
    pub disk: f64,    // Disk usage percentage
    pub network: f64, // Network usage percentage
    pub load: f64,    // Load average
    pub uptime: u64,  // Uptime in seconds
}

/// Node status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node: String,
    pub cpu: f64,
    pub memory: f64,
    pub disk: f64,
    pub load: f64,
    pub uptime: u64,
    pub version: String,
    pub status: String,
}

/// Get node metrics for a specific node
pub async fn get_node_metrics(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<NodeMetrics, String> {
    validate_node(node)?;
    let path = format!("nodes/{}/status", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get node metrics for {}: {}", node, e))?;

    {
        let data = &response;
        let cpu = data.get("cpu").and_then(|c| c.as_f64()).unwrap_or(0.0);
        let memory = data.get("memory").and_then(|m| m.as_f64()).unwrap_or(0.0);
        let disk = data.get("disk").and_then(|d| d.as_f64()).unwrap_or(0.0);
        let network = data.get("network").and_then(|n| n.as_f64()).unwrap_or(0.0);
        let load = data.get("load").and_then(|l| l.as_f64()).unwrap_or(0.0);
        let uptime = data.get("uptime").and_then(|u| u.as_u64()).unwrap_or(0);

        Ok(NodeMetrics {
            cpu,
            memory,
            disk,
            network,
            load,
            uptime,
        })
    }
}

/// List all nodes in a cluster
pub async fn list_nodes(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<NodeStatus>, String> {
    let path = "cluster/resources";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list nodes: {}", e))?;

    if let Some(resources) = response.as_array() {
        let node_list: Vec<NodeStatus> = resources
            .iter()
            .filter_map(|resource| {
                let node = resource.get("node").and_then(|n| n.as_str())?.to_string();
                let cpu = resource.get("cpu").and_then(|c| c.as_f64()).unwrap_or(0.0);
                let memory = resource
                    .get("memory")
                    .and_then(|m| m.as_f64())
                    .unwrap_or(0.0);
                let disk = resource.get("disk").and_then(|d| d.as_f64()).unwrap_or(0.0);
                let load = resource.get("load").and_then(|l| l.as_f64()).unwrap_or(0.0);
                let uptime = resource.get("uptime").and_then(|u| u.as_u64()).unwrap_or(0);
                let version = resource
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let status = resource
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                Some(NodeStatus {
                    node,
                    cpu,
                    memory,
                    disk,
                    load,
                    uptime,
                    version,
                    status,
                })
            })
            .collect();

        Ok(node_list)
    } else {
        Ok(vec![])
    }
}

/// Valid timeframe values for RRD data queries.
#[derive(Debug, Clone, PartialEq)]
pub enum RrdTimeframe {
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl RrdTimeframe {
    pub fn as_str(&self) -> &str {
        match self {
            RrdTimeframe::Hour => "hour",
            RrdTimeframe::Day => "day",
            RrdTimeframe::Week => "week",
            RrdTimeframe::Month => "month",
            RrdTimeframe::Year => "year",
        }
    }
}

impl std::str::FromStr for RrdTimeframe {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "hour" => Ok(RrdTimeframe::Hour),
            "day" => Ok(RrdTimeframe::Day),
            "week" => Ok(RrdTimeframe::Week),
            "month" => Ok(RrdTimeframe::Month),
            "year" => Ok(RrdTimeframe::Year),
            _ => Err(format!(
                "Invalid timeframe '{}' — must be one of: hour, day, week, month, year",
                s
            )),
        }
    }
}

/// Validate a Proxmox node name: alphanumeric, hyphens, underscores, dots; max 64 chars.
fn validate_node(node: &str) -> Result<(), String> {
    if node.is_empty() {
        return Err("node name must not be empty".to_string());
    }
    if node.len() > 64 {
        return Err(format!("node name exceeds 64 characters: {}", node));
    }
    if !node
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(format!(
            "node '{}' contains invalid characters — only alphanumeric, '-', '_', '.' are allowed",
            node
        ));
    }
    Ok(())
}

/// Validate a Proxmox storage name: alphanumeric, hyphens, underscores only.
fn validate_storage(storage: &str) -> Result<(), String> {
    if storage.is_empty() {
        return Err("storage name must not be empty".to_string());
    }
    if !storage
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!(
            "storage '{}' contains invalid characters — only alphanumeric, '-', '_' are allowed",
            storage
        ));
    }
    Ok(())
}

/// Validate a VMID: must be in range 100–999999999.
fn validate_vmid(vmid: u32) -> Result<(), String> {
    if !(100..=999_999_999).contains(&vmid) {
        return Err(format!("vmid {} is out of valid range 100–999999999", vmid));
    }
    Ok(())
}

/// Get RRD time-series data for a node.
///
/// `timeframe` must be one of: hour, day, week, month, year.
/// Returns raw data points as returned by Proxmox (many optional float fields).
pub async fn get_node_rrd_data(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    timeframe: &str,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    validate_node(node)?;
    let tf: RrdTimeframe = timeframe.parse()?;
    let path = format!("nodes/{}/rrddata?timeframe={}", node, tf.as_str());

    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get node RRD data for {}: {}", node, e))?;

    response
        .as_array()
        .cloned()
        .ok_or_else(|| "Unexpected response format for node RRD data: expected array".to_string())
}

/// Get RRD time-series data for a QEMU VM.
///
/// `timeframe` must be one of: hour, day, week, month, year.
/// Returns raw data points as returned by Proxmox.
pub async fn get_vm_rrd_data(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    timeframe: &str,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    validate_node(node)?;
    validate_vmid(vmid)?;
    let tf: RrdTimeframe = timeframe.parse()?;
    let path = format!(
        "nodes/{}/qemu/{}/rrddata?timeframe={}",
        node,
        vmid,
        tf.as_str()
    );

    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get VM RRD data for {}/{}: {}", node, vmid, e))?;

    response
        .as_array()
        .cloned()
        .ok_or_else(|| "Unexpected response format for VM RRD data: expected array".to_string())
}

/// Get RRD time-series data for a storage.
///
/// `timeframe` must be one of: hour, day, week, month, year.
/// Returns raw data points as returned by Proxmox.
pub async fn get_storage_rrd_data(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    storage: &str,
    timeframe: &str,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    validate_node(node)?;
    validate_storage(storage)?;
    let tf: RrdTimeframe = timeframe.parse()?;
    let path = format!(
        "nodes/{}/storage/{}/rrddata?timeframe={}",
        node,
        storage,
        tf.as_str()
    );

    let response: serde_json::Value = client.get(&path, Some(ticket)).await.map_err(|e| {
        format!(
            "Failed to get storage RRD data for {}/{}: {}",
            node, storage, e
        )
    })?;

    response.as_array().cloned().ok_or_else(|| {
        "Unexpected response format for storage RRD data: expected array".to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_metrics_serialization() {
        let metrics = NodeMetrics {
            cpu: 42.5,
            memory: 65.3,
            disk: 30.1,
            network: 15.8,
            load: 2.5,
            uptime: 86400,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: NodeMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(metrics.cpu, deserialized.cpu);
        assert_eq!(metrics.memory, deserialized.memory);
    }

    #[test]
    fn test_node_status_serialization() {
        let status = NodeStatus {
            node: "pve-node-1".to_string(),
            cpu: 42.5,
            memory: 65.3,
            disk: 30.1,
            load: 2.5,
            uptime: 86400,
            version: "7.4-15".to_string(),
            status: "online".to_string(),
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: NodeStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(status.node, deserialized.node);
        assert_eq!(status.status, "online");
    }

    #[test]
    fn test_rrd_timeframe_from_str_valid() {
        assert_eq!("hour".parse::<RrdTimeframe>().unwrap(), RrdTimeframe::Hour);
        assert_eq!("day".parse::<RrdTimeframe>().unwrap(), RrdTimeframe::Day);
        assert_eq!("week".parse::<RrdTimeframe>().unwrap(), RrdTimeframe::Week);
        assert_eq!(
            "month".parse::<RrdTimeframe>().unwrap(),
            RrdTimeframe::Month
        );
        assert_eq!("year".parse::<RrdTimeframe>().unwrap(), RrdTimeframe::Year);
    }

    #[test]
    fn test_rrd_timeframe_from_str_invalid() {
        assert!("".parse::<RrdTimeframe>().is_err());
        assert!("hourly".parse::<RrdTimeframe>().is_err());
        assert!("HOUR".parse::<RrdTimeframe>().is_err());
        assert!("minute".parse::<RrdTimeframe>().is_err());
    }

    #[test]
    fn test_rrd_timeframe_as_str_round_trips() {
        for (s, expected) in &[
            ("hour", "hour"),
            ("day", "day"),
            ("week", "week"),
            ("month", "month"),
            ("year", "year"),
        ] {
            let tf: RrdTimeframe = s.parse().unwrap();
            assert_eq!(tf.as_str(), *expected);
        }
    }

    #[test]
    fn test_validate_node_valid() {
        assert!(validate_node("pve-node1").is_ok());
        assert!(validate_node("node.local").is_ok());
        assert!(validate_node("pve_01").is_ok());
        assert!(validate_node("a").is_ok());
    }

    #[test]
    fn test_validate_node_rejects_empty() {
        assert!(validate_node("").is_err());
    }

    #[test]
    fn test_validate_node_rejects_too_long() {
        let long = "a".repeat(65);
        assert!(validate_node(&long).is_err());
    }

    #[test]
    fn test_validate_node_rejects_path_traversal() {
        assert!(validate_node("../etc/passwd").is_err());
        assert!(validate_node("node/evil").is_err());
        assert!(validate_node("node\0bad").is_err());
    }

    #[test]
    fn test_validate_storage_valid() {
        assert!(validate_storage("local").is_ok());
        assert!(validate_storage("local-lvm").is_ok());
        assert!(validate_storage("ceph_pool").is_ok());
        assert!(validate_storage("NFS1").is_ok());
    }

    #[test]
    fn test_validate_storage_rejects_empty() {
        assert!(validate_storage("").is_err());
    }

    #[test]
    fn test_validate_storage_rejects_dots_and_slashes() {
        assert!(validate_storage("stor.age").is_err());
        assert!(validate_storage("stor/age").is_err());
        assert!(validate_storage("../secret").is_err());
    }

    #[test]
    fn test_validate_vmid_valid() {
        assert!(validate_vmid(100).is_ok());
        assert!(validate_vmid(999_999_999).is_ok());
        assert!(validate_vmid(500).is_ok());
    }

    #[test]
    fn test_validate_vmid_rejects_out_of_range() {
        assert!(validate_vmid(0).is_err());
        assert!(validate_vmid(99).is_err());
        assert!(validate_vmid(1_000_000_000).is_err());
    }

    #[test]
    fn test_rrd_path_node() {
        let node = "pve-node1";
        let tf = RrdTimeframe::Hour;
        let path = format!("nodes/{}/rrddata?timeframe={}", node, tf.as_str());
        assert_eq!(path, "nodes/pve-node1/rrddata?timeframe=hour");
    }

    #[test]
    fn test_rrd_path_vm() {
        let node = "pve-node1";
        let vmid: u32 = 100;
        let tf = RrdTimeframe::Day;
        let path = format!(
            "nodes/{}/qemu/{}/rrddata?timeframe={}",
            node,
            vmid,
            tf.as_str()
        );
        assert_eq!(path, "nodes/pve-node1/qemu/100/rrddata?timeframe=day");
    }

    #[test]
    fn test_rrd_path_storage() {
        let node = "pve-node1";
        let storage = "local-lvm";
        let tf = RrdTimeframe::Week;
        let path = format!(
            "nodes/{}/storage/{}/rrddata?timeframe={}",
            node,
            storage,
            tf.as_str()
        );
        assert_eq!(
            path,
            "nodes/pve-node1/storage/local-lvm/rrddata?timeframe=week"
        );
    }
}
