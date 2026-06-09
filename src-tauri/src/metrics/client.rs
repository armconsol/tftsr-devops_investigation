use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PodMetrics {
    pub name: String,
    pub namespace: String,
    pub containers: Vec<ContainerMetrics>,
    pub cpu: String,    // e.g., "100m"
    pub memory: String, // e.g., "256Mi"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerMetrics {
    pub name: String,
    pub cpu: String,
    pub memory: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeMetrics {
    pub name: String,
    pub cpu: String,
    pub memory: String,
    pub cpu_percent: f64,
    pub memory_percent: f64,
}

/// Parse kubectl top pods output (JSON format)
pub fn parse_pod_metrics(json_output: &str) -> Result<Vec<PodMetrics>> {
    let value: serde_json::Value =
        serde_json::from_str(json_output).context("Failed to parse kubectl top pods JSON")?;

    let items = value
        .get("items")
        .and_then(|v| v.as_array())
        .context("Missing items array")?;

    let mut metrics = Vec::new();

    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();

        let namespace = item
            .get("metadata")
            .and_then(|m| m.get("namespace"))
            .and_then(|n| n.as_str())
            .unwrap_or("default")
            .to_string();

        let containers_data = item.get("containers").and_then(|c| c.as_array());

        let mut containers = Vec::new();
        let mut total_cpu_nano = 0u64;
        let mut total_memory_kb = 0u64;

        if let Some(containers_data) = containers_data {
            for container in containers_data {
                let container_name = container
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();

                let cpu_usage = container
                    .get("usage")
                    .and_then(|u| u.get("cpu"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("0")
                    .to_string();

                let memory_usage = container
                    .get("usage")
                    .and_then(|u| u.get("memory"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("0")
                    .to_string();

                // Parse for totals
                total_cpu_nano += parse_cpu_to_nanocores(&cpu_usage);
                total_memory_kb += parse_memory_to_kb(&memory_usage);

                containers.push(ContainerMetrics {
                    name: container_name,
                    cpu: cpu_usage,
                    memory: memory_usage,
                });
            }
        }

        metrics.push(PodMetrics {
            name,
            namespace,
            containers,
            cpu: format_cpu_from_nanocores(total_cpu_nano),
            memory: format_memory_from_kb(total_memory_kb),
        });
    }

    Ok(metrics)
}

/// Parse kubectl top nodes output (JSON format)
pub fn parse_node_metrics(json_output: &str) -> Result<Vec<NodeMetrics>> {
    let value: serde_json::Value =
        serde_json::from_str(json_output).context("Failed to parse kubectl top nodes JSON")?;

    let items = value
        .get("items")
        .and_then(|v| v.as_array())
        .context("Missing items array")?;

    let mut metrics = Vec::new();

    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();

        let cpu = item
            .get("usage")
            .and_then(|u| u.get("cpu"))
            .and_then(|c| c.as_str())
            .unwrap_or("0")
            .to_string();

        let memory = item
            .get("usage")
            .and_then(|u| u.get("memory"))
            .and_then(|m| m.as_str())
            .unwrap_or("0")
            .to_string();

        // Calculate percentages (simplified - would need capacity from kubectl get nodes)
        let cpu_percent = 0.0; // TODO: Calculate from capacity
        let memory_percent = 0.0; // TODO: Calculate from capacity

        metrics.push(NodeMetrics {
            name,
            cpu,
            memory,
            cpu_percent,
            memory_percent,
        });
    }

    Ok(metrics)
}

/// Parse CPU string to nanocores (e.g., "100m" -> 100000000, "2" -> 2000000000)
fn parse_cpu_to_nanocores(cpu: &str) -> u64 {
    if cpu.ends_with('n') {
        cpu.trim_end_matches('n').parse::<u64>().unwrap_or(0)
    } else if cpu.ends_with('u') {
        cpu.trim_end_matches('u').parse::<u64>().unwrap_or(0) * 1000
    } else if cpu.ends_with('m') {
        cpu.trim_end_matches('m').parse::<u64>().unwrap_or(0) * 1_000_000
    } else {
        cpu.parse::<u64>().unwrap_or(0) * 1_000_000_000
    }
}

/// Parse memory string to kilobytes (e.g., "256Mi" -> 262144, "1Gi" -> 1048576)
fn parse_memory_to_kb(memory: &str) -> u64 {
    if memory.ends_with("Ki") {
        memory.trim_end_matches("Ki").parse::<u64>().unwrap_or(0)
    } else if memory.ends_with("Mi") {
        memory.trim_end_matches("Mi").parse::<u64>().unwrap_or(0) * 1024
    } else if memory.ends_with("Gi") {
        memory.trim_end_matches("Gi").parse::<u64>().unwrap_or(0) * 1024 * 1024
    } else if memory.ends_with("Ti") {
        memory.trim_end_matches("Ti").parse::<u64>().unwrap_or(0) * 1024 * 1024 * 1024
    } else {
        memory.parse::<u64>().unwrap_or(0) / 1024 // Assume bytes
    }
}

/// Format nanocores back to human-readable (e.g., 100000000 -> "100m")
fn format_cpu_from_nanocores(nanocores: u64) -> String {
    if nanocores >= 1_000_000_000 {
        format!("{:.1}", nanocores as f64 / 1_000_000_000.0)
    } else {
        format!("{}m", nanocores / 1_000_000)
    }
}

/// Format kilobytes back to human-readable (e.g., 262144 -> "256Mi")
fn format_memory_from_kb(kb: u64) -> String {
    if kb >= 1024 * 1024 * 1024 {
        format!("{:.1}Ti", kb as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if kb >= 1024 * 1024 {
        format!("{:.0}Gi", kb as f64 / (1024.0 * 1024.0))
    } else if kb >= 1024 {
        format!("{:.0}Mi", kb as f64 / 1024.0)
    } else {
        format!("{}Ki", kb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu() {
        assert_eq!(parse_cpu_to_nanocores("100m"), 100_000_000);
        assert_eq!(parse_cpu_to_nanocores("2"), 2_000_000_000);
        assert_eq!(parse_cpu_to_nanocores("500u"), 500_000);
    }

    #[test]
    fn test_parse_memory() {
        assert_eq!(parse_memory_to_kb("256Mi"), 262_144);
        assert_eq!(parse_memory_to_kb("1Gi"), 1_048_576);
        assert_eq!(parse_memory_to_kb("512Ki"), 512);
    }

    #[test]
    fn test_format_cpu() {
        assert_eq!(format_cpu_from_nanocores(100_000_000), "100m");
        assert_eq!(format_cpu_from_nanocores(2_000_000_000), "2.0");
    }

    #[test]
    fn test_format_memory() {
        assert_eq!(format_memory_from_kb(262_144), "256Mi");
        assert_eq!(format_memory_from_kb(1_048_576), "1Gi");
    }
}
