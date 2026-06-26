use crate::metrics::{NodeMetrics, PodMetrics};
use crate::state::AppState;
use tauri::State;

/// RAII guard that removes a temp kubeconfig file when dropped.
///
/// Using a Drop-based guard guarantees the sensitive kubeconfig is removed
/// even on panic or early `?` return — manual `remove_file` calls only run
/// on the happy path and were silently leaking the file on errors.
struct TempKubeconfig(std::path::PathBuf);

impl TempKubeconfig {
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempKubeconfig {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.0) {
            // Only log when the file actually existed; NotFound is expected on
            // Windows when the path was never written.
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    "Failed to remove temp kubeconfig {}: {}",
                    self.0.display(),
                    e
                );
            }
        }
    }
}

/// Write the kubeconfig content to a unique temp file with 0600 permissions
/// and return an RAII guard that cleans up on drop.
fn write_temp_kubeconfig(content: &str) -> Result<TempKubeconfig, String> {
    let path =
        std::env::temp_dir().join(format!("kubeconfig-metrics-{}.yaml", uuid::Uuid::now_v7()));
    let guard = TempKubeconfig(path);

    std::fs::write(guard.path(), content.as_bytes())
        .map_err(|e| format!("Failed to write kubeconfig: {e}"))?;

    // Ensure owner-only permissions (0600 on Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(guard.path(), std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("Failed to set kubeconfig permissions: {e}"))?;
    }

    Ok(guard)
}

/// Get pod metrics from kubectl top pods
#[tauri::command]
pub async fn get_pod_metrics(
    cluster_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<PodMetrics>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| "Cluster not found".to_string())?;

    // Write temp kubeconfig (auto-removed on drop)
    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let kubeconfig = write_temp_kubeconfig(kubeconfig_content)?;

    // Run kubectl top pods with JSON output
    let args = vec![
        "top".to_string(),
        "pods".to_string(),
        "-n".to_string(),
        namespace,
        "--no-headers=false".to_string(),
        "-o".to_string(),
        "json".to_string(),
        "--kubeconfig".to_string(),
        kubeconfig.path().to_string_lossy().to_string(),
    ];

    let output = crate::shell::kubectl::execute_kubectl(&args, None, None).await?;

    if output.exit_code != 0 {
        return Err(format!("kubectl top pods failed: {}", output.stderr));
    }

    let json_output = &output.stdout;
    crate::metrics::client::parse_pod_metrics(json_output)
        .map_err(|e| format!("Failed to parse pod metrics: {e}"))
    // kubeconfig dropped here, file removed
}

/// Get node metrics from kubectl top nodes
#[tauri::command]
pub async fn get_node_metrics(
    cluster_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<NodeMetrics>, String> {
    let clusters = state.clusters.lock().await;
    let cluster = clusters
        .get(&cluster_id)
        .ok_or_else(|| "Cluster not found".to_string())?;

    // Write temp kubeconfig (auto-removed on drop)
    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let kubeconfig = write_temp_kubeconfig(kubeconfig_content)?;

    // Run kubectl top nodes with JSON output
    let args = vec![
        "top".to_string(),
        "nodes".to_string(),
        "--no-headers=false".to_string(),
        "-o".to_string(),
        "json".to_string(),
        "--kubeconfig".to_string(),
        kubeconfig.path().to_string_lossy().to_string(),
    ];

    let output = crate::shell::kubectl::execute_kubectl(&args, None, None).await?;

    if output.exit_code != 0 {
        return Err(format!("kubectl top nodes failed: {}", output.stderr));
    }

    let json_output = &output.stdout;
    crate::metrics::client::parse_node_metrics(json_output)
        .map_err(|e| format!("Failed to parse node metrics: {e}"))
    // kubeconfig dropped here, file removed
}
