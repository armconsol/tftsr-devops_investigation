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
    _client: &crate::proxmox::client::ProxmoxClient,
    _node: &str,
    _ticket: &str,
) -> Result<NodeMetrics, String> {
    // Implementation will be completed in Phase 2
    Err("Not implemented yet".to_string())
}

/// List all nodes in a cluster
pub async fn list_nodes(
    _client: &crate::proxmox::client::ProxmoxClient,
    _ticket: &str,
) -> Result<Vec<NodeStatus>, String> {
    // Implementation will be completed in Phase 2
    Err("Not implemented yet".to_string())
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
}
