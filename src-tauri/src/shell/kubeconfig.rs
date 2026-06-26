// Kubeconfig Management
//
// This module handles:
// - Auto-detection of ~/.kube/config
// - Parsing kubeconfig YAML
// - Encrypted storage of kubeconfig files
// - Context switching

use crate::state::AppState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubeconfigContext {
    pub name: String,
    pub cluster_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubeconfigInfo {
    pub id: String,
    pub name: String,
    pub context: String,
    pub cluster_url: Option<String>,
    pub is_active: bool,
}

pub async fn auto_detect_kubeconfig(_state: &AppState) -> Result<(), String> {
    // TODO: Implement kubeconfig auto-detection
    // For now, return an error instead of panicking
    Err("Kubeconfig auto-detection not yet implemented".to_string())
}

/// Return the `current-context` value from a kubeconfig YAML, if set.
/// Uses simple line scanning so it stays consistent with `parse_kubeconfig_contexts`.
pub fn extract_current_context_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("current-context:") {
            let val = trimmed
                .trim_start_matches("current-context:")
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}

pub fn parse_kubeconfig_contexts(content: &str) -> Result<Vec<KubeconfigContext>, String> {
    // Parse YAML kubeconfig file
    // Simple string parsing to extract contexts and cluster URLs

    let mut contexts = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // First pass: find all contexts with their cluster names
    let mut in_contexts = false;
    let mut _current_context_name = String::new();
    let mut current_cluster_name = String::new();

    for line in &lines {
        let trimmed = line.trim();

        if trimmed == "contexts:" {
            in_contexts = true;
            continue;
        }

        if in_contexts {
            // Check if we've left the contexts section (hit another top-level key)
            if !line.starts_with(' ') && !trimmed.is_empty() && !trimmed.starts_with('-') {
                break;
            }

            // Context name (at the end of a context block)
            if trimmed.starts_with("name:") && !current_cluster_name.is_empty() {
                _current_context_name = trimmed.trim_start_matches("name:").trim().to_string();

                // Find cluster URL
                let cluster_url = find_cluster_url(&lines, &current_cluster_name);

                contexts.push(KubeconfigContext {
                    name: _current_context_name.clone(),
                    cluster_url,
                });

                // Reset for next context
                _current_context_name.clear();
                current_cluster_name.clear();
            }

            // Cluster reference (inside context block)
            if trimmed.starts_with("cluster:") {
                current_cluster_name = trimmed.trim_start_matches("cluster:").trim().to_string();
            }
        }
    }

    Ok(contexts)
}

fn find_cluster_url(lines: &[&str], cluster_name: &str) -> String {
    let mut in_clusters = false;
    let mut _current_cluster_name = String::new();
    let mut found_target_cluster = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed == "clusters:" {
            in_clusters = true;
            continue;
        }

        if in_clusters {
            // Check if we've left the clusters section
            if !line.starts_with(' ') && !trimmed.is_empty() && !trimmed.starts_with('-') {
                break;
            }

            // Found the name of a cluster
            if trimmed.starts_with("name:") {
                _current_cluster_name = trimmed.trim_start_matches("name:").trim().to_string();

                if _current_cluster_name == cluster_name {
                    found_target_cluster = true;
                }
                continue;
            }

            // Found server URL - check if it's for our target cluster
            if found_target_cluster && trimmed.starts_with("server:") {
                return trimmed.trim_start_matches("server:").trim().to_string();
            }

            // New cluster definition starts - reset
            if trimmed.starts_with("- cluster:") {
                found_target_cluster = false;
            }
        }
    }

    String::new()
}

pub async fn get_active_kubeconfig(_state: &AppState) -> Result<Option<String>, String> {
    // TODO: Implement active kubeconfig retrieval
    // For now, return an error instead of panicking
    Err("Active kubeconfig retrieval not yet implemented".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_kubeconfig_contexts() {
        let yaml = r#"
apiVersion: v1
kind: Config
clusters:
- cluster:
    server: https://kubernetes.default.svc
  name: default
contexts:
- context:
    cluster: default
    user: default
  name: default
current-context: default
users:
- name: default
  user:
    token: test-token
"#;

        let result = parse_kubeconfig_contexts(yaml);
        assert!(result.is_ok());
        let contexts = result.unwrap();
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].name, "default");
    }

    #[test]
    #[ignore] // Requires AppState setup
    fn test_encrypt_kubeconfig_content() {
        // TODO: Test kubeconfig encryption using existing auth::encrypt_token
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_get_active_kubeconfig() {
        // TODO: Test active kubeconfig retrieval
    }
}
