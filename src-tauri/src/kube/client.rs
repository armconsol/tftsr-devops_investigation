use std::sync::Arc;

pub struct ClusterClient {
    pub id: String,
    pub name: String,
    pub context: String,
    pub server_url: String,
    pub kubeconfig_content: Arc<String>,
}

impl ClusterClient {
    pub fn new(id: String, name: String, context: String, server_url: String, kubeconfig_content: Arc<String>) -> Self {
        Self {
            id,
            name,
            context,
            server_url,
            kubeconfig_content,
        }
    }
}
