// Storage management module
// Provides operations for managing Proxmox storage

use serde::{Deserialize, Serialize};

/// Storage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub storage: String,
    pub node: String,
    pub type_: String,
    pub content: String,
    pub size: u64,
    pub used: u64,
    pub available: u64,
    pub status: String,
}

/// List all storages
pub async fn list_storages(
    _client: &crate::proxmox::client::ProxmoxClient,
    _node: &str,
    _ticket: &str,
) -> Result<Vec<StorageInfo>, String> {
    Err("Not implemented yet".to_string())
}

/// Get storage status
pub async fn get_storage_status(
    _client: &crate::proxmox::client::ProxmoxClient,
    _node: &str,
    _storage: &str,
    _ticket: &str,
) -> Result<StorageInfo, String> {
    Err("Not implemented yet".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_info_serialization() {
        let storage = StorageInfo {
            storage: "local".to_string(),
            node: "pve-node-1".to_string(),
            type_: "dir".to_string(),
            content: "images,backup,iso".to_string(),
            size: 1000000000000,
            used: 300000000000,
            available: 700000000000,
            status: "available".to_string(),
        };

        let json = serde_json::to_string(&storage).unwrap();
        let deserialized: StorageInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(storage.storage, deserialized.storage);
        assert_eq!(storage.status, "available");
    }
}
