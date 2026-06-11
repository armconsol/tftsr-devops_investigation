// APT repository management module
// Provides operations for managing package updates and repositories

use serde::{Deserialize, Serialize};

/// APT package update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APTUpdate {
    pub package: String,
    pub version: String,
    pub available_version: String,
    pub size: u64,
    pub release: String,
}

/// APT repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APTRepository {
    pub repository_id: String,
    pub url: String,
    pub distribution: String,
    pub component: String,
    pub enabled: bool,
    pub type_: String,
}

/// List APT updates
pub async fn list_apt_updates(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<APTUpdate>, String> {
    let path = format!("nodes/{}/apt/update", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list APT updates: {}", e))?;

    let updates: Vec<APTUpdate> = response
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|update| {
                    let package = update.get("package")?.as_str()?.to_string();
                    let version = update.get("version")?.as_str()?.to_string();
                    let available_version = update
                        .get("available")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let size = update.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
                    let release = update
                        .get("release")
                        .and_then(|r| r.as_str())
                        .unwrap_or("")
                        .to_string();

                    Some(APTUpdate {
                        package,
                        version,
                        available_version,
                        size,
                        release,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(updates)
}

/// Update APT repositories
pub async fn update_apt_repos(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/apt/sources", node);
    let _response: serde_json::Value = client
        .post(&path, &serde_json::json!({}), Some(ticket))
        .await
        .map_err(|e| format!("Failed to update APT repositories: {}", e))?;
    Ok(())
}

/// List APT repositories
pub async fn list_apt_repositories(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<APTRepository>, String> {
    let path = format!("nodes/{}/apt/sources", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list APT repositories: {}", e))?;

    if let Some(repos) = response.get("data").and_then(|d| d.as_array()) {
        let repo_list: Vec<APTRepository> = repos
            .iter()
            .filter_map(|repo| {
                let id = repo.get("id")?.as_str()?.to_string();
                let url = repo.get("url")?.as_str().unwrap_or("").to_string();
                let distribution = repo
                    .get("distribution")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();
                let component = repo
                    .get("component")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let enabled = repo
                    .get("enabled")
                    .and_then(|e| e.as_bool())
                    .unwrap_or(true);
                let type_ = repo
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("deb")
                    .to_string();

                Some(APTRepository {
                    repository_id: id,
                    url,
                    distribution,
                    component,
                    enabled,
                    type_,
                })
            })
            .collect();

        Ok(repo_list)
    } else {
        Err("Invalid response format: missing 'data' field".to_string())
    }
}

/// Add APT repository
pub async fn add_apt_repository(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    repo: &APTRepository,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/apt/sources", node);
    let config = serde_json::json!({
        "id": repo.repository_id,
        "url": repo.url,
        "distribution": repo.distribution,
        "component": repo.component,
        "enabled": repo.enabled,
        "type": repo.type_
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to add APT repository {}: {}", repo.repository_id, e))?;
    Ok(())
}

/// Update APT repository
pub async fn update_apt_repository(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    repo_id: &str,
    repo: &APTRepository,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/apt/sources/{}", node, repo_id);
    let config = serde_json::json!({
        "url": repo.url,
        "distribution": repo.distribution,
        "component": repo.component,
        "enabled": repo.enabled,
        "type": repo.type_
    });

    let _response: serde_json::Value = client
        .put(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to update APT repository {}: {}", repo_id, e))?;
    Ok(())
}

/// Delete APT repository
pub async fn delete_apt_repository(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    repo_id: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/apt/sources/{}", node, repo_id);
    let _response: serde_json::Value = client
        .delete(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to delete APT repository {}: {}", repo_id, e))?;
    Ok(())
}

/// Install APT package
pub async fn install_apt_package(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    package: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/apt", node);
    let config = serde_json::json!({
        "packages": [package]
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to install APT package {}: {}", package, e))?;
    Ok(())
}

/// Upgrade APT packages
pub async fn upgrade_apt_packages(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<(), String> {
    let path = format!("nodes/{}/apt", node);
    let config = serde_json::json!({
        "dist_upgrade": true
    });

    let _response: serde_json::Value = client
        .post(&path, &config, Some(ticket))
        .await
        .map_err(|e| format!("Failed to upgrade APT packages: {}", e))?;
    Ok(())
}
