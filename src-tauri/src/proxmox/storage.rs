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

/// List ISO images available in a storage (client-side filtered from storage content)
pub async fn list_storage_content_iso(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    storage: &str,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let path = format!("nodes/{}/storage/{}/content", node, storage);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list storage content for {}/{}: {}", node, storage, e))?;

    response
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter(|item| {
                    item.get("content")
                        .and_then(|c| c.as_str())
                        .map(|c| c == "iso")
                        .unwrap_or(false)
                })
                .cloned()
                .collect::<Vec<_>>()
        })
        .ok_or_else(|| "Invalid response format from storage content".to_string())
}

/// Upload an ISO file to a Proxmox storage pool.
/// Returns the task UPID string that can be polled for completion.
pub async fn upload_iso(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    storage: &str,
    filename: &str,
    file_bytes: Vec<u8>,
    ticket: &str,
) -> Result<String, String> {
    let path = format!("nodes/{}/storage/{}/upload", node, storage);

    let file_part = reqwest::multipart::Part::bytes(file_bytes)
        .file_name(filename.to_string())
        .mime_str("application/octet-stream")
        .map_err(|e| format!("Failed to build multipart part: {}", e))?;

    let form = reqwest::multipart::Form::new()
        .text("content", "iso")
        .text("filename", filename.to_string())
        .part("file", file_part);

    let task_id: String = client
        .post_multipart(&path, form, Some(ticket))
        .await
        .map_err(|e| format!("Failed to upload ISO to {}/{}: {}", node, storage, e))?;

    Ok(task_id)
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
