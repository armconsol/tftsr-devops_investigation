pub mod client;
pub mod portforward;
pub mod refresh;

pub use client::ClusterClient;
pub use portforward::{PortForwardSession, PortForwardStatus};
pub use refresh::RefreshRegistry;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_cluster_client_new() {
        let content = Arc::new("kubeconfig-content".to_string());
        let client = ClusterClient::new(
            "cluster-1".to_string(),
            "Production".to_string(),
            "prod-context".to_string(),
            "https://k8s.example.com".to_string(),
            content,
        );

        assert_eq!(client.id, "cluster-1");
        assert_eq!(client.name, "Production");
        assert_eq!(client.context, "prod-context");
        assert_eq!(client.server_url, "https://k8s.example.com");
    }
}
