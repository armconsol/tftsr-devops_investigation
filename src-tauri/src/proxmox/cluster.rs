use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cluster information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub id: String,
    pub name: String,
    pub cluster_type: ClusterType,
    pub url: String,
    pub port: u16,
    pub username: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Cluster type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ClusterType {
    #[default]
    VE,  // Proxmox VE
    PBS, // Proxmox Backup Server
}

/// Cluster registry for managing multiple clusters
pub struct ClusterRegistry {
    clusters: HashMap<String, ClusterInfo>,
}

impl ClusterRegistry {
    /// Create a new cluster registry
    pub fn new() -> Self {
        Self {
            clusters: HashMap::new(),
        }
    }

    /// Add a cluster
    pub fn add_cluster(&mut self, cluster: ClusterInfo) {
        self.clusters.insert(cluster.id.clone(), cluster);
    }

    /// Remove a cluster
    pub fn remove_cluster(&mut self, id: &str) -> Option<ClusterInfo> {
        self.clusters.remove(id)
    }

    /// Get a cluster by ID
    pub fn get_cluster(&self, id: &str) -> Option<&ClusterInfo> {
        self.clusters.get(id)
    }

    /// Get all clusters
    pub fn list_clusters(&self) -> Vec<&ClusterInfo> {
        self.clusters.values().collect()
    }

    /// Get clusters by type
    pub fn list_clusters_by_type(&self, cluster_type: &ClusterType) -> Vec<&ClusterInfo> {
        self.clusters
            .values()
            .filter(|c| &c.cluster_type == cluster_type)
            .collect()
    }

    /// Get cluster count
    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }

    /// Check if a cluster exists
    pub fn has_cluster(&self, id: &str) -> bool {
        self.clusters.contains_key(id)
    }
}

impl Default for ClusterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_registry_new() {
        let registry = ClusterRegistry::new();
        assert_eq!(registry.cluster_count(), 0);
    }

    #[test]
    fn test_cluster_registry_add_and_get() {
        let mut registry = ClusterRegistry::new();

        let cluster = ClusterInfo {
            id: "cluster-1".to_string(),
            name: "Production".to_string(),
            cluster_type: ClusterType::VE,
            url: "https://pve.example.com".to_string(),
            port: 8006,
            username: "root@pam".to_string(),
            created_at: "2026-06-10 12:00:00".to_string(),
            updated_at: "2026-06-10 12:00:00".to_string(),
        };

        registry.add_cluster(cluster.clone());
        assert_eq!(registry.cluster_count(), 1);

        let retrieved = registry.get_cluster("cluster-1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Production");
    }

    #[test]
    fn test_cluster_registry_remove() {
        let mut registry = ClusterRegistry::new();

        let cluster = ClusterInfo {
            id: "cluster-1".to_string(),
            name: "Production".to_string(),
            cluster_type: ClusterType::VE,
            url: "https://pve.example.com".to_string(),
            port: 8006,
            username: "root@pam".to_string(),
            created_at: "2026-06-10 12:00:00".to_string(),
            updated_at: "2026-06-10 12:00:00".to_string(),
        };

        registry.add_cluster(cluster);
        assert_eq!(registry.cluster_count(), 1);

        let removed = registry.remove_cluster("cluster-1");
        assert!(removed.is_some());
        assert_eq!(registry.cluster_count(), 0);
    }

    #[test]
    fn test_cluster_registry_list_by_type() {
        let mut registry = ClusterRegistry::new();

        let ve_cluster = ClusterInfo {
            id: "ve-1".to_string(),
            name: "VE Cluster".to_string(),
            cluster_type: ClusterType::VE,
            url: "https://pve.example.com".to_string(),
            port: 8006,
            username: "root@pam".to_string(),
            created_at: "2026-06-10 12:00:00".to_string(),
            updated_at: "2026-06-10 12:00:00".to_string(),
        };

        let pbs_cluster = ClusterInfo {
            id: "pbs-1".to_string(),
            name: "PBS Cluster".to_string(),
            cluster_type: ClusterType::PBS,
            url: "https://pbs.example.com".to_string(),
            port: 8007,
            username: "root@pam".to_string(),
            created_at: "2026-06-10 12:00:00".to_string(),
            updated_at: "2026-06-10 12:00:00".to_string(),
        };

        registry.add_cluster(ve_cluster);
        registry.add_cluster(pbs_cluster);

        let ve_clusters = registry.list_clusters_by_type(&ClusterType::VE);
        assert_eq!(ve_clusters.len(), 1);

        let pbs_clusters = registry.list_clusters_by_type(&ClusterType::PBS);
        assert_eq!(pbs_clusters.len(), 1);
    }
}
