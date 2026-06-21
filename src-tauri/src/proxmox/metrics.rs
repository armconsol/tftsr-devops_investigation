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
