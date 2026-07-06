// APT repository management module
// Provides operations for managing package updates and repositories

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// APT package update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APTUpdate {
    pub package: String,
    pub version: String,
    pub available_version: String,
    pub size: u64,
    pub release: String,
}

/// One repository entry from an APT sources file, shaped as the frontend
/// `AptRepository` interface expects: the raw PVE arrays, not a flattened
/// first-element-only view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APTRepository {
    pub types: Vec<String>,
    pub uris: Vec<String>,
    pub suites: Vec<String>,
    pub components: Vec<String>,
    pub enabled: bool,
    pub comment: Option<String>,
}

/// List APT updates
pub async fn list_apt_updates(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<APTUpdate>, String> {
    let path = format!("nodes/{node}/apt/update");
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list APT updates: {e}"))?;

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

/// API path used to refresh the APT package index on a node.
pub fn apt_refresh_path(node: &str) -> String {
    format!("nodes/{node}/apt/update")
}

/// Refresh the APT package index on a node (equivalent to `apt-get update`).
/// PVE runs this as a task; the returned string is the task UPID.
pub async fn refresh_apt_cache(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<String, String> {
    let response: serde_json::Value = client
        .post_form(&apt_refresh_path(node), &[], Some(ticket))
        .await
        .map_err(|e| format!("Failed to refresh APT cache: {e}"))?;
    Ok(response.as_str().unwrap_or_default().to_string())
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_repository_entry(entry: &Value) -> APTRepository {
    // PVE encodes Enabled as 1/0; older shapes use a boolean.
    let enabled = entry
        .get("Enabled")
        .map(|e| e.as_bool().unwrap_or_else(|| e.as_i64().unwrap_or(1) != 0))
        .unwrap_or(true);

    APTRepository {
        types: string_array(entry.get("Types")),
        uris: string_array(entry.get("URIs")),
        suites: string_array(entry.get("Suites")),
        components: string_array(entry.get("Components")),
        enabled,
        comment: entry
            .get("Comment")
            .and_then(|c| c.as_str())
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .map(str::to_string),
    }
}

/// Parse the (envelope-unwrapped) response of GET /nodes/{node}/apt/repositories.
/// Real PVE nests entries under `files[].repositories[]`; a flat `files[]` shape
/// with the repository fields directly on the file object is accepted as fallback.
pub fn parse_apt_repositories(response: &Value) -> Vec<APTRepository> {
    let files = match response.get("files").and_then(|f| f.as_array()) {
        Some(f) => f,
        None => return vec![],
    };

    files
        .iter()
        .flat_map(|file| {
            if let Some(repos) = file.get("repositories").and_then(|r| r.as_array()) {
                repos.iter().map(parse_repository_entry).collect::<Vec<_>>()
            } else if file.get("Types").is_some() || file.get("URIs").is_some() {
                vec![parse_repository_entry(file)]
            } else {
                vec![]
            }
        })
        .collect()
}

/// List APT repositories
pub async fn list_apt_repositories(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<Vec<APTRepository>, String> {
    let path = format!("nodes/{node}/apt/repositories");
    let response: serde_json::Value = client
        .get(&path, Some(ticket))
        .await
        .map_err(|e| format!("Failed to list APT repositories: {e}"))?;

    Ok(parse_apt_repositories(&response))
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
        let package = pve_response
            .get("Package")
            .and_then(|p| p.as_str())
            .unwrap();
        assert_eq!(package, "curl");
        assert!(
            pve_response.get("package").is_none(),
            "PVE uses 'Package' not 'package'"
        );
    }

    #[test]
    fn test_parse_apt_repositories_nested_repositories_shape() {
        // Real PVE: entries live under files[].repositories[]
        let response = serde_json::json!({
            "files": [
                {
                    "path": "/etc/apt/sources.list",
                    "file-type": "list",
                    "repositories": [
                        {
                            "Types": ["deb"],
                            "URIs": ["http://deb.debian.org/debian"],
                            "Suites": ["bookworm", "bookworm-updates"],
                            "Components": ["main", "contrib"],
                            "Enabled": 1,
                            "Comment": " main repo"
                        },
                        {
                            "Types": ["deb"],
                            "URIs": ["http://security.debian.org/debian-security"],
                            "Suites": ["bookworm-security"],
                            "Components": ["main"],
                            "Enabled": 0
                        }
                    ]
                }
            ],
            "infos": [],
            "standard-repos": []
        });

        let repos = parse_apt_repositories(&response);
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].types, vec!["deb"]);
        assert_eq!(repos[0].uris, vec!["http://deb.debian.org/debian"]);
        assert_eq!(repos[0].suites, vec!["bookworm", "bookworm-updates"]);
        assert_eq!(repos[0].components, vec!["main", "contrib"]);
        assert!(repos[0].enabled);
        assert_eq!(repos[0].comment.as_deref(), Some("main repo"));
        assert!(!repos[1].enabled);
        assert_eq!(repos[1].comment, None);
    }

    #[test]
    fn test_parse_apt_repositories_flat_files_shape() {
        // Fallback: fields directly on the file object
        let response = serde_json::json!({
            "files": [
                {
                    "Types": ["deb"],
                    "URIs": ["http://deb.debian.org/debian"],
                    "Suites": ["bookworm"],
                    "Components": ["main"],
                    "Enabled": true
                }
            ]
        });

        let repos = parse_apt_repositories(&response);
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].uris, vec!["http://deb.debian.org/debian"]);
        assert!(repos[0].enabled);
    }

    #[test]
    fn test_parse_apt_repositories_missing_arrays_default_empty() {
        // A malformed entry must never produce missing fields for the frontend
        let response = serde_json::json!({
            "files": [
                {
                    "repositories": [
                        { "Enabled": 1 }
                    ]
                }
            ]
        });

        let repos = parse_apt_repositories(&response);
        assert_eq!(repos.len(), 1);
        assert!(repos[0].types.is_empty());
        assert!(repos[0].uris.is_empty());
        assert!(repos[0].suites.is_empty());
        assert!(repos[0].components.is_empty());
    }

    #[test]
    fn test_parse_apt_repositories_no_files_key() {
        assert!(parse_apt_repositories(&serde_json::json!({})).is_empty());
        assert!(parse_apt_repositories(&serde_json::json!({"files": []})).is_empty());
    }

    #[test]
    fn test_apt_repository_serializes_array_fields() {
        // The frontend renders repo.types.join(' ') etc. — the JSON keys must be
        // lowercase array fields, never absent.
        let repo = APTRepository {
            types: vec!["deb".into()],
            uris: vec!["http://example.com".into()],
            suites: vec!["bookworm".into()],
            components: vec!["main".into()],
            enabled: true,
            comment: None,
        };
        let json: Value = serde_json::to_value(&repo).unwrap();
        assert!(json.get("types").unwrap().is_array());
        assert!(json.get("uris").unwrap().is_array());
        assert!(json.get("suites").unwrap().is_array());
        assert!(json.get("components").unwrap().is_array());
        assert_eq!(json.get("enabled").unwrap(), &Value::Bool(true));
    }

    #[test]
    fn test_apt_refresh_path() {
        assert_eq!(apt_refresh_path("vmhost1"), "nodes/vmhost1/apt/update");
    }
}
