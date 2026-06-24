// System Updates module
// Extends existing updates module with additional functionality

use serde::{Deserialize, Serialize};

/// Remote update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteUpdateInfo {
    pub remote: String,
    pub package: String,
    pub version: String,
    pub available_version: String,
    pub size: u64,
}

/// Update summary for multiple remotes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSummary {
    pub checked_at: String,
    pub remotes: Vec<RemoteUpdateInfo>,
    pub total_updates: u32,
}

/// List updates from all remotes
pub async fn list_updates_all_remotes(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<RemoteUpdateInfo>, String> {
    let path = "remotes/updates";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list updates from all remotes: {}", e))?;

    let arr = match response.as_array() {
        Some(a) => a,
        None => return Ok(vec![]),
    };
    let updates: Vec<RemoteUpdateInfo> = arr
        .iter()
        .filter_map(|update| {
            let remote = update.get("remote")?.as_str()?.to_string();
            let package = update.get("package")?.as_str()?.to_string();
            let version = update.get("version")?.as_str()?.to_string();
            let available_version = update
                .get("available")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let size = update.get("size").and_then(|s| s.as_u64()).unwrap_or(0);

            Some(RemoteUpdateInfo {
                remote,
                package,
                version,
                available_version,
                size,
            })
        })
        .collect();

    Ok(updates)
}

/// Refresh update list for all remotes
pub async fn refresh_updates_all(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<(), String> {
    let path = "remotes/updates";
    let _response: serde_json::Value = client
        .post_form(path, &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to refresh updates: {}", e))?;
    Ok(())
}

/// Install updates from remotes
pub async fn install_updates_remotes(
    client: &crate::proxmox::client::ProxmoxClient,
    packages: &[&str],
    ticket: &str,
) -> Result<(), String> {
    let path = "remotes/updates";
    let config = serde_json::json!({
        "packages": packages
    });

    let _response: serde_json::Value = client
        .post(path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to install updates: {}", e))?;
    Ok(())
}

/// Get update status for all remotes
pub async fn get_update_status_all(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<UpdateSummary, String> {
    let updates = list_updates_all_remotes(client, ticket).await?;

    let checked_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let total_updates = updates.len() as u32;

    Ok(UpdateSummary {
        checked_at,
        remotes: updates,
        total_updates,
    })
}

/// List PVE remote repositories
pub async fn list_pve_remotes(
    client: &crate::proxmox::client::ProxmoxClient,
    ticket: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let path = "pve/updates";
    let response: serde_json::Value = client
        .get(path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list PVE remotes: {}", e))?;

    // response IS already the data (handle_response already unwrapped the envelope)
    if let Some(arr) = response.as_array() {
        Ok(arr.to_vec())
    } else if response.is_null() {
        Ok(vec![])
    } else {
        Ok(vec![response])
    }
}

/// Check updates for specific remote
pub async fn check_remote_updates(
    client: &crate::proxmox::client::ProxmoxClient,
    remote: &str,
    ticket: &str,
) -> Result<Vec<RemoteUpdateInfo>, String> {
    let path = format!("pve/{}/updates", remote);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to check updates for remote {}: {}", remote, e))?;

    let arr = match response.as_array() {
        Some(a) => a,
        None => return Ok(vec![]),
    };
    let updates: Vec<RemoteUpdateInfo> = arr
        .iter()
        .filter_map(|update| {
            let package = update.get("package")?.as_str()?.to_string();
            let version = update.get("version")?.as_str()?.to_string();
            let available_version = update
                .get("available")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let size = update.get("size").and_then(|s| s.as_u64()).unwrap_or(0);

            Some(RemoteUpdateInfo {
                remote: remote.to_string(),
                package,
                version,
                available_version,
                size,
            })
        })
        .collect();

    Ok(updates)
}
