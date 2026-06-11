// Update management module
// Provides operations for managing Proxmox updates

use serde::{Deserialize, Serialize};

/// Update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub package: String,
    pub version: String,
    pub available_version: String,
    pub size: u64,
}

/// Update status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub checked_at: String,
    pub updates: Vec<UpdateInfo>,
    pub update_count: u32,
}

/// Check for updates
pub async fn check_updates(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<UpdateStatus, String> {
    let path = "nodes/self/update";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to check for updates: {}", e))?;

    let checked_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let updates: Vec<UpdateInfo> = response
        .get("data")
        .and_then(|d| d.as_array())
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|update| {
            let package = update.get("package")?.as_str()?.to_string();
            let version = update.get("version")?.as_str()?.to_string();
            let available_version = update.get("available")?.as_str().unwrap_or("").to_string();
            let size = update.get("size")?.as_u64().unwrap_or(0);

            Some(UpdateInfo {
                package,
                version,
                available_version,
                size,
            })
        })
        .collect();

    let update_count = updates.len() as u32;

    Ok(UpdateStatus {
        checked_at,
        updates,
        update_count,
    })
}

/// List available updates
pub async fn list_updates(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<UpdateInfo>, String> {
    let path = "nodes/self/update";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list updates: {}", e))?;

    let updates: Vec<UpdateInfo> = response
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|update| {
                    let package = update.get("package")?.as_str()?.to_string();
                    let version = update.get("version")?.as_str()?.to_string();
                    let available_version =
                        update.get("available")?.as_str().unwrap_or("").to_string();
                    let size = update.get("size")?.as_u64().unwrap_or(0);

                    Some(UpdateInfo {
                        package,
                        version,
                        available_version,
                        size,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(updates)
}

/// Get update status
pub async fn get_update_status(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<serde_json::Value, String> {
    let path = "nodes/self/update/status";
    client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get update status: {}", e))
}

/// Refresh update list
pub async fn refresh_updates(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<(), String> {
    let path = "nodes/self/update";
    let _response: serde_json::Value = client
        .post(path, &serde_json::json!({}), Some(ticket))
        .await
        .map_err(|e| format!("Failed to refresh updates: {}", e))?;
    Ok(())
}

/// Install updates
pub async fn install_updates(
    client: &crate::proxmox::client::ProxmoxClient,
    packages: &[&str],
    ticket: &str,
) -> Result<(), String> {
    let path = "nodes/self/update";
    let config = serde_json::json!({
        "packages": packages
    });

    let _response: serde_json::Value = client
        .post(path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to install updates: {}", e))?;
    Ok(())
}

/// Get update history
pub async fn get_update_history(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let path = "nodes/self/update/history";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to get update history: {}", e))?;

    if let Some(history) = response.get("data").and_then(|d| d.as_array()) {
        Ok(history.to_vec())
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_info_serialization() {
        let update = UpdateInfo {
            package: "proxmox-ve".to_string(),
            version: "7.4-15".to_string(),
            available_version: "7.4-16".to_string(),
            size: 50000000,
        };

        let json = serde_json::to_string(&update).unwrap();
        let deserialized: UpdateInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(update.package, deserialized.package);
        assert_eq!(update.available_version, deserialized.available_version);
    }

    #[test]
    fn test_update_status_serialization() {
        let status = UpdateStatus {
            checked_at: "2026-06-10 14:30:00".to_string(),
            updates: vec![],
            update_count: 0,
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: UpdateStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(status.checked_at, deserialized.checked_at);
    }
}
