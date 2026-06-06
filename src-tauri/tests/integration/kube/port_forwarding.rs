// Port forwarding integration tests
// Tests: start port forward, list port forwards, stop port forward, delete port forward

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
async fn test_start_port_forward_success() {
    let state = setup_test_state();
    
    // Add a cluster first
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
        pod: "nginx-pod-abc123".to_string(),
        container_port: 80,
    };

    let result = trcaa_lib::commands::kube::start_port_forward(
        request,
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.id.len() > 0);
    assert_eq!(response.cluster_id, "cluster-1");
    assert_eq!(response.namespace, "default");
    assert_eq!(response.pod, "nginx-pod-abc123");
    assert_eq!(response.container_port, 80);
    assert_eq!(response.status, "Active");
}

#[tokio::test]
async fn test_start_port_forward_cluster_not_found() {
    let state = setup_test_state();
    
    let request = trcaa_lib::commands::kube::PortForwardRequest {
        cluster_id: "non-existent".to_string(),
        namespace: "default".to_string(),
        pod: "nginx-pod".to_string(),
        container_port: 80,
    };

    let result = trcaa_lib::commands::kube::start_port_forward(
        request,
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cluster non-existent not found"));
}

#[tokio::test]
async fn test_list_port_forwards_empty() {
    let state = setup_test_state();
    
    let result = trcaa_lib::commands::kube::list_port_forwards(
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_ok());
    let forwards = result.unwrap();
    assert!(forwards.is_empty());
}

#[tokio::test]
async fn test_list_port_forwards_multiple() {
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
    ).await.unwrap();

    // Start first port forward
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

    // Start second port forward
    let request2 = trcaa_lib::commands::kube::PortForwardRequest {
        cluster_id: "cluster-1".to_string(),
        namespace: "kube-system".to_string(),
        pod: "pod-2".to_string(),
        container_port: 443,
    };
    
    trcaa_lib::commands::kube::start_port_forward(
        request2,
        trcaa_lib::State::new(&state),
    ).await.unwrap();

    // List port forwards
    let result = trcaa_lib::commands::kube::list_port_forwards(
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_ok());
    let forwards = result.unwrap();
    assert_eq!(forwards.len(), 2);
    
    let pods: Vec<&str> = forwards.iter().map(|f| f.pod.as_str()).collect();
    assert!(pods.contains(&"pod-1"));
    assert!(pods.contains(&"pod-2"));
}

#[tokio::test]
async fn test_stop_port_forward_success() {
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

    // Verify it's active
    let list_result = trcaa_lib::commands::kube::list_port_forwards(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert_eq!(list_result[0].status, "Active");

    // Stop port forward
    let result = trcaa_lib::commands::kube::stop_port_forward(
        start_result.id.clone(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_ok());

    // Verify it's stopped
    let list_result = trcaa_lib::commands::kube::list_port_forwards(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert_eq!(list_result[0].status, "Stopped");
}

#[tokio::test]
async fn test_stop_port_forward_not_found() {
    let state = setup_test_state();
    
    let result = trcaa_lib::commands::kube::stop_port_forward(
        "non-existent".to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Port forward session non-existent not found"));
}

#[tokio::test]
async fn test_delete_port_forward_success() {
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

    // Verify port forward exists
    let list_result = trcaa_lib::commands::kube::list_port_forwards(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert_eq!(list_result.len(), 1);

    // Delete port forward
    let result = trcaa_lib::commands::kube::delete_port_forward(
        start_result.id.clone(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_ok());

    // Verify port forward is gone
    let list_result = trcaa_lib::commands::kube::list_port_forwards(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert!(list_result.is_empty());
}

#[tokio::test]
async fn test_delete_port_forward_not_found() {
    let state = setup_test_state();
    
    let result = trcaa_lib::commands::kube::delete_port_forward(
        "non-existent".to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Port forward session non-existent not found"));
}

#[tokio::test]
async fn test_port_forward_session_lifecycle() {
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

    // Verify session is active
    let session_id = start_result.id.clone();
    let list_result = trcaa_lib::commands::kube::list_port_forwards(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert_eq!(list_result[0].id, session_id);
    assert_eq!(list_result[0].status, "Active");

    // Stop port forward
    trcaa_lib::commands::kube::stop_port_forward(
        session_id.clone(),
        trcaa_lib::State::new(&state),
    ).await.unwrap();

    // Verify session is stopped
    let list_result = trcaa_lib::commands::kube::list_port_forwards(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert_eq!(list_result[0].status, "Stopped");

    // Delete port forward
    trcaa_lib::commands::kube::delete_port_forward(
        session_id.clone(),
        trcaa_lib::State::new(&state),
    ).await.unwrap();

    // Verify session is deleted
    let list_result = trcaa_lib::commands::kube::list_port_forwards(
        trcaa_lib::State::new(&state),
    ).await.unwrap();
    assert!(list_result.is_empty());
}
