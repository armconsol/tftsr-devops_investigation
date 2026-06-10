use crate::kube::portforward::{PortForwardSession, PortForwardSessionConfig};
use crate::kube::ClusterClient;
use crate::shell::helm::locate_helm;
use crate::shell::kubectl::locate_kubectl;
use crate::state::AppState;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::info;

// Regex pattern for Kubernetes resource names - cached for performance
lazy_static! {
    static ref NAME_PATTERN_REGEX: Regex = Regex::new(r"^[a-z0-9]([a-z0-9.-]*[a-z0-9])?$").unwrap();
}

static KUBECONFIG_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn unique_kubeconfig_path(cluster_id: impl AsRef<str>) -> std::path::PathBuf {
    let n = KUBECONFIG_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!("kubeconfig-{}-{}.yaml", cluster_id.as_ref(), n))
}

struct TempFileCleanup(std::path::PathBuf);
impl Drop for TempFileCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Write kubeconfig content to a temp file with owner-only permissions (0600 on Unix).
/// Kubeconfig files contain cluster credentials and must never be world-readable.
fn write_secure_temp_file(path: &std::path::Path, content: &str) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .map_err(|e| format!("Failed to create kubeconfig temp file: {e}"))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))
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
    pub namespace: String,
    pub status: String,
    pub ready: String,
    pub age: String,
    pub containers: Vec<String>,
    pub restarts: Option<u32>,
    pub ip: Option<String>,
    pub node: Option<String>,
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

/// Detect the authentication method used by a kubeconfig for a given context.
///
/// Returns a human-readable string describing the auth type and any relevant
/// warnings (e.g. exec plugin binary name, file-path cert references).
fn detect_auth_method(kubeconfig: &str, context_name: &str) -> String {
    let yaml: serde_yaml::Value = match serde_yaml::from_str(kubeconfig) {
        Ok(v) => v,
        Err(_) => return "unknown (YAML parse error)".to_string(),
    };

    // Resolve the user name for this context.
    let user_name = yaml
        .get("contexts")
        .and_then(|c| c.as_sequence())
        .and_then(|contexts| {
            contexts
                .iter()
                .find(|ctx| ctx.get("name").and_then(|n| n.as_str()) == Some(context_name))
        })
        .and_then(|ctx| ctx.get("context"))
        .and_then(|c| c.get("user"))
        .and_then(|u| u.as_str())
        .unwrap_or(context_name)
        .to_string();

    let user_entry = yaml
        .get("users")
        .and_then(|u| u.as_sequence())
        .and_then(|users| {
            users
                .iter()
                .find(|u| u.get("name").and_then(|n| n.as_str()) == Some(user_name.as_str()))
        })
        .and_then(|u| u.get("user"));

    let Some(user) = user_entry else {
        return format!("unknown (user '{user_name}' not found in kubeconfig)");
    };

    if let Some(exec) = user.get("exec") {
        let cmd = exec
            .get("command")
            .and_then(|c| c.as_str())
            .unwrap_or("unknown");
        return format!(
            "exec plugin (command: \"{cmd}\") — the plugin binary must be in PATH when the app runs"
        );
    }

    if user.get("token").is_some() {
        return "bearer token (inline)".to_string();
    }

    if user.get("client-certificate-data").is_some() {
        return "client certificate (inline base64)".to_string();
    }

    if let Some(cert_path) = user.get("client-certificate").and_then(|c| c.as_str()) {
        return format!("client certificate (file: {cert_path}) — file must exist on this machine");
    }

    if user.get("username").is_some() {
        return "basic auth (username/password)".to_string();
    }

    "unknown".to_string()
}

