use crate::kube::portforward::{PortForwardSession, PortForwardSessionConfig};
use crate::kube::ClusterClient;
use crate::shell::kubectl::locate_kubectl;
use crate::state::AppState;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Stdio;
use std::sync::Arc;
use tauri::State;
use tokio::io::AsyncWriteExt;
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
    pub containers: Vec<String>,
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
    let value: serde_yaml::Value =
        serde_yaml::from_str(content).map_err(|e| format!("Invalid kubeconfig YAML: {}", e))?;

    // Prefer current-context — this is what kubectl uses by default and what the
    // user intends when they upload their kubeconfig. Falling back to contexts[0]
    // picks the wrong entry when the file has multiple contexts.
    if let Some(current) = value.get("current-context").and_then(|c| c.as_str()) {
        if !current.is_empty() {
            return Ok(current.to_string());
        }
    }

    // No current-context set — fall back to the first context in the list
    let contexts = value
        .get("contexts")
        .and_then(|c| c.as_sequence())
        .ok_or("Missing 'contexts' field in kubeconfig")?;

    if contexts.is_empty() {
        return Err("No contexts found in kubeconfig".to_string());
    }

    contexts[0]
        .get("name")
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Context name not found".to_string())
}

fn extract_server_url(content: &str) -> Result<String, String> {
    let value: serde_yaml::Value =
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

/// Load a stored kubeconfig into the in-memory cluster map so all kube commands can use it.
///
/// This bridges the kubeconfig_files table (encrypted storage) with the in-memory
/// state.clusters map that every kubernetes command requires.
#[tauri::command]
pub async fn connect_cluster_from_kubeconfig(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Read name and encrypted content from DB
    let (name, encrypted_content) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT name, encrypted_content FROM kubeconfig_files WHERE id = ?1",
            rusqlite::params![&id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|e| format!("Kubeconfig {id} not found in storage: {e}"))?
    };

    let content = crate::integrations::auth::decrypt_token(&encrypted_content)?;
    let context = extract_context(&content)?;
    let server_url = extract_server_url(&content).unwrap_or_default();

    let client = ClusterClient::new(id.clone(), name, context, server_url, Arc::new(content));

    let mut clusters = state.clusters.lock().await;
    clusters.insert(id, client);

    Ok(())
}

