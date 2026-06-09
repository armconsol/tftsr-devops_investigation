use crate::metrics::{NodeMetrics, PodMetrics};
use crate::state::AppState;
use tauri::State;

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

    // Write temp kubeconfig
    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let temp_path =
        std::env::temp_dir().join(format!("kubeconfig-metrics-{}.yaml", uuid::Uuid::now_v7()));
    std::fs::write(&temp_path, kubeconfig_content.as_bytes())
        .map_err(|e| format!("Failed to write kubeconfig: {e}"))?;

    // Ensure owner-only permissions (0600 on Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("Failed to set kubeconfig permissions: {e}"))?;
    }

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
        temp_path.to_string_lossy().to_string(),
    ];

    let output = crate::shell::kubectl::execute_kubectl(&args, None, None).await?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    if output.exit_code != 0 {
        return Err(format!("kubectl top pods failed: {}", output.stderr));
    }

    let json_output = &output.stdout;
    crate::metrics::client::parse_pod_metrics(&json_output)
        .map_err(|e| format!("Failed to parse pod metrics: {e}"))
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

    // Write temp kubeconfig
    let kubeconfig_content = cluster.kubeconfig_content.as_ref();
    let temp_path =
        std::env::temp_dir().join(format!("kubeconfig-metrics-{}.yaml", uuid::Uuid::now_v7()));
    std::fs::write(&temp_path, kubeconfig_content.as_bytes())
        .map_err(|e| format!("Failed to write kubeconfig: {e}"))?;

    // Ensure owner-only permissions (0600 on Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("Failed to set kubeconfig permissions: {e}"))?;
    }

    // Run kubectl top nodes with JSON output
    let args = vec![
        "top".to_string(),
        "nodes".to_string(),
        "--no-headers=false".to_string(),
        "-o".to_string(),
        "json".to_string(),
        "--kubeconfig".to_string(),
        temp_path.to_string_lossy().to_string(),
    ];

    let output = crate::shell::kubectl::execute_kubectl(&args, None, None).await?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    if output.exit_code != 0 {
        return Err(format!("kubectl top nodes failed: {}", output.stderr));
    }

    let json_output = &output.stdout;
    crate::metrics::client::parse_node_metrics(&json_output)
        .map_err(|e| format!("Failed to parse node metrics: {e}"))
}
