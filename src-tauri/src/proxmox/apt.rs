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

    let arr = match response.as_array() {
        Some(a) => a,
        None => return Ok(vec![]),
    };
    let updates: Vec<APTUpdate> = arr
        .iter()
        .filter_map(|update| {
            let package = update.get("Package")?.as_str()?.to_string();
            let version = update.get("Version")?.as_str()?.to_string();
            let available_version = update
                .get("OldVersion")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let size = update.get("Size").and_then(|s| s.as_u64()).unwrap_or(0);
            let release = update
                .get("Origin")
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
        .collect();

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
        .post_form(&path, &[], Some(ticket))
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
    let path = format!("nodes/{}/apt/repositories", node);
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list APT repositories: {}", e))?;

    // response IS already the data object (handle_response unwrapped the envelope)
    // GET /nodes/{node}/apt/repositories returns {"files": [...], "infos": [...], ...}
    let files = match response.get("files").and_then(|f| f.as_array()) {
        Some(f) => f,
        None => return Ok(vec![]),
    };
    let repo_list: Vec<APTRepository> = files
        .iter()
        .map(|file| {
            let uris = file.get("URIs").and_then(|u| u.as_array());
            let suites = file.get("Suites").and_then(|s| s.as_array());
            let components = file.get("Components").and_then(|c| c.as_array());
            let types = file.get("Types").and_then(|t| t.as_array());

            let url = uris
                .and_then(|u| u.first())
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .to_string();
            let distribution = suites
                .and_then(|s| s.first())
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let component = components
                .and_then(|c| c.first())
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string();
            let type_ = types
                .and_then(|t| t.first())
                .and_then(|t| t.as_str())
                .unwrap_or("deb")
                .to_string();
            let enabled = file
                .get("Enabled")
                .and_then(|e| e.as_bool())
                .unwrap_or(true);
            let repository_id = format!("{}-{}", type_, url);

            APTRepository {
                repository_id,
                url,
                distribution,
                component,
                enabled,
                type_,
            }
        })
        .collect();

    Ok(repo_list)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apt_update_struct_serialization() {
        let update = APTUpdate {
            package: "curl".to_string(),
            version: "7.88.1-10+deb12u8".to_string(),
            available_version: "7.88.1-10+deb12u7".to_string(),
            size: 1024,
            release: "Debian".to_string(),
        };
        let json = serde_json::to_string(&update).unwrap();
        let deserialized: APTUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(update.package, deserialized.package);
        assert_eq!(update.version, deserialized.version);
    }

    #[test]
    fn test_apt_update_pve_uses_capitalized_field_names() {
        // PVE API returns capitalized names: Package, Version, OldVersion, Origin
        let pve_response = serde_json::json!({
            "Package": "curl",
            "Version": "7.88.1-10+deb12u8",
            "OldVersion": "7.88.1-10+deb12u7",
            "Size": 1024,
            "Origin": "Debian"
        });
        let package = pve_response.get("Package").and_then(|p| p.as_str()).unwrap();
        let version = pve_response.get("Version").and_then(|v| v.as_str()).unwrap();
        let old_version = pve_response.get("OldVersion").and_then(|v| v.as_str()).unwrap();
        let origin = pve_response.get("Origin").and_then(|r| r.as_str()).unwrap();
        assert_eq!(package, "curl");
        assert_eq!(version, "7.88.1-10+deb12u8");
        assert_eq!(old_version, "7.88.1-10+deb12u7");
        assert_eq!(origin, "Debian");

        // Confirm lowercase fields don't exist in PVE response
        assert!(pve_response.get("package").is_none(), "PVE uses 'Package' not 'package'");
    }

    #[test]
    fn test_apt_repository_list_reads_files_array() {
        // PVE GET /nodes/{node}/apt/repositories returns {"files": [...], "infos": [...]}
        // The list function reads from the "files" key, not the top-level array
        let pve_response = serde_json::json!({
            "files": [
                {
                    "URIs": ["http://deb.debian.org/debian"],
                    "Suites": ["bookworm"],
                    "Components": ["main"],
                    "Types": ["deb"],
                    "Enabled": true
                }
            ],
            "infos": [],
            "standard-repos": []
        });
        let files = pve_response.get("files").and_then(|f| f.as_array()).unwrap();
        assert_eq!(files.len(), 1, "must read from 'files' key not top-level array");

        let first = &files[0];
        let url = first.get("URIs").and_then(|u| u.as_array())
            .and_then(|u| u.first())
            .and_then(|u| u.as_str())
            .unwrap();
        assert_eq!(url, "http://deb.debian.org/debian");
    }
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
