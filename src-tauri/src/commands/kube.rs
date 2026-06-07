use crate::kube::portforward::{PortForwardSession, PortForwardSessionConfig};
use crate::kube::ClusterClient;
use crate::shell::kubectl::locate_kubectl;
use crate::state::AppState;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::sync::Arc;
use tauri::State;
use tokio::process::Command;
use tracing::info;

// Regex pattern for Kubernetes resource names - cached for performance
lazy_static! {
    static ref NAME_PATTERN_REGEX: Regex = Regex::new(r"^[a-z0-9]([a-z0-9.-]*[a-z0-9])?$").unwrap();
}

struct TempFileCleanup(std::path::PathBuf);
impl Drop for TempFileCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub id: String,
    pub name: String,
    pub context: String,
    pub cluster_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardRequest {
    pub cluster_id: String,
    pub namespace: String,
    pub pod: String,
    pub container_port: u16,
    /// Optional: Local port to bind to. If 0, kubectl will allocate dynamically.
    #[serde(default)]
    pub local_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardResponse {
    pub id: String,
    pub cluster_id: String,
    pub namespace: String,
    pub pod: String,
    pub container_ports: Vec<u16>,
    pub local_ports: Vec<u16>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodInfo {
    pub name: String,
    pub status: String,
    pub ready: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConnectionStatus {
    pub status: ClusterConnectionState,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClusterConnectionState {
    Connected,
    Disconnected { error: String },
}

#[tauri::command]
pub async fn add_cluster(
    id: String,
    name: String,
    kubeconfig_content: String,
    state: State<'_, AppState>,
) -> Result<ClusterInfo, String> {
    if kubeconfig_content.trim().is_empty() {
        return Err("Kubeconfig content cannot be empty".to_string());
    }

    let context = extract_context(&kubeconfig_content)?;
    let server_url = extract_server_url(&kubeconfig_content)?;

    let kubeconfig_arc = Arc::new(kubeconfig_content.clone());
    let client = ClusterClient::new(
        id.clone(),
        name.clone(),
        context.clone(),
        server_url.clone(),
        kubeconfig_arc,
    );

    {
        let mut clusters = state.clusters.lock().await;
        clusters.insert(id.clone(), client);
    }

    Ok(ClusterInfo {
        id,
        name,
        context,
        cluster_url: server_url,
    })
}

fn extract_context(content: &str) -> Result<String, String> {
    let value: Value =
        serde_yaml::from_str(content).map_err(|e| format!("Invalid kubeconfig YAML: {}", e))?;

    let contexts = value
        .get("contexts")
        .and_then(|c| c.as_sequence())
        .ok_or("Missing 'contexts' field in kubeconfig")?;

    if contexts.is_empty() {
        return Err("No contexts found in kubeconfig".to_string());
    }

    let first_context = contexts[0].get("name").and_then(|n| n.as_str());
    first_context
        .map(|s| s.to_string())
        .ok_or_else(|| "Context name not found".to_string())
}

fn extract_server_url(content: &str) -> Result<String, String> {
    let value: Value =
        serde_yaml::from_str(content).map_err(|e| format!("Invalid kubeconfig YAML: {}", e))?;

    let clusters = value
        .get("clusters")
        .and_then(|c| c.as_sequence())
        .ok_or("Missing 'clusters' field in kubeconfig")?;

    if clusters.is_empty() {
        return Err("No clusters found in kubeconfig".to_string());
    }

    let cluster = &clusters[0];
    let server = cluster
        .get("cluster")
        .and_then(|c| c.get("server"))
        .and_then(|s| s.as_str());

    server
        .map(|s| s.to_string())
        .ok_or_else(|| "Server URL not found in cluster".to_string())
}

#[tauri::command]
pub async fn remove_cluster(id: String, state: State<'_, AppState>) -> Result<(), String> {
    // Check existence in memory BEFORE touching the DB
    let exists = {
        let clusters = state.clusters.lock().await;
        clusters.contains_key(&id)
    };
    if !exists {
        return Err(format!("Cluster {id} not found"));
    }

    // Safe to delete from DB now
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute("DELETE FROM clusters WHERE id = ?1", [&id])
            .map_err(|e| format!("Failed to delete cluster: {e}"))?;
    }

    let mut clusters = state.clusters.lock().await;
    clusters.remove(&id);

    // Cascade: close all port forwards for this cluster
    let mut port_forwards = state.port_forwards.lock().await;
    let session_ids_to_remove: Vec<String> = port_forwards
        .iter()
        .filter(|(_, session)| session.cluster_id == id)
        .map(|(id, _)| id.clone())
        .collect();

    for session_id in session_ids_to_remove {
        if let Some(mut session) = port_forwards.remove(&session_id) {
            session.close().await;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn list_clusters(state: State<'_, AppState>) -> Result<Vec<ClusterInfo>, String> {
    let clusters = state.clusters.lock().await;

    let cluster_list: Vec<ClusterInfo> = clusters
        .values()
        .map(|c| ClusterInfo {
            id: c.id.clone(),
            name: c.name.clone(),
            context: c.context.clone(),
            cluster_url: c.server_url.clone(),
        })
        .collect();

    Ok(cluster_list)
}

#[tauri::command]
pub async fn test_cluster_connection(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<ClusterConnectionStatus, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    // Write kubeconfig to temp file and ensure cleanup even on panic
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    // Run kubectl cluster-info
    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("cluster-info")
        .env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    let status = if output.status.success() {
        ClusterConnectionState::Connected
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        ClusterConnectionState::Disconnected {
            error: stderr.to_string(),
        }
    };

    Ok(ClusterConnectionStatus {
        status,
        context: context.clone(),
    })
}

#[tauri::command]
pub async fn discover_pods(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<PodInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    // Write kubeconfig to temp file and ensure cleanup even on panic
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-pods.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    // Run kubectl get pods with full JSON output
    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("pods")
        .arg("-n")
        .arg(&namespace)
        .arg("-o")
        .arg("json")
        .env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to list pods: {}", stderr));
    }

    // Parse actual JSON output to get real pod information
    let stdout = String::from_utf8_lossy(&output.stdout);
    let pods = parse_pods_json(&stdout)?;

    Ok(pods)
}


// Regex patterns for Kubernetes resource names
// Must match: ^[a-z0-9]([a-z0-9.-]*[a-z0-9])?$ (DNS subdomain name)
// Added max length check (253 chars) to prevent ReDoS attacks
const MAX_NAME_LENGTH: usize = 253;

/// Validates a Kubernetes resource name against DNS subdomain naming rules.
///
/// # Arguments
/// * `name` - The name to validate
/// * `field_name` - The field name for error messages
///
/// # Returns
/// * `Ok(())` if the name is valid
/// * `Err(String)` with an error message if the name is invalid
pub fn validate_resource_name(name: &str, field_name: &str) -> Result<(), String> {
    // Check max length to prevent ReDoS attacks
    if name.len() > MAX_NAME_LENGTH {
        return Err(format!(
            "{} '{}' exceeds maximum length of {} characters",
            field_name, name, MAX_NAME_LENGTH
        ));
    }

    // Reject names starting with hyphens or dots
    if name.starts_with('-') || name.starts_with('.') {
        return Err(format!(
            "{} '{}' cannot start with a hyphen or dot",
            field_name, name
        ));
    }

    // Reject names ending with hyphens or dots
    if name.ends_with('-') || name.ends_with('.') {
        return Err(format!(
            "{} '{}' cannot end with a hyphen or dot",
            field_name, name
        ));
    }

    // Use cached regex pattern
    if !NAME_PATTERN_REGEX.is_match(name) {
        return Err(format!(
            "{} '{}' does not match pattern {}",
            field_name, name, r"^[a-z0-9]([a-z0-9.-]*[a-z0-9])?$"
        ));
    }

    Ok(())
}

#[tauri::command]
pub async fn start_port_forward(
    request: PortForwardRequest,
    state: State<'_, AppState>,
) -> Result<PortForwardResponse, String> {
    let session_id = uuid::Uuid::now_v7().to_string();

    // Validate namespace and pod names FIRST to prevent command injection
    // Validation must happen before any operations to prevent partial state creation
    validate_resource_name(&request.namespace, "namespace")?;
    validate_resource_name(&request.pod, "pod")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&request.cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", request.cluster_id))?;

    let cluster_name = cluster.name.clone();
    let kubeconfig_content = cluster.kubeconfig_content.clone();

    // Use kubectl's dynamic port binding by specifying 0 as local port
    // This avoids race condition with port allocation
    // Note: Dynamic port allocation (when local_port=0) currently returns 0
    // The actual allocated port could be captured from kubectl's stderr/stdout
    // but this requires parsing kubectl output which is complex and error-prone
    // For now, users must specify a local port or use the default behavior
    let local_port = if request.local_port > 0 {
        request.local_port
    } else {
        0 // Let kubectl allocate dynamically (currently not captured)
    };

    info!(
        session_id = %session_id,
        cluster_id = %request.cluster_id,
        namespace = %request.namespace,
        pod = %request.pod,
        container_port = request.container_port,
        local_port,
        "Allocating local port for port-forward"
    );

    // Write kubeconfig to temp file
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}.yaml", request.cluster_id));

    std::fs::write(&temp_path, kubeconfig_content.as_ref())
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    // Build kubectl command
    let kubectl_path = locate_kubectl()?;
    let args = vec![
        "port-forward".to_string(),
        format!("pod/{}", request.pod),
        format!("{}:{}", local_port, request.container_port),
        "-n".to_string(),
        request.namespace.clone(),
    ];

    info!(
        session_id = %session_id,
        command = ?args,
        "Spawning kubectl port-forward subprocess"
    );

    // Spawn kubectl subprocess
    let child = Command::new(kubectl_path)
        .args(&args)
        .env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", &cluster.context)
        .spawn()
        .map_err(|e| format!("Failed to spawn kubectl: {e}"))?;

    // Create session with allocated port
    let session = PortForwardSession::new(PortForwardSessionConfig {
        id: session_id.clone(),
        cluster_id: request.cluster_id.clone(),
        cluster_name,
        namespace: request.namespace.clone(),
        pod: request.pod.clone(),
        container: None,
        ports: vec![request.container_port],
        local_ports: vec![local_port],
        temp_kubeconfig_path: Some(temp_path),
    });

    // Store child handle in session - spawn background task to wait on child
    {
        let mut port_forwards = state.port_forwards.lock().await;
        port_forwards.insert(session_id.clone(), session);
        let session_mut = port_forwards.get_mut(&session_id).unwrap();
        session_mut.spawn_child_waiter(child);
    }

    info!(
        session_id = %session_id,
        local_port,
        "Port-forward session started"
    );

    Ok(PortForwardResponse {
        id: session_id,
        cluster_id: request.cluster_id,
        namespace: request.namespace,
        pod: request.pod,
        container_ports: vec![request.container_port],
        local_ports: vec![local_port],
        status: "Active".to_string(),
    })
}

#[tauri::command]
pub async fn stop_port_forward(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut port_forwards = state.port_forwards.lock().await;

    if let Some(session) = port_forwards.get_mut(&id) {
        session.stop_async().await;
        info!(session_id = %id, "Port-forward session stopped");
        Ok(())
    } else {
        Err(format!("Port forward session {id} not found"))
    }
}

#[tauri::command]
pub async fn list_port_forwards(
    state: State<'_, AppState>,
) -> Result<Vec<PortForwardResponse>, String> {
    let port_forwards = state.port_forwards.lock().await;

    let mut forwards = Vec::new();
    for s in port_forwards.values() {
        let status_str = {
            let status = s.shared_status.lock().await;
            match &*status {
                crate::kube::PortForwardStatus::Active => "Active".to_string(),
                crate::kube::PortForwardStatus::Stopped => "Stopped".to_string(),
                crate::kube::PortForwardStatus::Error(e) => e.clone(),
            }
        };
        forwards.push(PortForwardResponse {
            id: s.id.clone(),
            cluster_id: s.cluster_id.clone(),
            namespace: s.namespace.clone(),
            pod: s.pod.clone(),
            container_ports: s.ports.clone(),
            local_ports: s.local_ports.clone(),
            status: status_str,
        });
    }

    Ok(forwards)
}

#[tauri::command]
pub async fn delete_port_forward(id: String, state: State<'_, AppState>) -> Result<(), String> {
    // Delete from database
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute("DELETE FROM port_forwards WHERE id = ?1", [&id])
            .map_err(|e| format!("Failed to delete port forward: {e}"))?;
    }

    let mut port_forwards = state.port_forwards.lock().await;

    if let Some(mut session) = port_forwards.remove(&id) {
        // Close the session to kill the child and clean up temp files
        session.close().await;
    } else {
        return Err(format!("Port forward session {id} not found"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_info_serialization() {
        let info = ClusterInfo {
            id: "cluster-1".to_string(),
            name: "Production".to_string(),
            context: "prod-context".to_string(),
            cluster_url: "https://k8s.example.com".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: ClusterInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.id, parsed.id);
        assert_eq!(info.name, parsed.name);
        assert_eq!(info.context, parsed.context);
        assert_eq!(info.cluster_url, parsed.cluster_url);
    }

    #[test]
    fn test_cluster_connection_state_serialization() {
        let connected = ClusterConnectionState::Connected;
        let json = serde_json::to_string(&connected).unwrap();
        let parsed: ClusterConnectionState = serde_json::from_str(&json).unwrap();

        assert!(matches!(parsed, ClusterConnectionState::Connected));

        let disconnected = ClusterConnectionState::Disconnected {
            error: "connection refused".to_string(),
        };
        let json = serde_json::to_string(&disconnected).unwrap();
        let parsed: ClusterConnectionState = serde_json::from_str(&json).unwrap();

        assert!(matches!(
            parsed,
            ClusterConnectionState::Disconnected { .. }
        ));
    }

    #[test]
    fn test_port_forward_request_serialization() {
        let request = PortForwardRequest {
            cluster_id: "cluster-1".to_string(),
            namespace: "default".to_string(),
            pod: "my-pod-abc123".to_string(),
            container_port: 8080,
            local_port: 0,
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: PortForwardRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.cluster_id, parsed.cluster_id);
        assert_eq!(request.namespace, parsed.namespace);
        assert_eq!(request.pod, parsed.pod);
        assert_eq!(request.container_port, parsed.container_port);
        assert_eq!(request.local_port, parsed.local_port);
    }

    #[test]
    fn test_validate_resource_name_valid() {
        // Valid names
        assert!(validate_resource_name("my-pod", "pod").is_ok());
        assert!(validate_resource_name("my-pod-123", "pod").is_ok());
        assert!(validate_resource_name("a", "pod").is_ok());
        assert!(validate_resource_name("my.pod.name", "pod").is_ok());
        assert!(validate_resource_name("123", "pod").is_ok());
    }

    #[test]
    fn test_validate_resource_name_invalid() {
        // Invalid names
        assert!(validate_resource_name("-mypod", "pod").is_err());
        assert!(validate_resource_name("mypod-", "pod").is_err());
        assert!(validate_resource_name(".mypod", "pod").is_err());
        assert!(validate_resource_name("mypod.", "pod").is_err());
        assert!(validate_resource_name("MYPOD", "pod").is_err());
        assert!(validate_resource_name("my_pod", "pod").is_err());
        assert!(validate_resource_name("", "pod").is_err());
    }

    #[test]
    fn test_validate_resource_name_length() {
        // Too long names
        let long_name = "a".repeat(254);
        assert!(validate_resource_name(&long_name, "pod").is_err());
    }
}

#[tauri::command]
pub async fn shutdown_port_forwards(state: State<'_, AppState>) -> Result<(), String> {
    let mut port_forwards = state.port_forwards.lock().await;

    // Close all active port forward sessions
    let session_ids: Vec<String> = port_forwards.keys().cloned().collect();

    for session_id in session_ids {
        if let Some(mut session) = port_forwards.remove(&session_id) {
            session.close().await;
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// New Resource Discovery Commands (Phase 1)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceInfo {
    pub name: String,
    pub status: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    pub name: Option<String>,
    pub port: u16,
    pub target_port: Option<String>,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub namespace: String,
    #[serde(rename = "type")]
    pub service_type: String,
    pub cluster_ip: String,
    pub external_ip: Option<String>,
    pub ports: Vec<ServicePort>,
    pub age: String,
    pub selector: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub name: String,
    pub namespace: String,
    pub ready: String,
    pub up_to_date: String,
    pub available: String,
    pub age: String,
    pub replicas: i32,
    pub labels: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatefulSetInfo {
    pub name: String,
    pub namespace: String,
    pub ready: String,
    pub age: String,
    pub replicas: i32,
    pub labels: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSetInfo {
    pub name: String,
    pub namespace: String,
    pub desired: i32,
    pub current: i32,
    pub ready: i32,
    pub up_to_date: i32,
    pub available: i32,
    pub age: String,
    pub labels: std::collections::HashMap<String, String>,
}

#[tauri::command]
pub async fn list_namespaces(cluster_id: String, state: State<'_, AppState>) -> Result<Vec<NamespaceInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-namespaces.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("namespaces")
        .arg("-o")
        .arg("json")
        .env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_namespaces_json(&output_str)
}

fn parse_namespaces_json(json_str: &str) -> Result<Vec<NamespaceInfo>, String> {
    let value: Value = serde_yaml::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_sequence())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut namespaces = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let status = item
            .get("status")
            .and_then(|s| s.get("phase"))
            .and_then(|p| p.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        namespaces.push(NamespaceInfo {
            name,
            status,
            age,
        });
    }

    Ok(namespaces)
}

#[tauri::command]
pub async fn list_pods(cluster_id: String, namespace: String, state: State<'_, AppState>) -> Result<Vec<PodInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-pods.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("pods")
        .arg("-n")
        .arg(&namespace)
        .arg("-o")
        .arg("json")
        .env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_pods_json(&output_str)
}

fn parse_pods_json(json_str: &str) -> Result<Vec<PodInfo>, String> {
    let value: Value = serde_yaml::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_sequence())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut pods = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let status = item
            .get("status")
            .and_then(|s| s.get("phase"))
            .and_then(|p| p.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let ready = item
            .get("status")
            .and_then(|s| s.get("containerStatuses"))
            .and_then(|c| c.as_sequence())
            .map(|containers| {
                let ready_count = containers.iter().filter(|c| {
                    c.get("ready").and_then(|r| r.as_bool()).unwrap_or(false)
                }).count();
                let total_count = containers.len();
                format!("{}/{}", ready_count, total_count)
            })
            .unwrap_or("0/0".to_string());

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        pods.push(PodInfo {
            name,
            status,
            ready,
            age,
        });
    }

    Ok(pods)
}

#[tauri::command]
pub async fn list_services(cluster_id: String, namespace: String, state: State<'_, AppState>) -> Result<Vec<ServiceInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-services.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("services")
        .arg("-n")
        .arg(&namespace)
        .arg("-o")
        .arg("json")
        .env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_services_json(&output_str)
}

fn parse_services_json(json_str: &str) -> Result<Vec<ServiceInfo>, String> {
    let value: Value = serde_yaml::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_sequence())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut services = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let namespace = item
            .get("metadata")
            .and_then(|m| m.get("namespace"))
            .and_then(|n| n.as_str())
            .unwrap_or("default")
            .to_string();

        let service_type = item
            .get("spec")
            .and_then(|s| s.get("type"))
            .and_then(|t| t.as_str())
            .unwrap_or("ClusterIP")
            .to_string();

        let cluster_ip = item
            .get("spec")
            .and_then(|s| s.get("clusterIP"))
            .and_then(|c| c.as_str())
            .unwrap_or("None")
            .to_string();

        let external_ip = item
            .get("status")
            .and_then(|s| s.get("loadBalancer"))
            .and_then(|l| l.get("ingress"))
            .and_then(|i| i.as_sequence())
            .and_then(|ingress| ingress.first())
            .and_then(|ing| ing.get("ip"))
            .and_then(|ip| ip.as_str())
            .map(|s| s.to_string());

        let ports = item
            .get("spec")
            .and_then(|s| s.get("ports"))
            .and_then(|p| p.as_sequence())
            .map(|ports_seq| {
                ports_seq.iter().map(|p| ServicePort {
                    name: p.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()),
                    port: p.get("port").and_then(|p| p.as_u64()).unwrap_or(0) as u16,
                    target_port: p.get("targetPort").and_then(|tp| tp.as_str()).map(|s| s.to_string()),
                    protocol: p.get("protocol").and_then(|p| p.as_str()).unwrap_or("TCP").to_string(),
                }).collect()
            })
            .unwrap_or_default();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        let selector = item
            .get("spec")
            .and_then(|s| s.get("selector"))
            .and_then(|s| s.as_mapping())
            .map(|s| {
                s.iter().filter_map(|(k, v)| {
                    k.as_str().map(|ks| (ks.to_string(), v.as_str().unwrap_or("").to_string()))
                }).collect()
            })
            .unwrap_or_default();

        services.push(ServiceInfo {
            name,
            namespace,
            service_type,
            cluster_ip,
            external_ip,
            ports,
            age,
            selector,
        });
    }

    Ok(services)
}

#[tauri::command]
pub async fn list_deployments(cluster_id: String, namespace: String, state: State<'_, AppState>) -> Result<Vec<DeploymentInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-deployments.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("deployments")
        .arg("-n")
        .arg(&namespace)
        .arg("-o")
        .arg("json")
        .env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_deployments_json(&output_str)
}

fn parse_deployments_json(json_str: &str) -> Result<Vec<DeploymentInfo>, String> {
    let value: Value = serde_yaml::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_sequence())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut deployments = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let namespace = item
            .get("metadata")
            .and_then(|m| m.get("namespace"))
            .and_then(|n| n.as_str())
            .unwrap_or("default")
            .to_string();

        let replicas = item
            .get("spec")
            .and_then(|s| s.get("replicas"))
            .and_then(|r| r.as_i64())
            .unwrap_or(0) as i32;

        let ready = item
            .get("status")
            .and_then(|s| s.get("readyReplicas"))
            .and_then(|r| r.as_i64())
            .map(|r| format!("{}/{}", r, replicas))
            .unwrap_or_else(|| format!("0/{}", replicas));

        let up_to_date = item
            .get("status")
            .and_then(|s| s.get("updatedReplicas"))
            .and_then(|r| r.as_i64())
            .map(|r| r.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let available = item
            .get("status")
            .and_then(|s| s.get("availableReplicas"))
            .and_then(|r| r.as_i64())
            .map(|r| r.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        let labels = item
            .get("metadata")
            .and_then(|m| m.get("labels"))
            .and_then(|l| l.as_mapping())
            .map(|l| {
                l.iter().filter_map(|(k, v)| {
                    k.as_str().map(|ks| (ks.to_string(), v.as_str().unwrap_or("").to_string()))
                }).collect()
            })
            .unwrap_or_default();

        deployments.push(DeploymentInfo {
            name,
            namespace,
            ready,
            up_to_date,
            available,
            age,
            replicas,
            labels,
        });
    }

    Ok(deployments)
}

#[tauri::command]
pub async fn list_statefulsets(cluster_id: String, namespace: String, state: State<'_, AppState>) -> Result<Vec<StatefulSetInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-statefulsets.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("statefulsets")
        .arg("-n")
        .arg(&namespace)
        .arg("-o")
        .arg("json")
        .env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_statefulsets_json(&output_str)
}

fn parse_statefulsets_json(json_str: &str) -> Result<Vec<StatefulSetInfo>, String> {
    let value: Value = serde_yaml::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_sequence())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut statefulsets = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let namespace = item
            .get("metadata")
            .and_then(|m| m.get("namespace"))
            .and_then(|n| n.as_str())
            .unwrap_or("default")
            .to_string();

        let replicas = item
            .get("spec")
            .and_then(|s| s.get("replicas"))
            .and_then(|r| r.as_i64())
            .unwrap_or(0) as i32;

        let ready = item
            .get("status")
            .and_then(|s| s.get("readyReplicas"))
            .and_then(|r| r.as_i64())
            .map(|r| format!("{}/{}", r, replicas))
            .unwrap_or_else(|| format!("0/{}", replicas));

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        let labels = item
            .get("metadata")
            .and_then(|m| m.get("labels"))
            .and_then(|l| l.as_mapping())
            .map(|l| {
                l.iter().filter_map(|(k, v)| {
                    k.as_str().map(|ks| (ks.to_string(), v.as_str().unwrap_or("").to_string()))
                }).collect()
            })
            .unwrap_or_default();

        statefulsets.push(StatefulSetInfo {
            name,
            namespace,
            ready,
            age,
            replicas,
            labels,
        });
    }

    Ok(statefulsets)
}

#[tauri::command]
pub async fn list_daemonsets(cluster_id: String, namespace: String, state: State<'_, AppState>) -> Result<Vec<DaemonSetInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-daemonsets.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("daemonsets")
        .arg("-n")
        .arg(&namespace)
        .arg("-o")
        .arg("json")
        .env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_daemonsets_json(&output_str)
}

fn parse_daemonsets_json(json_str: &str) -> Result<Vec<DaemonSetInfo>, String> {
    let value: Value = serde_yaml::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_sequence())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut daemonsets = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let namespace = item
            .get("metadata")
            .and_then(|m| m.get("namespace"))
            .and_then(|n| n.as_str())
            .unwrap_or("default")
            .to_string();

        let desired = item
            .get("status")
            .and_then(|s| s.get("desiredNumberScheduled"))
            .and_then(|r| r.as_i64())
            .unwrap_or(0) as i32;

        let current = item
            .get("status")
            .and_then(|s| s.get("currentNumberScheduled"))
            .and_then(|r| r.as_i64())
            .unwrap_or(0) as i32;

        let ready = item
            .get("status")
            .and_then(|s| s.get("numberReady"))
            .and_then(|r| r.as_i64())
            .unwrap_or(0) as i32;

        let up_to_date = item
            .get("status")
            .and_then(|s| s.get("updatedNumberScheduled"))
            .and_then(|r| r.as_i64())
            .unwrap_or(0) as i32;

        let available = item
            .get("status")
            .and_then(|s| s.get("numberAvailable"))
            .and_then(|r| r.as_i64())
            .unwrap_or(0) as i32;

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        let labels = item
            .get("metadata")
            .and_then(|m| m.get("labels"))
            .and_then(|l| l.as_mapping())
            .map(|l| {
                l.iter().filter_map(|(k, v)| {
                    k.as_str().map(|ks| (ks.to_string(), v.as_str().unwrap_or("").to_string()))
                }).collect()
            })
            .unwrap_or_default();

        daemonsets.push(DaemonSetInfo {
            name,
            namespace,
            desired,
            current,
            ready,
            up_to_date,
            available,
            age,
            labels,
        });
    }

    Ok(daemonsets)
}

fn parse_creation_timestamp(timestamp: &str) -> String {
    if timestamp.is_empty() || timestamp == "null" {
        return "N/A".to_string();
    }

    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(dt);

        if diff.num_days() > 0 {
            return format!("{}d", diff.num_days());
        }
        if diff.num_hours() > 0 {
            return format!("{}h", diff.num_hours());
        }
        if diff.num_minutes() > 0 {
            return format!("{}m", diff.num_minutes());
        }
        return format!("{}s", diff.num_seconds());
    }

    "N/A".to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Resource Management Commands (Phase 2)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_pod_logs(cluster_id: String, namespace: String, pod_name: String, container_name: String, state: State<'_, AppState>) -> Result<LogResponse, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-logs.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("logs")
        .arg(pod_name)
        .arg("-n")
        .arg(namespace)
        .arg("-c")
        .arg(container_name)
        .env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let logs = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(LogResponse { logs })
}

#[tauri::command]
pub async fn scale_deployment(cluster_id: String, namespace: String, deployment_name: String, replicas: i32, state: State<'_, AppState>) -> Result<(), String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-scale.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("scale")
        .arg("deployment")
        .arg(deployment_name)
        .arg("--replicas")
        .arg(replicas.to_string())
        .arg("-n")
        .arg(namespace)
        .env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn restart_deployment(cluster_id: String, namespace: String, deployment_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-restart.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("rollout")
        .arg("restart")
        .arg("deployment")
        .arg(deployment_name)
        .arg("-n")
        .arg(namespace)
        .env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_resource(cluster_id: String, resource_type: String, namespace: String, resource_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-delete.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("delete")
        .arg(resource_type)
        .arg(resource_name)
        .arg("-n")
        .arg(namespace)
        .env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn exec_pod(cluster_id: String, namespace: String, pod_name: String, container_name: Option<String>, command: String, state: State<'_, AppState>) -> Result<ExecResponse, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-exec.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut cmd = Command::new(kubectl_path);
    cmd.arg("exec")
        .arg(pod_name)
        .arg("-n")
        .arg(namespace)
        .arg("--")
        .arg("sh")
        .arg("-c")
        .arg(command);

    if let Some(container) = container_name {
        cmd.arg("-c").arg(container);
    }

    cmd.env("KUBECONFIG", temp_path.to_string_lossy().to_string())
        .env("KUBERNETES_CONTEXT", context);

    let output = cmd.output().await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(ExecResponse { stdout, stderr, exit_code: output.status.code() })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogResponse {
    pub logs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

