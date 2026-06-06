use crate::kube::ClusterClient;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::sync::Arc;
use tauri::State;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardResponse {
    pub id: String,
    pub cluster_id: String,
    pub namespace: String,
    pub pod: String,
    pub container_port: u16,
    pub local_port: u16,
    pub status: String,
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
    let value: Value = serde_yaml::from_str(content)
        .map_err(|e| format!("Invalid kubeconfig YAML: {}", e))?;

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
    let value: Value = serde_yaml::from_str(content)
        .map_err(|e| format!("Invalid kubeconfig YAML: {}", e))?;

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
    let mut clusters = state.clusters.lock().await;

    if clusters.remove(&id).is_none() {
        return Err(format!("Cluster {id} not found"));
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
pub async fn start_port_forward(
    request: PortForwardRequest,
    state: State<'_, AppState>,
) -> Result<PortForwardResponse, String> {
    let session_id = uuid::Uuid::now_v7().to_string();

    let clusters = state.clusters.lock().await;
    let cluster = clusters.get(&request.cluster_id)
        .ok_or_else(|| format!("Cluster {} not found", request.cluster_id))?;

    let cluster_name = cluster.name.clone();
    let _kubeconfig_content = cluster.kubeconfig_content.clone();

    let session = crate::kube::PortForwardSession::new(
        session_id.clone(),
        request.cluster_id.clone(),
        cluster_name,
        request.namespace.clone(),
        request.pod.clone(),
        None,
        vec![request.container_port],
        vec![0],
    );

    {
        let mut port_forwards = state.port_forwards.lock().await;
        port_forwards.insert(session_id.clone(), session);
    }

    Ok(PortForwardResponse {
        id: session_id,
        cluster_id: request.cluster_id,
        namespace: request.namespace,
        pod: request.pod,
        container_port: request.container_port,
        local_port: 0,
        status: "Active".to_string(),
    })
}

#[tauri::command]
pub async fn stop_port_forward(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut port_forwards = state.port_forwards.lock().await;

    if let Some(session) = port_forwards.get_mut(&id) {
        session.stop();
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

    let forwards: Vec<PortForwardResponse> = port_forwards
        .values()
        .map(|s| PortForwardResponse {
            id: s.id.clone(),
            cluster_id: s.cluster_id.clone(),
            namespace: s.namespace.clone(),
            pod: s.pod.clone(),
            container_port: s.ports.first().copied().unwrap_or(0),
            local_port: s.local_ports.first().copied().unwrap_or(0),
            status: match s.status {
                crate::kube::PortForwardStatus::Active => "Active".to_string(),
                crate::kube::PortForwardStatus::Stopped => "Stopped".to_string(),
                crate::kube::PortForwardStatus::Error(ref e) => e.clone(),
            },
        })
        .collect();

    Ok(forwards)
}

#[tauri::command]
pub async fn delete_port_forward(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut port_forwards = state.port_forwards.lock().await;

    if port_forwards.remove(&id).is_none() {
        return Err(format!("Port forward session {id} not found"));
    }

    Ok(())
}