/// Diagnostic: test a kubeconfig's ability to reach the cluster.
///
/// Returns a human-readable summary including the context name, kubectl binary
/// path, exit code, and the full stdout/stderr from `kubectl cluster-info`.
/// This command is safe to call at any time — it writes a temp file, tests the
/// connection, then deletes the file regardless of the outcome.
#[tauri::command]
pub async fn test_kubectl_connection(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let (kubeconfig_content, context) = {
        let clusters = state.clusters.lock().await;
        let cluster = clusters.get(&cluster_id).ok_or_else(|| {
            format!(
                "Cluster {} not found in session — try re-selecting the cluster",
                cluster_id
            )
        })?;
        (cluster.kubeconfig_content.clone(), cluster.context.clone())
    };

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-diag.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content.as_ref())
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(&kubectl_path)
        .arg("cluster-info")
        .arg("--context")
        .arg(context.as_str())
        .arg("--kubeconfig")
        .arg(&temp_path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    Ok(format!(
        "Context:  {context}\nKubectl:  {kubectl}\nExit:     {exit}\n\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
        context = context,
        kubectl = kubectl_path.display(),
        exit = exit_code,
        stdout = if stdout.is_empty() { "(none)" } else { &stdout },
        stderr = if stderr.is_empty() { "(none)" } else { &stderr },
    ))
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
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
        .arg("--context")
        .arg(cluster.context.as_str())
        .arg("--kubeconfig")
        .arg(&temp_path)
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
pub async fn list_namespaces(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<NamespaceInfo>, String> {
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
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
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

        namespaces.push(NamespaceInfo { name, status, age });
    }

    Ok(namespaces)
}

#[tauri::command]
pub async fn list_pods(
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

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-pods.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("pods");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
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
            .and_then(|c| c.as_array())
            .map(|container_statuses| {
                let ready_count = container_statuses
                    .iter()
                    .filter(|c| c.get("ready").and_then(|r| r.as_bool()).unwrap_or(false))
                    .count();
                let total_count = container_statuses.len();
                format!("{}/{}", ready_count, total_count)
            })
            .unwrap_or("0/0".to_string());

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        let containers = item
            .get("spec")
            .and_then(|s| s.get("containers"))
            .and_then(|c| c.as_array())
            .map(|spec_containers| {
                spec_containers
                    .iter()
                    .filter_map(|c| {
                        c.get("name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();

        pods.push(PodInfo {
            name,
            status,
            ready,
            age,
            containers,
        });
    }

    Ok(pods)
}

#[tauri::command]
pub async fn list_services(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<ServiceInfo>, String> {
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

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("services");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
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
            .and_then(|i| i.as_array())
            .and_then(|ingress| ingress.first())
            .and_then(|ing| ing.get("ip"))
            .and_then(|ip| ip.as_str())
            .map(|s| s.to_string());

        let ports = item
            .get("spec")
            .and_then(|s| s.get("ports"))
            .and_then(|p| p.as_array())
            .map(|ports_seq| {
                ports_seq
                    .iter()
                    .map(|p| ServicePort {
                        name: p
                            .get("name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string()),
                        port: p.get("port").and_then(|p| p.as_u64()).unwrap_or(0) as u16,
                        target_port: p
                            .get("targetPort")
                            .and_then(|tp| tp.as_str())
                            .map(|s| s.to_string()),
                        protocol: p
                            .get("protocol")
                            .and_then(|p| p.as_str())
                            .unwrap_or("TCP")
                            .to_string(),
                    })
                    .collect()
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
            .and_then(|s| s.as_object())
            .map(|s| {
                s.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
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
pub async fn list_deployments(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<DeploymentInfo>, String> {
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

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("deployments");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
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
            .and_then(|l| l.as_object())
            .map(|l| {
                l.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
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
pub async fn list_statefulsets(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<StatefulSetInfo>, String> {
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

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("statefulsets");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
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
            .and_then(|l| l.as_object())
            .map(|l| {
                l.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
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
pub async fn list_daemonsets(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<DaemonSetInfo>, String> {
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

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("daemonsets");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
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
            .and_then(|l| l.as_object())
            .map(|l| {
                l.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
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
pub async fn get_pod_logs(
    cluster_id: String,
    namespace: String,
    pod_name: String,
    container_name: String,
    state: State<'_, AppState>,
) -> Result<LogResponse, String> {
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
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
pub async fn scale_deployment(
    cluster_id: String,
    namespace: String,
    deployment_name: String,
    replicas: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
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
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
pub async fn restart_deployment(
    cluster_id: String,
    namespace: String,
    deployment_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
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
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
pub async fn delete_resource(
    cluster_id: String,
    resource_type: String,
    namespace: String,
    resource_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
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
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
pub async fn exec_pod(
    cluster_id: String,
    namespace: String,
    pod_name: String,
    container_name: Option<String>,
    shell: Option<String>,
    command: String,
    state: State<'_, AppState>,
) -> Result<ExecResponse, String> {
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

    const ALLOWED_SHELLS: &[&str] = &[
        "sh",
        "bash",
        "ash",
        "dash",
        "/bin/sh",
        "/bin/bash",
        "/bin/ash",
        "/bin/dash",
    ];
    let shell_cmd = shell.as_deref().unwrap_or("sh");
    if !ALLOWED_SHELLS.contains(&shell_cmd) {
        return Err(format!(
            "Unsupported shell '{}'; allowed: sh, bash, ash, dash",
            shell_cmd
        ));
    }

    let mut cmd = Command::new(kubectl_path);
    cmd.arg("exec").arg(&pod_name).arg("-n").arg(&namespace);

    if let Some(ref container) = container_name {
        cmd.arg("-c").arg(container);
    }

    cmd.arg("--").arg(shell_cmd).arg("-c").arg(&command);

    cmd.arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str());

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(ExecResponse {
        stdout,
        stderr,
        exit_code: output.status.code(),
    })
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

// ─────────────────────────────────────────────────────────────────────────────
// Additional Resource Discovery Commands (Phase 3)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaSetInfo {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready: String,
    pub age: String,
    pub labels: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub name: String,
    pub namespace: String,
    pub completions: String,
    pub duration: String,
    pub age: String,
    pub labels: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobInfo {
    pub name: String,
    pub namespace: String,
    pub schedule: String,
    pub active: i32,
    pub last_schedule: String,
    pub age: String,
    pub labels: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapInfo {
    pub name: String,
    pub namespace: String,
    pub data_keys: i32,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretInfo {
    pub name: String,
    pub namespace: String,
    #[serde(rename = "type")]
    pub secret_type: String,
    pub data_keys: i32,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub status: String,
    pub roles: String,
    pub version: String,
    pub internal_ip: String,
    pub external_ip: Option<String>,
    pub os_image: String,
    pub kernel_version: String,
    pub kubelet_version: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub name: String,
    pub namespace: String,
    pub event_type: String,
    pub reason: String,
    pub object: String,
    pub count: i32,
    pub first_seen: String,
    pub last_seen: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressInfo {
    pub name: String,
    pub namespace: String,
    pub class: Option<String>,
    pub host: String,
    pub addresses: Vec<String>,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeClaimInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub volume: String,
    pub capacity: String,
    pub access_modes: Vec<String>,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeInfo {
    pub name: String,
    pub status: String,
    pub capacity: String,
    pub access_modes: Vec<String>,
    pub reclaim_policy: String,
    pub storage_class: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccountInfo {
    pub name: String,
    pub namespace: String,
    pub secrets: i32,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInfo {
    pub name: String,
    pub namespace: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRoleInfo {
    pub name: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleBindingInfo {
    pub name: String,
    pub namespace: String,
    pub role: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRoleBindingInfo {
    pub name: String,
    pub cluster_role: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizontalPodAutoscalerInfo {
    pub name: String,
    pub namespace: String,
    pub min_replicas: i32,
    pub max_replicas: i32,
    pub current_replicas: i32,
    pub desired_replicas: i32,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageClassInfo {
    pub name: String,
    pub provisioner: String,
    pub reclaim_policy: String,
    pub volume_binding_mode: String,
    pub allow_volume_expansion: bool,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyInfo {
    pub name: String,
    pub namespace: String,
    pub pod_selector: String,
    pub policy_types: Vec<String>,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuotaInfo {
    pub name: String,
    pub namespace: String,
    pub request_cpu: String,
    pub request_memory: String,
    pub limit_cpu: String,
    pub limit_memory: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitRangeInfo {
    pub name: String,
    pub namespace: String,
    pub limit_count: usize,
    pub age: String,
}

#[tauri::command]
pub async fn list_replicasets(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<ReplicaSetInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-replicasets.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("replicasets");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_replicasets_json(&output_str)
}

fn parse_replicasets_json(json_str: &str) -> Result<Vec<ReplicaSetInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut replicasets = Vec::new();
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
            .and_then(|l| l.as_object())
            .map(|l| {
                l.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
            })
            .unwrap_or_default();

        replicasets.push(ReplicaSetInfo {
            name,
            namespace,
            replicas,
            ready,
            age,
            labels,
        });
    }

    Ok(replicasets)
}

#[tauri::command]
pub async fn list_jobs(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<JobInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-jobs.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("jobs");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_jobs_json(&output_str)
}

fn parse_jobs_json(json_str: &str) -> Result<Vec<JobInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut jobs = Vec::new();
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

        let completions = item
            .get("status")
            .and_then(|s| s.get("succeeded"))
            .and_then(|s| s.as_i64())
            .map(|s| {
                let total = item
                    .get("spec")
                    .and_then(|sp| sp.get("completions"))
                    .and_then(|c| c.as_i64())
                    .unwrap_or(1);
                format!("{}/{}", s, total)
            })
            .unwrap_or_else(|| "0/0".to_string());

        let duration = item
            .get("status")
            .and_then(|s| s.get("startTime"))
            .and_then(|st| st.as_str())
            .and_then(|st| {
                let completion_time = item
                    .get("status")
                    .and_then(|s| s.get("completionTime"))
                    .and_then(|ct| ct.as_str());
                completion_time.or(Some(st))
            })
            .map(|st| {
                if let Ok(start) = chrono::DateTime::parse_from_rfc3339(st) {
                    let end_time = item
                        .get("status")
                        .and_then(|s| s.get("completionTime"))
                        .and_then(|ct| ct.as_str());
                    if let Some(end) = end_time {
                        if let Ok(end_dt) = chrono::DateTime::parse_from_rfc3339(end) {
                            let diff = end_dt.signed_duration_since(start);
                            if diff.num_minutes() > 0 {
                                return format!("{}m", diff.num_minutes());
                            }
                            return format!("{}s", diff.num_seconds());
                        }
                    }
                    let now = chrono::Utc::now();
                    let diff = now.signed_duration_since(start);
                    if diff.num_minutes() > 0 {
                        return format!("{}m", diff.num_minutes());
                    }
                    return format!("{}s", diff.num_seconds());
                }
                "N/A".to_string()
            })
            .unwrap_or("N/A".to_string());

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        let labels = item
            .get("metadata")
            .and_then(|m| m.get("labels"))
            .and_then(|l| l.as_object())
            .map(|l| {
                l.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
            })
            .unwrap_or_default();

        jobs.push(JobInfo {
            name,
            namespace,
            completions,
            duration,
            age,
            labels,
        });
    }

    Ok(jobs)
}

#[tauri::command]
pub async fn list_cronjobs(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<CronJobInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-cronjobs.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("cronjobs");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_cronjobs_json(&output_str)
}

fn parse_cronjobs_json(json_str: &str) -> Result<Vec<CronJobInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut cronjobs = Vec::new();
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

        let schedule = item
            .get("spec")
            .and_then(|s| s.get("schedule"))
            .and_then(|s| s.as_str())
            .unwrap_or("* * * * *")
            .to_string();

        let active = item
            .get("status")
            .and_then(|s| s.get("active"))
            .and_then(|a| a.as_array())
            .map(|a| a.len() as i32)
            .unwrap_or(0);

        let last_schedule = item
            .get("status")
            .and_then(|s| s.get("lastScheduleTime"))
            .and_then(|l| l.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        let labels = item
            .get("metadata")
            .and_then(|m| m.get("labels"))
            .and_then(|l| l.as_object())
            .map(|l| {
                l.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
            })
            .unwrap_or_default();

        cronjobs.push(CronJobInfo {
            name,
            namespace,
            schedule,
            active,
            last_schedule,
            age,
            labels,
        });
    }

    Ok(cronjobs)
}

#[tauri::command]
pub async fn list_configmaps(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<ConfigMapInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-configmaps.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("configmaps");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_configmaps_json(&output_str)
}

fn parse_configmaps_json(json_str: &str) -> Result<Vec<ConfigMapInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut configmaps = Vec::new();
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

        let data_keys = item
            .get("data")
            .and_then(|d| d.as_object())
            .map(|d| d.len() as i32)
            .unwrap_or(0);

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        configmaps.push(ConfigMapInfo {
            name,
            namespace,
            data_keys,
            age,
        });
    }

    Ok(configmaps)
}

#[tauri::command]
pub async fn list_secrets(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<SecretInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-secrets.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("secrets");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_secrets_json(&output_str)
}

fn parse_secrets_json(json_str: &str) -> Result<Vec<SecretInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut secrets = Vec::new();
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

        let secret_type = item
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("Opaque")
            .to_string();

        let data_keys = item
            .get("data")
            .and_then(|d| d.as_object())
            .map(|d| d.len() as i32)
            .unwrap_or(0);

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        secrets.push(SecretInfo {
            name,
            namespace,
            secret_type,
            data_keys,
            age,
        });
    }

    Ok(secrets)
}

#[tauri::command]
pub async fn list_nodes(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<NodeInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-nodes.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("nodes")
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_nodes_json(&output_str)
}

fn parse_nodes_json(json_str: &str) -> Result<Vec<NodeInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut nodes = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let status = item
            .get("status")
            .and_then(|s| s.get("conditions"))
            .and_then(|c| c.as_array())
            .and_then(|conditions| {
                conditions
                    .iter()
                    .find(|c| c.get("type").and_then(|t| t.as_str()) == Some("Ready"))
            })
            .and_then(|c| c.get("status").and_then(|s| s.as_str()))
            .map(|s| match s {
                "True" => "Ready",
                "False" => "NotReady",
                _ => "Unknown",
            })
            .unwrap_or("Unknown")
            .to_string();

        let roles = item
            .get("metadata")
            .and_then(|m| m.get("labels"))
            .and_then(|l| l.as_object())
            .map(|l| {
                let mut role_list: Vec<String> = Vec::new();
                if l.contains_key("node-role.kubernetes.io/control-plane")
                    || l.contains_key("node-role.kubernetes.io/master")
                {
                    role_list.push("control-plane".to_string());
                }
                if l.contains_key("node-role.kubernetes.io/worker") {
                    role_list.push("worker".to_string());
                }
                if l.contains_key("node-role.kubernetes.io/etcd") {
                    role_list.push("etcd".to_string());
                }
                if l.contains_key("node-role.kubernetes.io/ingress") {
                    role_list.push("ingress".to_string());
                }
                if role_list.is_empty() {
                    role_list.push("none".to_string());
                }
                role_list.join(",")
            })
            .unwrap_or("none".to_string());

        let version = item
            .get("status")
            .and_then(|s| s.get("nodeInfo"))
            .and_then(|n| n.get("kubeletVersion"))
            .and_then(|v| v.as_str())
            .unwrap_or("N/A")
            .to_string();

        let internal_ip = item
            .get("status")
            .and_then(|s| s.get("addresses"))
            .and_then(|a| a.as_array())
            .and_then(|addresses| {
                addresses
                    .iter()
                    .find(|addr| addr.get("type").and_then(|t| t.as_str()) == Some("InternalIP"))
            })
            .and_then(|addr| addr.get("address").and_then(|a| a.as_str()))
            .unwrap_or("N/A")
            .to_string();

        let external_ip = item
            .get("status")
            .and_then(|s| s.get("addresses"))
            .and_then(|a| a.as_array())
            .and_then(|addresses| {
                addresses
                    .iter()
                    .find(|addr| addr.get("type").and_then(|t| t.as_str()) == Some("ExternalIP"))
            })
            .and_then(|addr| addr.get("address").and_then(|a| a.as_str()))
            .map(|s| s.to_string());

        let os_image = item
            .get("status")
            .and_then(|s| s.get("nodeInfo"))
            .and_then(|n| n.get("osImage"))
            .and_then(|o| o.as_str())
            .unwrap_or("N/A")
            .to_string();

        let kernel_version = item
            .get("status")
            .and_then(|s| s.get("nodeInfo"))
            .and_then(|n| n.get("kernelVersion"))
            .and_then(|k| k.as_str())
            .unwrap_or("N/A")
            .to_string();

        let kubelet_version = item
            .get("status")
            .and_then(|s| s.get("nodeInfo"))
            .and_then(|n| n.get("kubeletVersion"))
            .and_then(|k| k.as_str())
            .unwrap_or("N/A")
            .to_string();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        nodes.push(NodeInfo {
            name,
            status,
            roles,
            version,
            internal_ip,
            external_ip,
            os_image,
            kernel_version,
            kubelet_version,
            age,
        });
    }

    Ok(nodes)
}

#[tauri::command]
pub async fn list_events(
    cluster_id: String,
    namespace: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<EventInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-events.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("events");
    if let Some(ns) = &namespace {
        kubectl_cmd.arg("-n").arg(ns);
    } else {
        kubectl_cmd.arg("--all-namespaces");
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_events_json(&output_str)
}

fn parse_events_json(json_str: &str) -> Result<Vec<EventInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut events = Vec::new();
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

        let event_type = item
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("Normal")
            .to_string();

        let reason = item
            .get("reason")
            .and_then(|r| r.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let object = item
            .get("involvedObject")
            .and_then(|o| o.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let count = item.get("count").and_then(|c| c.as_i64()).unwrap_or(1) as i32;

        let first_seen = item
            .get("firstTimestamp")
            .and_then(|f| f.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        let last_seen = item
            .get("lastTimestamp")
            .and_then(|l| l.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        let message = item
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .to_string();

        events.push(EventInfo {
            name,
            namespace,
            event_type,
            reason,
            object,
            count,
            first_seen,
            last_seen,
            message,
        });
    }

    Ok(events)
}

#[tauri::command]
pub async fn list_ingresses(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<IngressInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-ingresses.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("ingresses");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_ingresses_json(&output_str)
}

fn parse_ingresses_json(json_str: &str) -> Result<Vec<IngressInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut ingresses = Vec::new();
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

        let class = item
            .get("spec")
            .and_then(|s| s.get("ingressClassName"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());

        let host = item
            .get("spec")
            .and_then(|s| s.get("rules"))
            .and_then(|r| r.as_array())
            .and_then(|rules| rules.first())
            .and_then(|rule| rule.get("host").and_then(|h| h.as_str()))
            .unwrap_or("")
            .to_string();

        let addresses = item
            .get("status")
            .and_then(|s| s.get("loadBalancer"))
            .and_then(|l| l.get("ingress"))
            .and_then(|i| i.as_array())
            .map(|ingress| {
                ingress
                    .iter()
                    .filter_map(|ing| {
                        ing.get("ip")
                            .and_then(|ip| ip.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        ingresses.push(IngressInfo {
            name,
            namespace,
            class,
            host,
            addresses,
            age,
        });
    }

    Ok(ingresses)
}

#[tauri::command]
pub async fn list_persistentvolumeclaims(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<PersistentVolumeClaimInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-pvcs.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("persistentvolumeclaims");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_pvcs_json(&output_str)
}

fn parse_pvcs_json(json_str: &str) -> Result<Vec<PersistentVolumeClaimInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut pvcs = Vec::new();
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

        let status = item
            .get("status")
            .and_then(|s| s.get("phase"))
            .and_then(|p| p.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let volume = item
            .get("spec")
            .and_then(|s| s.get("volumeName"))
            .and_then(|v| v.as_str())
            .unwrap_or("N/A")
            .to_string();

        let capacity = item
            .get("status")
            .and_then(|s| s.get("capacity"))
            .and_then(|c| c.as_object())
            .map(|c| {
                let storage = c.get("storage").and_then(|s| s.as_str()).unwrap_or("N/A");
                storage.to_string()
            })
            .unwrap_or("N/A".to_string());

        let access_modes = item
            .get("spec")
            .and_then(|s| s.get("accessModes"))
            .and_then(|a| a.as_array())
            .map(|modes| {
                modes
                    .iter()
                    .filter_map(|m| m.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        pvcs.push(PersistentVolumeClaimInfo {
            name,
            namespace,
            status,
            volume,
            capacity,
            access_modes,
            age,
        });
    }

    Ok(pvcs)
}

#[tauri::command]
pub async fn list_persistentvolumes(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<PersistentVolumeInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-pvs.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("persistentvolumes")
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_pvs_json(&output_str)
}

fn parse_pvs_json(json_str: &str) -> Result<Vec<PersistentVolumeInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut pvs = Vec::new();
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

        let capacity = item
            .get("spec")
            .and_then(|s| s.get("capacity"))
            .and_then(|c| c.as_object())
            .map(|c| {
                let storage = c.get("storage").and_then(|s| s.as_str()).unwrap_or("N/A");
                storage.to_string()
            })
            .unwrap_or("N/A".to_string());

        let access_modes = item
            .get("spec")
            .and_then(|s| s.get("accessModes"))
            .and_then(|a| a.as_array())
            .map(|modes| {
                modes
                    .iter()
                    .filter_map(|m| m.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let reclaim_policy = item
            .get("spec")
            .and_then(|s| s.get("persistentVolumeReclaimPolicy"))
            .and_then(|r| r.as_str())
            .unwrap_or("Retain")
            .to_string();

        let storage_class = item
            .get("spec")
            .and_then(|s| s.get("storageClassName"))
            .and_then(|s| s.as_str())
            .unwrap_or("N/A")
            .to_string();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        pvs.push(PersistentVolumeInfo {
            name,
            status,
            capacity,
            access_modes,
            reclaim_policy,
            storage_class,
            age,
        });
    }

    Ok(pvs)
}

#[tauri::command]
pub async fn list_serviceaccounts(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<ServiceAccountInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-sas.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("serviceaccounts");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_serviceaccounts_json(&output_str)
}

fn parse_serviceaccounts_json(json_str: &str) -> Result<Vec<ServiceAccountInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut serviceaccounts = Vec::new();
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

        let secrets = item
            .get("secrets")
            .and_then(|s| s.as_array())
            .map(|s| s.len() as i32)
            .unwrap_or(0);

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        serviceaccounts.push(ServiceAccountInfo {
            name,
            namespace,
            secrets,
            age,
        });
    }

    Ok(serviceaccounts)
}

#[tauri::command]
pub async fn list_roles(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<RoleInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-roles.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("roles");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_roles_json(&output_str)
}

fn parse_roles_json(json_str: &str) -> Result<Vec<RoleInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut roles = Vec::new();
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

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        roles.push(RoleInfo {
            name,
            namespace,
            age,
        });
    }

    Ok(roles)
}

#[tauri::command]
pub async fn list_clusterroles(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ClusterRoleInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-clusterroles.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("clusterroles")
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_clusterroles_json(&output_str)
}

fn parse_clusterroles_json(json_str: &str) -> Result<Vec<ClusterRoleInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut clusterroles = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        clusterroles.push(ClusterRoleInfo { name, age });
    }

    Ok(clusterroles)
}

#[tauri::command]
pub async fn list_rolebindings(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<RoleBindingInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-rolebindings.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("rolebindings");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_rolebindings_json(&output_str)
}

fn parse_rolebindings_json(json_str: &str) -> Result<Vec<RoleBindingInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut rolebindings = Vec::new();
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

        let role = item
            .get("roleRef")
            .and_then(|r| r.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        rolebindings.push(RoleBindingInfo {
            name,
            namespace,
            role,
            age,
        });
    }

    Ok(rolebindings)
}

#[tauri::command]
pub async fn list_clusterrolebindings(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ClusterRoleBindingInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!(
        "kubeconfig-{}-clusterrolebindings.yaml",
        cluster_id
    ));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("clusterrolebindings")
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_clusterrolebindings_json(&output_str)
}

fn parse_clusterrolebindings_json(json_str: &str) -> Result<Vec<ClusterRoleBindingInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut clusterrolebindings = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let cluster_role = item
            .get("roleRef")
            .and_then(|r| r.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        clusterrolebindings.push(ClusterRoleBindingInfo {
            name,
            cluster_role,
            age,
        });
    }

    Ok(clusterrolebindings)
}

#[tauri::command]
pub async fn list_horizontalpodautoscalers(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<HorizontalPodAutoscalerInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-hpas.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("horizontalpodautoscalers");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_hpas_json(&output_str)
}

fn parse_hpas_json(json_str: &str) -> Result<Vec<HorizontalPodAutoscalerInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut hpas = Vec::new();
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

        let min_replicas = item
            .get("spec")
            .and_then(|s| s.get("minReplicas"))
            .and_then(|r| r.as_i64())
            .unwrap_or(1) as i32;

        let max_replicas = item
            .get("spec")
            .and_then(|s| s.get("maxReplicas"))
            .and_then(|r| r.as_i64())
            .unwrap_or(1) as i32;

        let current_replicas = item
            .get("status")
            .and_then(|s| s.get("currentReplicas"))
            .and_then(|r| r.as_i64())
            .unwrap_or(0) as i32;

        let desired_replicas = item
            .get("status")
            .and_then(|s| s.get("desiredReplicas"))
            .and_then(|r| r.as_i64())
            .unwrap_or(0) as i32;

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        hpas.push(HorizontalPodAutoscalerInfo {
            name,
            namespace,
            min_replicas,
            max_replicas,
            current_replicas,
            desired_replicas,
            age,
        });
    }

    Ok(hpas)
}

#[tauri::command]
pub async fn list_storageclasses(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<StorageClassInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-storageclasses.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("storageclasses")
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_storageclasses_json(&output_str)
}

fn parse_storageclasses_json(json_str: &str) -> Result<Vec<StorageClassInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut storageclasses = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let provisioner = item
            .get("provisioner")
            .and_then(|p| p.as_str())
            .unwrap_or("unknown")
            .to_string();

        let reclaim_policy = item
            .get("reclaimPolicy")
            .and_then(|r| r.as_str())
            .unwrap_or("Delete")
            .to_string();

        let volume_binding_mode = item
            .get("volumeBindingMode")
            .and_then(|v| v.as_str())
            .unwrap_or("Immediate")
            .to_string();

        let allow_volume_expansion = item
            .get("allowVolumeExpansion")
            .and_then(|a| a.as_bool())
            .unwrap_or(false);

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        storageclasses.push(StorageClassInfo {
            name,
            provisioner,
            reclaim_policy,
            volume_binding_mode,
            allow_volume_expansion,
            age,
        });
    }

    Ok(storageclasses)
}

#[tauri::command]
pub async fn list_networkpolicies(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<NetworkPolicyInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-networkpolicies.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("networkpolicies");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_networkpolicies_json(&output_str)
}

fn parse_networkpolicies_json(json_str: &str) -> Result<Vec<NetworkPolicyInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut networkpolicies = Vec::new();
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

        let pod_selector = item
            .get("spec")
            .and_then(|s| s.get("podSelector"))
            .map(|ps| serde_json::to_string(ps).unwrap_or_default())
            .unwrap_or_default();

        let policy_types = item
            .get("spec")
            .and_then(|s| s.get("policyTypes"))
            .and_then(|pt| pt.as_array())
            .map(|types| {
                types
                    .iter()
                    .filter_map(|t| t.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        networkpolicies.push(NetworkPolicyInfo {
            name,
            namespace,
            pod_selector,
            policy_types,
            age,
        });
    }

    Ok(networkpolicies)
}

#[tauri::command]
pub async fn list_resourcequotas(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<ResourceQuotaInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-resourcequotas.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("resourcequotas");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_resourcequotas_json(&output_str)
}

fn parse_resourcequotas_json(json_str: &str) -> Result<Vec<ResourceQuotaInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut resourcequotas = Vec::new();
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

        let hard = item.get("status").and_then(|s| s.get("hard"));

        let request_cpu = hard
            .and_then(|h| h.get("requests.cpu"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let request_memory = hard
            .and_then(|h| h.get("requests.memory"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let limit_cpu = hard
            .and_then(|h| h.get("limits.cpu"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let limit_memory = hard
            .and_then(|h| h.get("limits.memory"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        resourcequotas.push(ResourceQuotaInfo {
            name,
            namespace,
            request_cpu,
            request_memory,
            limit_cpu,
            limit_memory,
            age,
        });
    }

    Ok(resourcequotas)
}

#[tauri::command]
pub async fn list_limitranges(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<LimitRangeInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-limitranges.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("limitranges");
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }
    let output = kubectl_cmd
        .arg("-o")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_limitranges_json(&output_str)
}

fn parse_limitranges_json(json_str: &str) -> Result<Vec<LimitRangeInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut limitranges = Vec::new();
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

        let limit_count = item
            .get("spec")
            .and_then(|s| s.get("limits"))
            .and_then(|l| l.as_array())
            .map(|l| l.len())
            .unwrap_or(0);

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        limitranges.push(LimitRangeInfo {
            name,
            namespace,
            limit_count,
            age,
        });
    }

    Ok(limitranges)
}

#[tauri::command]
pub async fn cordon_node(
    cluster_id: String,
    node_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-cordon.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("cordon")
        .arg(node_name)
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
pub async fn uncordon_node(
    cluster_id: String,
    node_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-uncordon.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("uncordon")
        .arg(node_name)
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
pub async fn drain_node(
    cluster_id: String,
    node_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-drain.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("drain")
        .arg(node_name)
        .arg("--ignore-daemonsets")
        .arg("--delete-emptydir-data")
        .arg("--force")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
pub async fn rollback_deployment(
    cluster_id: String,
    namespace: String,
    deployment_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-rollback.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("rollout")
        .arg("undo")
        .arg("deployment")
        .arg(deployment_name)
        .arg("-n")
        .arg(namespace)
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
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
pub async fn create_resource(
    cluster_id: String,
    namespace: String,
    _resource_type: String,
    yaml_content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-create.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut cmd = Command::new(kubectl_path);
    cmd.arg("create")
        .arg("-f")
        .arg("-")
        .arg("-n")
        .arg(namespace)
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn kubectl: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(yaml_content.as_bytes())
            .await
            .map_err(|e| format!("Failed to write yaml to stdin: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn edit_resource(
    cluster_id: String,
    namespace: String,
    _resource_type: String,
    _resource_name: String,
    yaml_content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("kubeconfig-{}-edit.yaml", cluster_id));
    let _cleanup = TempFileCleanup(temp_path.clone());

    std::fs::write(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut cmd = Command::new(kubectl_path);
    cmd.arg("apply")
        .arg("-f")
        .arg("-")
        .arg("-n")
        .arg(namespace)
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn kubectl: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(yaml_content.as_bytes())
            .await
            .map_err(|e| format!("Failed to write yaml to stdin: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn subscribe_to_k8s_events(
    cluster_id: String,
    namespace: String,
    resource_type: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let _app_state = state.inner();

    let rx = crate::kube::start_resource_watcher(_app_state, cluster_id, namespace, resource_type)
        .await
        .map_err(|e| format!("Failed to start watcher: {e}"))?;

    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get duration: {e}"))?;
    let unsubscribe_id = format!("watcher-{}", duration.as_millis());

    state
        .inner()
        .watchers
        .lock()
        .unwrap()
        .insert(unsubscribe_id.clone(), rx);

    Ok(unsubscribe_id)
}

#[tauri::command]
pub async fn subscribe_to_all_k8s_events(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let _app_state = state.inner();

    let rx = crate::kube::start_all_resources_watcher(_app_state, cluster_id)
        .await
        .map_err(|e| format!("Failed to start all watcher: {e}"))?;

    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get duration: {e}"))?;
    let unsubscribe_id = format!("watcher-all-{}", duration.as_millis());

    state
        .inner()
        .watchers
        .lock()
        .unwrap()
        .insert(unsubscribe_id.clone(), rx);

    Ok(unsubscribe_id)
}

#[tauri::command]
pub async fn unsubscribe_from_k8s_events(
    unsubscribe_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let removed = state
        .inner()
        .watchers
        .lock()
        .unwrap()
        .remove(&unsubscribe_id);

    if removed.is_none() {
        return Err(format!("Watcher {} not found", unsubscribe_id));
    }

    Ok(())
}