/// Diagnostic: test a kubeconfig's ability to reach the cluster.
///
/// Runs two staged checks:
///   1. Connectivity — `kubectl get --raw=/healthz` (no auth required)
///   2. Authentication — `kubectl cluster-info` (requires valid credentials)
///
/// Also detects the auth method used by the context so the caller knows whether
/// an exec plugin or external certificate file might be missing.
/// This command is safe to call at any time — it writes a temp file, runs the
/// tests, then deletes the file regardless of the outcome.
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content.as_ref())
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;
    let auth_method = detect_auth_method(kubeconfig_content.as_ref(), &context);

    // Stage 1: basic connectivity — /healthz requires no authentication.
    let healthz = Command::new(&kubectl_path)
        .arg("get")
        .arg("--raw=/healthz")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    let healthz_ok = healthz.status.success();
    let healthz_body = String::from_utf8_lossy(&healthz.stdout).trim().to_string();
    let healthz_err = String::from_utf8_lossy(&healthz.stderr).trim().to_string();
    let connectivity_line = if healthz_ok {
        format!(
            "OK  ({})",
            if healthz_body.is_empty() {
                "cluster reachable"
            } else {
                &healthz_body
            }
        )
    } else {
        let hint = if healthz_err.is_empty() {
            "no stderr"
        } else {
            healthz_err.lines().last().unwrap_or(&healthz_err)
        };
        format!("FAIL  — {hint}")
    };

    // Stage 2: authenticated cluster-info.
    let auth_output = Command::new(&kubectl_path)
        .arg("cluster-info")
        .arg("--context")
        .arg(context.as_str())
        .arg("--kubeconfig")
        .arg(&temp_path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {e}"))?;

    let stdout = String::from_utf8_lossy(&auth_output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&auth_output.stderr).to_string();
    let exit_code = auth_output.status.code().unwrap_or(-1);

    Ok(format!(
        "Context:       {context}\nKubectl:       {kubectl}\nAuth method:   {auth}\n\n\
         ── Stage 1: Connectivity (/healthz, no auth) ──\n{connectivity}\n\n\
         ── Stage 2: Authentication (kubectl cluster-info) ──\nExit:     {exit}\n\n\
         --- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
        context = context,
        kubectl = kubectl_path.display(),
        auth = auth_method,
        connectivity = connectivity_line,
        exit = exit_code,
        stdout = if stdout.is_empty() {
            "(none)\n"
        } else {
            &stdout
        },
        stderr = if stderr.is_empty() {
            "(none)\n"
        } else {
            &stderr
        },
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
    let temp_path = unique_kubeconfig_path(&cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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
    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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
    let temp_path = unique_kubeconfig_path(&request.cluster_id);

    write_secure_temp_file(&temp_path, kubeconfig_content.as_ref())
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

        let restarts = item
            .get("status")
            .and_then(|s| s.get("containerStatuses"))
            .and_then(|c| c.as_array())
            .map(|container_statuses| {
                container_statuses
                    .iter()
                    .map(|c| c.get("restartCount").and_then(|r| r.as_u64()).unwrap_or(0) as u32)
                    .sum::<u32>()
            });

        let ip = item
            .get("status")
            .and_then(|s| s.get("podIP"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let node = item
            .get("spec")
            .and_then(|s| s.get("nodeName"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        pods.push(PodInfo {
            name,
            namespace,
            status,
            ready,
            age,
            containers,
            restarts,
            ip,
            node,
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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

// ─────────────────────────────────────────────────────────────────────────────
// Phase 4: Additional Resource Discovery
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationControllerInfo {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready: String,
    pub age: String,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodDisruptionBudgetInfo {
    pub name: String,
    pub namespace: String,
    pub min_available: String,
    pub max_unavailable: String,
    pub allowed_disruptions: i32,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityClassInfo {
    pub name: String,
    pub value: i32,
    pub global_default: bool,
    pub description: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeClassInfo {
    pub name: String,
    pub handler: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseInfo {
    pub name: String,
    pub namespace: String,
    pub holder: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutatingWebhookConfigurationInfo {
    pub name: String,
    pub webhooks: i32,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatingWebhookConfigurationInfo {
    pub name: String,
    pub webhooks: i32,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    pub name: String,
    pub namespace: String,
    pub endpoints: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSliceInfo {
    pub name: String,
    pub namespace: String,
    pub address_type: String,
    pub ports: String,
    pub endpoints: i32,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressClassInfo {
    pub name: String,
    pub controller: String,
    pub is_default: bool,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceResourceInfo {
    pub name: String,
    pub status: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterColumn {
    pub name: String,
    pub json_path: String,
    #[serde(rename = "type")]
    pub column_type: String,
    pub description: Option<String>,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdVersion {
    pub name: String,
    pub served: bool,
    pub storage: bool,
    pub printer_columns: Vec<PrinterColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdInfo {
    pub name: String,
    pub group: String,
    pub version: String,
    pub versions: Vec<CrdVersion>,
    pub kind: String,
    pub plural: String,
    pub scope: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomResourceInfo {
    pub name: String,
    pub namespace: String,
    pub age: String,
    pub additional_columns: HashMap<String, String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// list_replicationcontrollers
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_replicationcontrollers(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<ReplicationControllerInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("replicationcontrollers");
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
    parse_replicationcontrollers_json(&output_str)
}

fn parse_replicationcontrollers_json(
    json_str: &str,
) -> Result<Vec<ReplicationControllerInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut result = Vec::new();
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

        result.push(ReplicationControllerInfo {
            name,
            namespace,
            replicas,
            ready,
            age,
            labels,
        });
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// list_poddisruptionbudgets
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_poddisruptionbudgets(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<PodDisruptionBudgetInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("poddisruptionbudgets");
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
    parse_poddisruptionbudgets_json(&output_str)
}

fn parse_poddisruptionbudgets_json(json_str: &str) -> Result<Vec<PodDisruptionBudgetInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut result = Vec::new();
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

        let min_available = item
            .get("spec")
            .and_then(|s| s.get("minAvailable"))
            .map(|v| v.to_string().trim_matches('"').to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let max_unavailable = item
            .get("spec")
            .and_then(|s| s.get("maxUnavailable"))
            .map(|v| v.to_string().trim_matches('"').to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let allowed_disruptions = item
            .get("status")
            .and_then(|s| s.get("disruptionsAllowed"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        result.push(PodDisruptionBudgetInfo {
            name,
            namespace,
            min_available,
            max_unavailable,
            allowed_disruptions,
            age,
        });
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// list_priorityclasses  (cluster-scoped)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_priorityclasses(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<PriorityClassInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("priorityclasses")
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
    parse_priorityclasses_json(&output_str)
}

fn parse_priorityclasses_json(json_str: &str) -> Result<Vec<PriorityClassInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut result = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let value_int = item.get("value").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        let global_default = item
            .get("globalDefault")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let description = item
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        result.push(PriorityClassInfo {
            name,
            value: value_int,
            global_default,
            description,
            age,
        });
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// list_runtimeclasses  (cluster-scoped)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_runtimeclasses(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<RuntimeClassInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("runtimeclasses")
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
    parse_runtimeclasses_json(&output_str)
}

fn parse_runtimeclasses_json(json_str: &str) -> Result<Vec<RuntimeClassInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut result = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let handler = item
            .get("handler")
            .and_then(|h| h.as_str())
            .unwrap_or("unknown")
            .to_string();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        result.push(RuntimeClassInfo { name, handler, age });
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// list_leases
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_leases(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<LeaseInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("leases");
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
    parse_leases_json(&output_str)
}

fn parse_leases_json(json_str: &str) -> Result<Vec<LeaseInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut result = Vec::new();
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

        let holder = item
            .get("spec")
            .and_then(|s| s.get("holderIdentity"))
            .and_then(|h| h.as_str())
            .unwrap_or("none")
            .to_string();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        result.push(LeaseInfo {
            name,
            namespace,
            holder,
            age,
        });
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// list_mutatingwebhookconfigurations  (cluster-scoped)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_mutatingwebhookconfigurations(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<MutatingWebhookConfigurationInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("mutatingwebhookconfigurations")
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
    parse_mutatingwebhookconfigurations_json(&output_str)
}

fn parse_mutatingwebhookconfigurations_json(
    json_str: &str,
) -> Result<Vec<MutatingWebhookConfigurationInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut result = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let webhooks = item
            .get("webhooks")
            .and_then(|w| w.as_array())
            .map(|w| w.len() as i32)
            .unwrap_or(0);

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        result.push(MutatingWebhookConfigurationInfo {
            name,
            webhooks,
            age,
        });
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// list_validatingwebhookconfigurations  (cluster-scoped)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_validatingwebhookconfigurations(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ValidatingWebhookConfigurationInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("validatingwebhookconfigurations")
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
    parse_validatingwebhookconfigurations_json(&output_str)
}

fn parse_validatingwebhookconfigurations_json(
    json_str: &str,
) -> Result<Vec<ValidatingWebhookConfigurationInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut result = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let webhooks = item
            .get("webhooks")
            .and_then(|w| w.as_array())
            .map(|w| w.len() as i32)
            .unwrap_or(0);

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        result.push(ValidatingWebhookConfigurationInfo {
            name,
            webhooks,
            age,
        });
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// list_endpoints
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_endpoints(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<EndpointInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("endpoints");
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
    parse_endpoints_json(&output_str)
}

fn parse_endpoints_json(json_str: &str) -> Result<Vec<EndpointInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut result = Vec::new();
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

        // Collect IP:port pairs from subsets
        let endpoints = item
            .get("subsets")
            .and_then(|s| s.as_array())
            .map(|subsets| {
                let mut addrs = Vec::new();
                for subset in subsets {
                    if let Some(addresses) = subset.get("addresses").and_then(|a| a.as_array()) {
                        for addr in addresses {
                            if let Some(ip) = addr.get("ip").and_then(|i| i.as_str()) {
                                addrs.push(ip.to_string());
                            }
                        }
                    }
                }
                addrs.join(", ")
            })
            .unwrap_or_else(|| "<none>".to_string());

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        result.push(EndpointInfo {
            name,
            namespace,
            endpoints,
            age,
        });
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// list_endpointslices
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_endpointslices(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<EndpointSliceInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg("endpointslices");
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
    parse_endpointslices_json(&output_str)
}

fn parse_endpointslices_json(json_str: &str) -> Result<Vec<EndpointSliceInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut result = Vec::new();
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

        let address_type = item
            .get("addressType")
            .and_then(|a| a.as_str())
            .unwrap_or("IPv4")
            .to_string();

        let ports = item
            .get("ports")
            .and_then(|p| p.as_array())
            .map(|ports| {
                ports
                    .iter()
                    .filter_map(|p| p.get("port").and_then(|v| v.as_u64()))
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "<none>".to_string());

        let endpoints = item
            .get("endpoints")
            .and_then(|e| e.as_array())
            .map(|e| e.len() as i32)
            .unwrap_or(0);

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        result.push(EndpointSliceInfo {
            name,
            namespace,
            address_type,
            ports,
            endpoints,
            age,
        });
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// list_ingressclasses  (cluster-scoped)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_ingressclasses(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<IngressClassInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("ingressclasses")
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
    parse_ingressclasses_json(&output_str)
}

fn parse_ingressclasses_json(json_str: &str) -> Result<Vec<IngressClassInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut result = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let controller = item
            .get("spec")
            .and_then(|s| s.get("controller"))
            .and_then(|c| c.as_str())
            .unwrap_or("unknown")
            .to_string();

        let is_default = item
            .get("metadata")
            .and_then(|m| m.get("annotations"))
            .and_then(|a| {
                a.get("ingressclass.kubernetes.io/is-default-class")
                    .and_then(|v| v.as_str())
            })
            .map(|v| v == "true")
            .unwrap_or(false);

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        result.push(IngressClassInfo {
            name,
            controller,
            is_default,
            age,
        });
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// list_namespaces_resource  (cluster-scoped, distinct from list_namespaces)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_namespaces_resource(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<NamespaceResourceInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
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
    parse_namespaces_resource_json(&output_str)
}

fn parse_namespaces_resource_json(json_str: &str) -> Result<Vec<NamespaceResourceInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut result = Vec::new();
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

        result.push(NamespaceResourceInfo { name, status, age });
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// list_crds  (cluster-scoped)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_crds(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<CrdInfo>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("get")
        .arg("customresourcedefinitions")
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
    parse_crds_json(&output_str)
}

fn parse_crds_json(json_str: &str) -> Result<Vec<CrdInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut result = Vec::new();
    for item in items {
        let name = item
            .get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let group = item
            .get("spec")
            .and_then(|s| s.get("group"))
            .and_then(|g| g.as_str())
            .unwrap_or("unknown")
            .to_string();

        let plural = item
            .get("spec")
            .and_then(|s| s.get("names"))
            .and_then(|n| n.get("plural"))
            .and_then(|p| p.as_str())
            .unwrap_or_else(|| {
                // Fallback: use name's first segment
                name.split('.').next().unwrap_or("unknown")
            })
            .to_string();

        let kind = item
            .get("spec")
            .and_then(|s| s.get("names"))
            .and_then(|n| n.get("kind"))
            .and_then(|k| k.as_str())
            .unwrap_or("unknown")
            .to_string();

        let scope = item
            .get("spec")
            .and_then(|s| s.get("scope"))
            .and_then(|s| s.as_str())
            .unwrap_or("Namespaced")
            .to_string();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        // Parse all versions with their printer columns
        let versions: Vec<CrdVersion> = item
            .get("spec")
            .and_then(|s| s.get("versions"))
            .and_then(|v| v.as_array())
            .map(|versions_array| {
                versions_array
                    .iter()
                    .filter_map(|ver| {
                        let version_name = ver.get("name").and_then(|n| n.as_str())?.to_string();
                        let served = ver.get("served").and_then(|s| s.as_bool()).unwrap_or(true);
                        let storage = ver
                            .get("storage")
                            .and_then(|s| s.as_bool())
                            .unwrap_or(false);

                        // Parse printer columns for this version
                        let printer_columns: Vec<PrinterColumn> = ver
                            .get("additionalPrinterColumns")
                            .and_then(|c| c.as_array())
                            .map(|cols| {
                                cols.iter()
                                    .filter_map(|col| {
                                        let col_name =
                                            col.get("name").and_then(|n| n.as_str())?.to_string();
                                        let json_path = col
                                            .get("jsonPath")
                                            .and_then(|j| j.as_str())?
                                            .to_string();
                                        let column_type = col
                                            .get("type")
                                            .and_then(|t| t.as_str())
                                            .unwrap_or("string")
                                            .to_string();
                                        let description = col
                                            .get("description")
                                            .and_then(|d| d.as_str())
                                            .map(|s| s.to_string());
                                        let priority = col
                                            .get("priority")
                                            .and_then(|p| p.as_i64())
                                            .unwrap_or(0)
                                            as i32;

                                        Some(PrinterColumn {
                                            name: col_name,
                                            json_path,
                                            column_type,
                                            description,
                                            priority,
                                        })
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        Some(CrdVersion {
                            name: version_name,
                            served,
                            storage,
                            printer_columns,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Default version is the first one (or the storage version if available)
        let version = versions
            .iter()
            .find(|v| v.storage)
            .or_else(|| versions.first())
            .map(|v| v.name.clone())
            .unwrap_or_else(|| "v1".to_string());

        result.push(CrdInfo {
            name,
            group,
            version,
            versions,
            kind,
            plural,
            scope,
            age,
        });
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// list_custom_resources
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_custom_resources(
    cluster_id: String,
    group: String,
    version: String,
    resource: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<CustomResourceInfo>, String> {
    validate_resource_name(&group, "group")?;
    validate_resource_name(&resource, "resource")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(&cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    // Build resource specifier: group/resource (version is part of the API group context)
    let resource_spec = format!("{}/{}", group, resource);

    let mut kubectl_cmd = Command::new(kubectl_path);
    kubectl_cmd.arg("get").arg(&resource_spec);
    if namespace.is_empty() {
        kubectl_cmd.arg("--all-namespaces");
    } else {
        kubectl_cmd.arg("-n").arg(&namespace);
    }

    info!(
        cluster_id = %cluster_id,
        group = %group,
        version = %version,
        resource = %resource,
        "Listing custom resources"
    );

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
    parse_custom_resources_json(&output_str)
}

fn parse_custom_resources_json(json_str: &str) -> Result<Vec<CustomResourceInfo>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))?;

    let items = value
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or("Missing 'items' array in kubectl JSON output")?;

    let mut result = Vec::new();
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
            .unwrap_or("")
            .to_string();

        let age = item
            .get("metadata")
            .and_then(|m| m.get("creationTimestamp"))
            .and_then(|c| c.as_str())
            .map(parse_creation_timestamp)
            .unwrap_or("N/A".to_string());

        // For now, we don't extract additional columns here as we don't have the CRD spec
        // The frontend will need to call with the CRD info to get proper column extraction
        // This is a limitation - ideally we'd pass printer columns to this function
        let additional_columns = HashMap::new();

        result.push(CustomResourceInfo {
            name,
            namespace,
            age,
            additional_columns,
        });
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 5: Action commands
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescribeResponse {
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecSessionResponse {
    pub session_id: String,
    pub cluster_id: String,
    pub namespace: String,
    pub pod: String,
    pub container: Option<String>,
    pub status: String,
}

#[tauri::command]
pub async fn force_delete_resource(
    cluster_id: String,
    resource_type: String,
    namespace: String,
    resource_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&resource_name, "resource_name")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(&cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    info!(
        cluster_id = %cluster_id,
        resource_type = %resource_type,
        namespace = %namespace,
        resource_name = %resource_name,
        "Force deleting resource"
    );

    let output = Command::new(kubectl_path)
        .arg("delete")
        .arg(&resource_type)
        .arg(&resource_name)
        .arg("-n")
        .arg(&namespace)
        .arg("--grace-period=0")
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
pub async fn describe_resource(
    cluster_id: String,
    resource_type: String,
    namespace: String,
    resource_name: String,
    state: State<'_, AppState>,
) -> Result<DescribeResponse, String> {
    validate_resource_name(&resource_name, "resource_name")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;
    let resource_spec = format!("{}/{}", resource_type, resource_name);

    let mut cmd = Command::new(kubectl_path);
    cmd.arg("describe").arg(&resource_spec);

    if !namespace.is_empty() {
        cmd.arg("-n").arg(&namespace);
    }

    let output = cmd
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

    let output_text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(DescribeResponse {
        output: output_text,
    })
}

#[tauri::command]
pub async fn get_resource_yaml(
    cluster_id: String,
    resource_type: String,
    namespace: String,
    resource_name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    validate_resource_name(&resource_name, "resource_name")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;
    let resource_spec = format!("{}/{}", resource_type, resource_name);

    let mut cmd = Command::new(kubectl_path);
    cmd.arg("get").arg(&resource_spec).arg("-o").arg("yaml");

    if !namespace.is_empty() {
        cmd.arg("-n").arg(&namespace);
    }

    let output = cmd
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

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub async fn attach_pod(
    cluster_id: String,
    namespace: String,
    pod_name: String,
    container_name: String,
    state: State<'_, AppState>,
) -> Result<ExecSessionResponse, String> {
    validate_resource_name(&pod_name, "pod_name")?;
    validate_resource_name(&namespace, "namespace")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let session_id = uuid::Uuid::now_v7().to_string();

    let temp_path = unique_kubeconfig_path(&cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut cmd = Command::new(kubectl_path);
    cmd.arg("attach")
        .arg("-it")
        .arg(&pod_name)
        .arg("-n")
        .arg(&namespace);

    if !container_name.is_empty() {
        cmd.arg("-c").arg(&container_name);
    }

    cmd.arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(context.as_str());

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl attach: {e}"))?;

    let status = if output.status.success() {
        "Completed".to_string()
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        format!("Error: {}", stderr.trim())
    };

    Ok(ExecSessionResponse {
        session_id,
        cluster_id,
        namespace,
        pod: pod_name,
        container: if container_name.is_empty() {
            None
        } else {
            Some(container_name)
        },
        status,
    })
}

#[tauri::command]
pub async fn restart_statefulset(
    cluster_id: String,
    namespace: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&name, "name")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("rollout")
        .arg("restart")
        .arg(format!("statefulsets/{}", name))
        .arg("-n")
        .arg(&namespace)
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
pub async fn restart_daemonset(
    cluster_id: String,
    namespace: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&name, "name")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("rollout")
        .arg("restart")
        .arg(format!("daemonsets/{}", name))
        .arg("-n")
        .arg(&namespace)
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
pub async fn scale_statefulset(
    cluster_id: String,
    namespace: String,
    name: String,
    replicas: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&name, "name")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("scale")
        .arg(format!("statefulsets/{}", name))
        .arg(format!("--replicas={}", replicas))
        .arg("-n")
        .arg(&namespace)
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
pub async fn scale_replicaset(
    cluster_id: String,
    namespace: String,
    name: String,
    replicas: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&name, "name")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("scale")
        .arg(format!("replicasets/{}", name))
        .arg(format!("--replicas={}", replicas))
        .arg("-n")
        .arg(&namespace)
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
pub async fn scale_replicationcontroller(
    cluster_id: String,
    namespace: String,
    name: String,
    replicas: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&name, "name")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("scale")
        .arg(format!("replicationcontrollers/{}", name))
        .arg(format!("--replicas={}", replicas))
        .arg("-n")
        .arg(&namespace)
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
pub async fn suspend_cronjob(
    cluster_id: String,
    namespace: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&name, "name")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("patch")
        .arg(format!("cronjob/{}", name))
        .arg("-p")
        .arg(r#"{"spec":{"suspend":true}}"#)
        .arg("-n")
        .arg(&namespace)
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
pub async fn resume_cronjob(
    cluster_id: String,
    namespace: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&name, "name")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let output = Command::new(kubectl_path)
        .arg("patch")
        .arg(format!("cronjob/{}", name))
        .arg("-p")
        .arg(r#"{"spec":{"suspend":false}}"#)
        .arg("-n")
        .arg(&namespace)
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
pub async fn trigger_cronjob(
    cluster_id: String,
    namespace: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&name, "name")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let job_name = format!("{}-manual", name);
    let from_spec = format!("cronjob/{}", name);

    let output = Command::new(kubectl_path)
        .arg("create")
        .arg("job")
        .arg(&job_name)
        .arg("--from")
        .arg(&from_spec)
        .arg("-n")
        .arg(&namespace)
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
pub async fn create_namespace(
    cluster_id: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&name, "name")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(&cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    info!(cluster_id = %cluster_id, namespace = %name, "Creating namespace");

    let output = Command::new(kubectl_path)
        .arg("create")
        .arg("namespace")
        .arg(&name)
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
pub async fn delete_namespace(
    cluster_id: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&name, "name")?;

    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;

    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let context = &cluster.context;

    let temp_path = unique_kubeconfig_path(&cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_content)
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    info!(cluster_id = %cluster_id, namespace = %name, "Deleting namespace");

    let output = Command::new(kubectl_path)
        .arg("delete")
        .arg("namespace")
        .arg(&name)
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

// ─────────────────────────────────────────────────────────────────────────────
// Phase 6: Log streaming (Tauri 2.x event channel)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStreamConfig {
    pub cluster_id: String,
    pub namespace: String,
    pub pod_name: String,
    pub container_name: String,
    pub follow: bool,
    pub timestamps: bool,
    pub tail_lines: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    pub stream_id: String,
    pub line: String,
}

#[tauri::command]
pub async fn stream_pod_logs(
    config: LogStreamConfig,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    validate_resource_name(&config.pod_name, "pod_name")?;
    validate_resource_name(&config.namespace, "namespace")?;

    let stream_id = uuid::Uuid::now_v7().to_string();

    let kubeconfig_content = {
        let clusters = state.clusters.lock().await;
        let cluster = clusters
            .get(&config.cluster_id)
            .ok_or_else(|| format!("Cluster {} not found", config.cluster_id))?;
        (cluster.kubeconfig_content.clone(), cluster.context.clone())
    };

    let (kubeconfig_arc, context) = kubeconfig_content;

    let temp_path = unique_kubeconfig_path(config.cluster_id);

    write_secure_temp_file(&temp_path, kubeconfig_arc.as_ref())
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let kubectl_path = locate_kubectl()?;

    let mut cmd = Command::new(kubectl_path);
    cmd.arg("logs")
        .arg(&config.pod_name)
        .arg("-n")
        .arg(&config.namespace);

    if !config.container_name.is_empty() {
        cmd.arg("-c").arg(&config.container_name);
    }

    if config.follow {
        cmd.arg("-f");
    }

    if config.timestamps {
        cmd.arg("--timestamps");
    }

    if let Some(tail) = config.tail_lines {
        cmd.arg(format!("--tail={}", tail));
    }

    cmd.arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--context")
        .arg(&context)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn kubectl logs: {e}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture kubectl stdout")?;

    let stream_id_clone = stream_id.clone();
    let app_handle_clone = app_handle.clone();

    let task = tokio::spawn(async move {
        let _cleanup = TempFileCleanup(temp_path);

        use tokio::io::{AsyncBufReadExt, BufReader};
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let payload = LogLine {
                stream_id: stream_id_clone.clone(),
                line,
            };
            if let Err(e) = app_handle_clone.emit("pod-log-line", &payload) {
                tracing::warn!(stream_id = %stream_id_clone, "Failed to emit log line event: {e}");
                break;
            }
        }

        let _ = child.wait().await;
    });

    let abort_handle = task.abort_handle();

    {
        let mut streams = state.log_streams.lock().await;
        streams.insert(stream_id.clone(), abort_handle);
    }

    info!(stream_id = %stream_id, pod = %config.pod_name, "Started pod log stream");

    Ok(stream_id)
}

#[tauri::command]
pub async fn stop_log_stream(stream_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut streams = state.log_streams.lock().await;
    if let Some(handle) = streams.remove(&stream_id) {
        handle.abort();
        info!(stream_id = %stream_id, "Stopped pod log stream");
        Ok(())
    } else {
        Err(format!("Log stream {} not found", stream_id))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 7: Helm commands
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmRepository {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmChart {
    pub name: String,
    pub chart_version: String,
    pub app_version: String,
    pub description: String,
    pub repository: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmRelease {
    pub name: String,
    pub namespace: String,
    pub chart: String,
    pub chart_version: String,
    pub app_version: String,
    pub status: String,
    pub updated: String,
}

#[tauri::command]
pub async fn helm_list_repos(
    _cluster_id: String,
    _state: State<'_, AppState>,
) -> Result<Vec<HelmRepository>, String> {
    let helm_path = locate_helm()?;

    let output = Command::new(helm_path)
        .arg("repo")
        .arg("list")
        .arg("--output")
        .arg("json")
        .output()
        .await
        .map_err(|e| format!("Failed to execute helm: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // helm repo list exits non-zero when no repos are configured — treat as empty list
        if stderr.contains("no repositories") || stderr.contains("Error: no repositories") {
            return Ok(Vec::new());
        }
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_helm_repos_json(&output_str)
}

fn parse_helm_repos_json(json_str: &str) -> Result<Vec<HelmRepository>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse helm JSON output: {}", e))?;

    let items = value
        .as_array()
        .ok_or("Expected JSON array from helm repo list")?;

    let mut result = Vec::new();
    for item in items {
        let name = item
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let url = item
            .get("url")
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();

        result.push(HelmRepository { name, url });
    }

    Ok(result)
}

#[tauri::command]
pub async fn helm_add_repo(
    _cluster_id: String,
    name: String,
    url: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&name, "repo name")?;

    let helm_path = locate_helm()?;

    info!(repo_name = %name, repo_url = %url, "Adding helm repository");

    let output = Command::new(helm_path)
        .arg("repo")
        .arg("add")
        .arg(&name)
        .arg(&url)
        .output()
        .await
        .map_err(|e| format!("Failed to execute helm: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn helm_update_repos(
    _cluster_id: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    let helm_path = locate_helm()?;

    let output = Command::new(helm_path)
        .arg("repo")
        .arg("update")
        .output()
        .await
        .map_err(|e| format!("Failed to execute helm: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn helm_search_repo(
    _cluster_id: String,
    query: String,
    _state: State<'_, AppState>,
) -> Result<Vec<HelmChart>, String> {
    let helm_path = locate_helm()?;

    let output = Command::new(helm_path)
        .arg("search")
        .arg("repo")
        .arg(&query)
        .arg("--output")
        .arg("json")
        .output()
        .await
        .map_err(|e| format!("Failed to execute helm: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_helm_search_json(&output_str)
}

fn parse_helm_search_json(json_str: &str) -> Result<Vec<HelmChart>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse helm JSON output: {}", e))?;

    let items = value
        .as_array()
        .ok_or("Expected JSON array from helm search repo")?;

    let mut result = Vec::new();
    for item in items {
        let name = item
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let chart_version = item
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let app_version = item
            .get("app_version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let description = item
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();

        // Repository is the prefix before the first '/' in the chart name
        let repository = name.split('/').next().unwrap_or("").to_string();

        result.push(HelmChart {
            name,
            chart_version,
            app_version,
            description,
            repository,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn helm_list_releases(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<HelmRelease>, String> {
    let kubeconfig_content = {
        let clusters = state.clusters.lock().await;
        let cluster = clusters
            .get(&cluster_id)
            .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
        (cluster.kubeconfig_content.clone(), cluster.context.clone())
    };

    let (kubeconfig_arc, context) = kubeconfig_content;

    let temp_path = unique_kubeconfig_path(cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_arc.as_ref())
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let helm_path = locate_helm()?;

    let mut cmd = Command::new(helm_path);
    cmd.arg("list");

    if namespace.is_empty() {
        cmd.arg("--all-namespaces");
    } else {
        cmd.arg("-n").arg(&namespace);
    }

    let output = cmd
        .arg("--output")
        .arg("json")
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--kube-context")
        .arg(&context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute helm: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_helm_releases_json(&output_str)
}

fn parse_helm_releases_json(json_str: &str) -> Result<Vec<HelmRelease>, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse helm JSON output: {}", e))?;

    let items = value
        .as_array()
        .ok_or("Expected JSON array from helm list")?;

    let mut result = Vec::new();
    for item in items {
        let name = item
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let namespace = item
            .get("namespace")
            .and_then(|n| n.as_str())
            .unwrap_or("default")
            .to_string();

        let chart = item
            .get("chart")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        // chart field is "chartname-version" — split off the version suffix
        let (chart_name, chart_version) = if let Some(pos) = chart.rfind('-') {
            (chart[..pos].to_string(), chart[pos + 1..].to_string())
        } else {
            (chart.clone(), String::new())
        };

        let app_version = item
            .get("app_version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let status = item
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();

        let updated = item
            .get("updated")
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();

        result.push(HelmRelease {
            name,
            namespace,
            chart: chart_name,
            chart_version,
            app_version,
            status,
            updated,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn helm_uninstall(
    cluster_id: String,
    namespace: String,
    release_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&release_name, "release_name")?;

    let kubeconfig_content = {
        let clusters = state.clusters.lock().await;
        let cluster = clusters
            .get(&cluster_id)
            .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
        (cluster.kubeconfig_content.clone(), cluster.context.clone())
    };

    let (kubeconfig_arc, context) = kubeconfig_content;

    let temp_path = unique_kubeconfig_path(&cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_arc.as_ref())
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let helm_path = locate_helm()?;

    info!(cluster_id = %cluster_id, release = %release_name, namespace = %namespace, "Uninstalling helm release");

    let output = Command::new(helm_path)
        .arg("uninstall")
        .arg(&release_name)
        .arg("-n")
        .arg(&namespace)
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--kube-context")
        .arg(&context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute helm: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn helm_rollback(
    cluster_id: String,
    namespace: String,
    release_name: String,
    revision: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_resource_name(&release_name, "release_name")?;

    let kubeconfig_content = {
        let clusters = state.clusters.lock().await;
        let cluster = clusters
            .get(&cluster_id)
            .ok_or_else(|| format!("Cluster {} not found", cluster_id))?;
        (cluster.kubeconfig_content.clone(), cluster.context.clone())
    };

    let (kubeconfig_arc, context) = kubeconfig_content;

    let temp_path = unique_kubeconfig_path(&cluster_id);
    let _cleanup = TempFileCleanup(temp_path.clone());

    write_secure_temp_file(&temp_path, kubeconfig_arc.as_ref())
        .map_err(|e| format!("Failed to write kubeconfig temp file: {e}"))?;

    let helm_path = locate_helm()?;

    info!(cluster_id = %cluster_id, release = %release_name, revision = ?revision, "Rolling back helm release");

    let mut cmd = Command::new(helm_path);
    cmd.arg("rollback").arg(&release_name);

    if let Some(rev) = revision {
        cmd.arg(rev.to_string());
    }

    let output = cmd
        .arg("-n")
        .arg(&namespace)
        .arg("--kubeconfig")
        .arg(&temp_path)
        .arg("--kube-context")
        .arg(&context)
        .output()
        .await
        .map_err(|e| format!("Failed to execute helm: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 8: New command unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod new_command_tests {
    use super::*;

    #[test]
    fn test_parse_replicationcontrollers_json() {
        let json = r#"{"items":[{"metadata":{"name":"my-rc","namespace":"default","creationTimestamp":"2024-01-01T00:00:00Z","labels":{"app":"myapp"}},"spec":{"replicas":3},"status":{"readyReplicas":3}}]}"#;
        let result = parse_replicationcontrollers_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "my-rc");
        assert_eq!(result[0].namespace, "default");
        assert_eq!(result[0].replicas, 3);
        assert_eq!(result[0].ready, "3/3");
    }

    #[test]
    fn test_parse_replicationcontrollers_json_empty() {
        let json = r#"{"items":[]}"#;
        let result = parse_replicationcontrollers_json(json).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_poddisruptionbudgets_json() {
        let json = r#"{"items":[{"metadata":{"name":"my-pdb","namespace":"default","creationTimestamp":"2024-01-01T00:00:00Z"},"spec":{"minAvailable":1},"status":{"disruptionsAllowed":2}}]}"#;
        let result = parse_poddisruptionbudgets_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "my-pdb");
        assert_eq!(result[0].allowed_disruptions, 2);
    }

    #[test]
    fn test_parse_priorityclasses_json() {
        let json = r#"{"items":[{"metadata":{"name":"high-priority","creationTimestamp":"2024-01-01T00:00:00Z"},"value":1000,"globalDefault":false,"description":"High priority class"}]}"#;
        let result = parse_priorityclasses_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "high-priority");
        assert_eq!(result[0].value, 1000);
        assert!(!result[0].global_default);
        assert_eq!(result[0].description, "High priority class");
    }

    #[test]
    fn test_parse_runtimeclasses_json() {
        let json = r#"{"items":[{"metadata":{"name":"gvisor","creationTimestamp":"2024-01-01T00:00:00Z"},"handler":"runsc"}]}"#;
        let result = parse_runtimeclasses_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "gvisor");
        assert_eq!(result[0].handler, "runsc");
    }

    #[test]
    fn test_parse_leases_json() {
        let json = r#"{"items":[{"metadata":{"name":"my-lease","namespace":"kube-system","creationTimestamp":"2024-01-01T00:00:00Z"},"spec":{"holderIdentity":"node-1"}}]}"#;
        let result = parse_leases_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "my-lease");
        assert_eq!(result[0].holder, "node-1");
    }

    #[test]
    fn test_parse_mutatingwebhookconfigurations_json() {
        let json = r#"{"items":[{"metadata":{"name":"my-mwh","creationTimestamp":"2024-01-01T00:00:00Z"},"webhooks":[{"name":"w1.example.com"},{"name":"w2.example.com"}]}]}"#;
        let result = parse_mutatingwebhookconfigurations_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "my-mwh");
        assert_eq!(result[0].webhooks, 2);
    }

    #[test]
    fn test_parse_validatingwebhookconfigurations_json() {
        let json = r#"{"items":[{"metadata":{"name":"my-vwh","creationTimestamp":"2024-01-01T00:00:00Z"},"webhooks":[{"name":"v1.example.com"}]}]}"#;
        let result = parse_validatingwebhookconfigurations_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].webhooks, 1);
    }

    #[test]
    fn test_parse_endpoints_json() {
        let json = r#"{"items":[{"metadata":{"name":"my-svc","namespace":"default","creationTimestamp":"2024-01-01T00:00:00Z"},"subsets":[{"addresses":[{"ip":"10.0.0.1"},{"ip":"10.0.0.2"}],"ports":[{"port":80}]}]}]}"#;
        let result = parse_endpoints_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "my-svc");
        assert!(result[0].endpoints.contains("10.0.0.1"));
    }

    #[test]
    fn test_parse_endpointslices_json() {
        let json = r#"{"items":[{"metadata":{"name":"my-eps","namespace":"default","creationTimestamp":"2024-01-01T00:00:00Z"},"addressType":"IPv4","ports":[{"port":80}],"endpoints":[{"addresses":["10.0.0.1"]},{"addresses":["10.0.0.2"]}]}]}"#;
        let result = parse_endpointslices_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].address_type, "IPv4");
        assert_eq!(result[0].endpoints, 2);
        assert_eq!(result[0].ports, "80");
    }

    #[test]
    fn test_parse_ingressclasses_json() {
        let json = r#"{"items":[{"metadata":{"name":"nginx","creationTimestamp":"2024-01-01T00:00:00Z","annotations":{"ingressclass.kubernetes.io/is-default-class":"true"}},"spec":{"controller":"k8s.io/ingress-nginx"}}]}"#;
        let result = parse_ingressclasses_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "nginx");
        assert_eq!(result[0].controller, "k8s.io/ingress-nginx");
        assert!(result[0].is_default);
    }

    #[test]
    fn test_parse_namespaces_resource_json() {
        let json = r#"{"items":[{"metadata":{"name":"kube-system","creationTimestamp":"2024-01-01T00:00:00Z"},"status":{"phase":"Active"}}]}"#;
        let result = parse_namespaces_resource_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "kube-system");
        assert_eq!(result[0].status, "Active");
    }

    #[test]
    fn test_parse_crds_json() {
        let json = r#"{"items":[{"metadata":{"name":"foos.example.com","creationTimestamp":"2024-01-01T00:00:00Z"},"spec":{"group":"example.com","versions":[{"name":"v1alpha1"}],"names":{"kind":"Foo"},"scope":"Namespaced"}}]}"#;
        let result = parse_crds_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].group, "example.com");
        assert_eq!(result[0].version, "v1alpha1");
        assert_eq!(result[0].kind, "Foo");
        assert_eq!(result[0].scope, "Namespaced");
    }

    #[test]
    fn test_parse_custom_resources_json() {
        let json = r#"{"items":[{"metadata":{"name":"my-foo","namespace":"default","creationTimestamp":"2024-01-01T00:00:00Z"}}]}"#;
        let result = parse_custom_resources_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "my-foo");
        assert_eq!(result[0].namespace, "default");
    }

    #[test]
    fn test_parse_helm_repos_json() {
        let json = r#"[{"name":"stable","url":"https://charts.helm.sh/stable"},{"name":"bitnami","url":"https://charts.bitnami.com/bitnami"}]"#;
        let result = parse_helm_repos_json(json).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "stable");
        assert_eq!(result[1].name, "bitnami");
    }

    #[test]
    fn test_parse_helm_search_json() {
        let json = r#"[{"name":"bitnami/nginx","version":"15.0.0","app_version":"1.25.0","description":"NGINX Open Source is a web server"}]"#;
        let result = parse_helm_search_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "bitnami/nginx");
        assert_eq!(result[0].chart_version, "15.0.0");
        assert_eq!(result[0].repository, "bitnami");
    }

    #[test]
    fn test_parse_helm_releases_json() {
        let json = r#"[{"name":"my-release","namespace":"default","chart":"nginx-15.0.0","app_version":"1.25.0","status":"deployed","updated":"2024-01-01 12:00:00.000000000 +0000 UTC"}]"#;
        let result = parse_helm_releases_json(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "my-release");
        assert_eq!(result[0].chart, "nginx");
        assert_eq!(result[0].chart_version, "15.0.0");
        assert_eq!(result[0].status, "deployed");
    }

    #[test]
    fn test_parse_helm_repos_json_empty() {
        let json = r#"[]"#;
        let result = parse_helm_repos_json(json).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_crds_json_empty() {
        let json = r#"{"items":[]}"#;
        let result = parse_crds_json(json).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_unique_kubeconfig_path_produces_distinct_paths() {
        let path1 = unique_kubeconfig_path("test-cluster");
        let path2 = unique_kubeconfig_path("test-cluster");
        assert_ne!(
            path1, path2,
            "successive calls must return distinct paths to prevent concurrent-call race conditions"
        );
    }
}
