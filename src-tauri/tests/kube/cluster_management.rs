// Cluster management integration tests
// Tests: add cluster, list clusters, remove cluster

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex as TokioMutex;

fn setup_test_state() -> trcaa_lib::state::AppState {
    let conn = rusqlite::Connection::open_in_memory().expect("Failed to create in-memory DB");

    trcaa_lib::state::AppState {
        db: Arc::new(StdMutex::new(conn)),
        settings: Arc::new(StdMutex::new(trcaa_lib::state::AppSettings::default())),
        app_data_dir: std::path::PathBuf::from("./test-data"),
        integration_webviews: Arc::new(StdMutex::new(HashMap::new())),
        mcp_connections: Arc::new(TokioMutex::new(HashMap::new())),
        pending_approvals: Arc::new(TokioMutex::new(HashMap::new())),
        clusters: Arc::new(TokioMutex::new(HashMap::new())),
        port_forwards: Arc::new(TokioMutex::new(HashMap::new())),
        refresh_registry: Arc::new(TokioMutex::new(trcaa_lib::kube::RefreshRegistry::new())),
    }
}

#[tokio::test]
async fn test_add_cluster_success() {
    let state = setup_test_state();

    let kubeconfig = r#"
apiVersion: v1
kind: Config
clusters:
- cluster:
    server: https://k8s.example.com:6443
  name: production
contexts:
- context:
    cluster: production
    user: admin
    namespace: default
  name: production-context
current-context: production-context
users:
- name: admin
  user:
    token: test-token
"#;

    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "Production Cluster".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    )
    .await;

    assert!(result.is_ok());
    let cluster_info = result.unwrap();
    assert_eq!(cluster_info.id, "cluster-1");
    assert_eq!(cluster_info.name, "Production Cluster");
    assert_eq!(cluster_info.context, "production-context");
    assert_eq!(cluster_info.cluster_url, "https://k8s.example.com:6443");
}

#[tokio::test]
async fn test_add_cluster_empty_content() {
    let state = setup_test_state();

    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "Empty Cluster".to_string(),
        "".to_string(),
        trcaa_lib::State::new(&state),
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Kubeconfig content cannot be empty"));
}

#[tokio::test]
async fn test_add_cluster_missing_contexts() {
    let state = setup_test_state();

    let kubeconfig = r#"
apiVersion: v1
kind: Config
clusters:
- cluster:
    server: https://k8s.example.com:6443
  name: production
users:
- name: admin
  user:
    token: test-token
"#;

    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "No Contexts".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Missing 'contexts' field"));
}

#[tokio::test]
async fn test_add_cluster_no_contexts() {
    let state = setup_test_state();

    let kubeconfig = r#"
apiVersion: v1
kind: Config
clusters:
- cluster:
    server: https://k8s.example.com:6443
  name: production
contexts: []
users:
- name: admin
  user:
    token: test-token
"#;

    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "Empty Contexts".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No contexts found"));
}

#[tokio::test]
async fn test_add_cluster_missing_clusters() {
    let state = setup_test_state();

    let kubeconfig = r#"
apiVersion: v1
kind: Config
contexts:
- context:
    cluster: production
    user: admin
  name: production-context
users:
- name: admin
  user:
    token: test-token
"#;

    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "No Clusters".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Missing 'clusters' field"));
}

#[tokio::test]
async fn test_add_cluster_invalid_yaml() {
    let state = setup_test_state();

    let kubeconfig = r#"
apiVersion: v1
kind: Config
invalid yaml here: [
  missing closing bracket
"#;

    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "Invalid YAML".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid kubeconfig YAML"));
}

#[tokio::test]
async fn test_list_clusters_empty() {
    let state = setup_test_state();

    let result = trcaa_lib::commands::kube::list_clusters(trcaa_lib::State::new(&state)).await;

    assert!(result.is_ok());
    let clusters = result.unwrap();
    assert!(clusters.is_empty());
}

#[tokio::test]
async fn test_list_clusters_multiple() {
    let state = setup_test_state();

    // Add first cluster
    let kubeconfig1 = r#"
apiVersion: v1
kind: Config
clusters:
- cluster:
    server: https://k8s1.example.com:6443
  name: cluster1
contexts:
- context:
    cluster: cluster1
    user: user1
  name: context1
users:
- name: user1
  user:
    token: token1
"#;

    trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "Cluster 1".to_string(),
        kubeconfig1.to_string(),
        trcaa_lib::State::new(&state),
    )
    .await
    .unwrap();

    // Add second cluster
    let kubeconfig2 = r#"
apiVersion: v1
kind: Config
clusters:
- cluster:
    server: https://k8s2.example.com:6443
  name: cluster2
contexts:
- context:
    cluster: cluster2
    user: user2
  name: context2
users:
- name: user2
  user:
    token: token2
"#;

    trcaa_lib::commands::kube::add_cluster(
        "cluster-2".to_string(),
        "Cluster 2".to_string(),
        kubeconfig2.to_string(),
        trcaa_lib::State::new(&state),
    )
    .await
    .unwrap();

    // List clusters
    let result = trcaa_lib::commands::kube::list_clusters(trcaa_lib::State::new(&state)).await;

    assert!(result.is_ok());
    let clusters = result.unwrap();
    assert_eq!(clusters.len(), 2);

    let cluster_names: Vec<&str> = clusters.iter().map(|c| c.name.as_str()).collect();
    assert!(cluster_names.contains(&"Cluster 1"));
    assert!(cluster_names.contains(&"Cluster 2"));
}

#[tokio::test]
async fn test_remove_cluster_success() {
    let state = setup_test_state();

    // Add a cluster
    let kubeconfig = r#"
apiVersion: v1
kind: Config
clusters:
- cluster:
    server: https://k8s.example.com:6443
  name: production
contexts:
- context:
    cluster: production
    user: admin
  name: prod-context
users:
- name: admin
  user:
    token: test-token
"#;

    trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "Production".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    )
    .await
    .unwrap();

    // Verify cluster exists
    let clusters = trcaa_lib::commands::kube::list_clusters(trcaa_lib::State::new(&state))
        .await
        .unwrap();
    assert_eq!(clusters.len(), 1);

    // Remove cluster
    let result = trcaa_lib::commands::kube::remove_cluster(
        "cluster-1".to_string(),
        trcaa_lib::State::new(&state),
    )
    .await;

    assert!(result.is_ok());

    // Verify cluster is gone
    let clusters = trcaa_lib::commands::kube::list_clusters(trcaa_lib::State::new(&state))
        .await
        .unwrap();
    assert!(clusters.is_empty());
}

#[tokio::test]
async fn test_remove_cluster_not_found() {
    let state = setup_test_state();

    let result = trcaa_lib::commands::kube::remove_cluster(
        "non-existent".to_string(),
        trcaa_lib::State::new(&state),
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Cluster non-existent not found"));
}

#[tokio::test]
async fn test_add_cluster_with_no_server_url() {
    let state = setup_test_state();

    let kubeconfig = r#"
apiVersion: v1
kind: Config
clusters:
- cluster:
    # No server URL
  name: production
contexts:
- context:
    cluster: production
    user: admin
  name: prod-context
users:
- name: admin
  user:
    token: test-token
"#;

    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "No Server".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Server URL not found"));
}
