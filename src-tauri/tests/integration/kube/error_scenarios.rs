// Error scenarios integration tests
// Tests: invalid kubeconfig, cluster not found, port conflicts, edge cases

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
async fn test_invalid_yaml_syntax() {
    let state = setup_test_state();
    
    let invalid_yaml = r#"
apiVersion: v1
kind: Config
clusters:
  - cluster:
      server: https://k8s.example.com
  invalid: [unclosed array
"#;

    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "Invalid YAML".to_string(),
        invalid_yaml.to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Invalid kubeconfig YAML") || err.contains("YAML"));
}

#[tokio::test]
async fn test_empty_kubeconfig() {
    let state = setup_test_state();
    
    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "Empty".to_string(),
        "".to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[tokio::test]
async fn test_whitespace_only_kubeconfig() {
    let state = setup_test_state();
    
    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "Whitespace".to_string(),
        "   \n\t  \n   ".to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[tokio::test]
async fn test_kubeconfig_with_null_values() {
    let state = setup_test_state();
    
    let kubeconfig = r#"
apiVersion: v1
kind: Config
clusters:
- cluster:
    server: null
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
        "Null Server".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Server URL not found"));
}

#[tokio::test]
async fn test_port_forward_to_nonexistent_cluster() {
    let state = setup_test_state();
    
    let request = trcaa_lib::commands::kube::PortForwardRequest {
        cluster_id: "non-existent-cluster".to_string(),
        namespace: "default".to_string(),
        pod: "nginx-pod".to_string(),
        container_port: 80,
    };

    let result = trcaa_lib::commands::kube::start_port_forward(
        request,
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_stop_nonexistent_port_forward() {
    let state = setup_test_state();
    
    let result = trcaa_lib::commands::kube::stop_port_forward(
        "non-existent-session".to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_delete_nonexistent_port_forward() {
    let state = setup_test_state();
    
    let result = trcaa_lib::commands::kube::delete_port_forward(
        "non-existent-session".to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_remove_nonexistent_cluster() {
    let state = setup_test_state();
    
    let result = trcaa_lib::commands::kube::remove_cluster(
        "non-existent-cluster".to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_kubeconfig_with_empty_clusters_array() {
    let state = setup_test_state();
    
    let kubeconfig = r#"
apiVersion: v1
kind: Config
clusters: []
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
        "Empty Clusters".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No clusters found"));
}

#[tokio::test]
async fn test_kubeconfig_with_empty_contexts_array() {
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
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No contexts found"));
}

#[tokio::test]
async fn test_kubeconfig_missing_api_version() {
    let state = setup_test_state();
    
    let kubeconfig = r#"
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

    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "No API Version".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    // Should still work - we only check for required fields
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_kubeconfig_with_extra_fields() {
    let state = setup_test_state();
    
    let kubeconfig = r#"
apiVersion: v1
kind: Config
metadata:
  name: my-config
  annotations:
    created-by: test
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

    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "With Metadata".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_kubeconfig_with_multiple_clusters() {
    let state = setup_test_state();
    
    // Use first cluster's server URL
    let kubeconfig = r#"
apiVersion: v1
kind: Config
clusters:
- cluster:
    server: https://k8s1.example.com:6443
  name: cluster1
- cluster:
    server: https://k8s2.example.com:6443
  name: cluster2
contexts:
- context:
    cluster: cluster1
    user: admin
  name: context1
users:
- name: admin
  user:
    token: test-token
"#;

    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "Multiple Clusters".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_ok());
    let cluster_info = result.unwrap();
    assert_eq!(cluster_info.cluster_url, "https://k8s1.example.com:6443");
}

#[tokio::test]
async fn test_kubeconfig_with_multiple_contexts() {
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
  name: default-context
- context:
    cluster: production
    user: admin
    namespace: kube-system
  name: kube-system-context
users:
- name: admin
  user:
    token: test-token
"#;

    let result = trcaa_lib::commands::kube::add_cluster(
        "cluster-1".to_string(),
        "Multiple Contexts".to_string(),
        kubeconfig.to_string(),
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_ok());
    let cluster_info = result.unwrap();
    // Should use first context
    assert_eq!(cluster_info.context, "default-context");
}

#[tokio::test]
async fn test_port_forward_with_empty_namespace() {
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

    // Try port forward with empty namespace
    let request = trcaa_lib::commands::kube::PortForwardRequest {
        cluster_id: "cluster-1".to_string(),
        namespace: "".to_string(),
        pod: "nginx-pod".to_string(),
        container_port: 80,
    };

    // Note: Current implementation doesn't validate namespace/pod
    // This may need validation added
    let result = trcaa_lib::commands::kube::start_port_forward(
        request,
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_ok()); // Current behavior allows empty namespace
}

#[tokio::test]
async fn test_port_forward_with_empty_pod() {
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

    // Try port forward with empty pod
    let request = trcaa_lib::commands::kube::PortForwardRequest {
        cluster_id: "cluster-1".to_string(),
        namespace: "default".to_string(),
        pod: "".to_string(),
        container_port: 80,
    };

    // Note: Current implementation doesn't validate pod name
    let result = trcaa_lib::commands::kube::start_port_forward(
        request,
        trcaa_lib::State::new(&state),
    ).await;

    assert!(result.is_ok()); // Current behavior allows empty pod
}
