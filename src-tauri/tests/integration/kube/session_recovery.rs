// Session recovery integration tests
// Tests: cluster and port forward persistence across restarts

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

fn setup_test_state() -> trcaa_lib::state::AppState {
    let conn = rusqlite::Connection::open_in_memory().expect("Failed to create in-memory DB");
    
    trcaa_lib::state::AppState {
        db: Arc::new(Mutex::new(conn)),
        settings: Arc::new(Mutex::new(trcaa_lib::state::AppSettings::default())),
        app_data_dir: std::path::PathBuf::from("./test-data"),
        integration_webviews: Arc::new(Mutex::new(HashMap::new())),
        mcp_connections: Arc::new(Mutex::new(HashMap::new())),
        pending_approvals: Arc::new(Mutex::new(HashMap::new())),
        clusters: Arc::new(Mutex::new(HashMap::new())),
        port_forwards: Arc::new(Mutex::new(HashMap::new())),
        refresh_registry: Arc::new(Mutex::new(trcaa_lib::kube::RefreshRegistry::new())),
    }
}

#[tokio::test]
async fn test_clusters_persist_in_memory() {
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
  name: prod-context
users:
- name: admin
  user:
    token: test-token
"#;

    // Add cluster
    trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "Production".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    ).await.unwrap();

    // List clusters - should find it
    let clusters = trcaa_lib::commands::kube::list_clusters(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert_eq!(clusters.len(), 1);
    
    // Note: In-memory state doesn't persist across restarts
    // This test documents the current in-memory behavior
    // For true persistence, database storage would be required
}

#[tokio::test]
async fn test_port_forwards_persist_in_memory() {
    let state = setup_test_state();
    
    // Add cluster
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
    ).await.unwrap();

    // Start port forward
    let request = trcaa_lib::commands::kube::PortForwardRequest {
        cluster_id: "cluster-1".to_string(),
        namespace: "default".to_string(),
        pod: "nginx-pod".to_string(),
        container_port: 80,
    };
    
    trcaa_lib::commands::kube::start_port_forward(
        request,
        trcaa_lib::State::new(&state),
    ).await.unwrap();

    // List port forwards - should find it
    let forwards = trcaa_lib::commands::kube::list_port_forwards(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert_eq!(forwards.len(), 1);
    
    // Note: In-memory state doesn't persist across restarts
    // For true persistence, database storage would be required
}

#[tokio::test]
async fn test_multiple_clusters_and_port_forwards() {
    let state = setup_test_state();
    
    // Add multiple clusters
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
    user: admin
  name: context1
users:
- name: admin
  user:
    token: token1
"#;
    
    trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "Cluster 1".to_string(),
        kubeconfig1.to_string(),
        trcaa_lib::State::new(&state),
    ).await.unwrap();

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
    user: admin
  name: context2
users:
- name: admin
  user:
    token: token2
"#;
    
    trcaa_lib::commands::kube::add_cluster(
        "cluster-2".to_string(),
        "Cluster 2".to_string(),
        kubeconfig2.to_string(),
        trcaa_lib::State::new(&state),
    ).await.unwrap();

    // Start multiple port forwards
    let request1 = trcaa_lib::commands::kube::PortForwardRequest {
        cluster_id: "cluster-1".to_string(),
        namespace: "default".to_string(),
        pod: "pod-1".to_string(),
        container_port: 80,
    };
    
    trcaa_lib::commands::kube::start_port_forward(
        request1,
        trcaa_lib::State::new(&state),
    ).await.unwrap();

    let request2 = trcaa_lib::commands::kube::PortForwardRequest {
        cluster_id: "cluster-2".to_string(),
        namespace: "kube-system".to_string(),
        pod: "pod-2".to_string(),
        container_port: 443,
    };
    
    trcaa_lib::commands::kube::start_port_forward(
        request2,
        trcaa_lib::State::new(&state),
    ).await.unwrap();

    // Verify all clusters exist
    let clusters = trcaa_lib::commands::kube::list_clusters(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert_eq!(clusters.len(), 2);

    // Verify all port forwards exist
    let forwards = trcaa_lib::commands::kube::list_port_forwards(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert_eq!(forwards.len(), 2);
}

#[tokio::test]
async fn test_cluster_removal_clears_cluster_data() {
    let state = setup_test_state();
    
    // Add cluster
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
    ).await.unwrap();

    // Verify cluster exists
    let clusters = trcaa_lib::commands::kube::list_clusters(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert_eq!(clusters.len(), 1);

    // Remove cluster
    trcaa_lib::commands::kube::remove_cluster(
        "cluster-1".to_string(),
        trcaa_lib::State::new(&state),
    ).await.unwrap();

    // Verify cluster is gone
    let clusters = trcaa_lib::commands::kube::list_clusters(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert!(clusters.is_empty());
}

#[tokio::test]
async fn test_port_forward_stop_clears_session() {
    let state = setup_test_state();
    
    // Add cluster
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
    ).await.unwrap();

    // Start port forward
    let request = trcaa_lib::commands::kube::PortForwardRequest {
        cluster_id: "cluster-1".to_string(),
        namespace: "default".to_string(),
        pod: "nginx-pod".to_string(),
        container_port: 80,
    };
    
    let start_result = trcaa_lib::commands::kube::start_port_forward(
        request,
        trcaa_lib::State::new(&state),
    ).await.unwrap();

    // Stop port forward
    trcaa_lib::commands::kube::stop_port_forward(
        start_result.id.clone(),
        trcaa_lib::State::new(&state),
    ).await.unwrap();

    // Verify session is stopped (not deleted)
    let forwards = trcaa_lib::commands::kube::list_port_forwards(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert_eq!(forwards.len(), 1);
    assert_eq!(forwards[0].status, "Stopped");
}

#[tokio::test]
async fn test_port_forward_delete_removes_session() {
    let state = setup_test_state();
    
    // Add cluster
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
    ).await.unwrap();

    // Start port forward
    let request = trcaa_lib::commands::kube::PortForwardRequest {
        cluster_id: "cluster-1".to_string(),
        namespace: "default".to_string(),
        pod: "nginx-pod".to_string(),
        container_port: 80,
    };
    
    let start_result = trcaa_lib::commands::kube::start_port_forward(
        request,
        trcaa_lib::State::new(&state),
    ).await.unwrap();

    // Delete port forward
    trcaa_lib::commands::kube::delete_port_forward(
        start_result.id.clone(),
        trcaa_lib::State::new(&state),
    ).await.unwrap();

    // Verify session is deleted
    let forwards = trcaa_lib::commands::kube::list_port_forwards(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert!(forwards.is_empty());
}
