use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Cluster {
    pub id: String,
    pub name: String,
    pub context: String,
    pub server_url: Option<String>,
    pub kubeconfig_id: String,
    pub created_at: String,
}

impl Cluster {
    pub fn new(
        id: String,
        name: String,
        context: String,
        server_url: Option<String>,
        kubeconfig_id: String,
        created_at: String,
    ) -> Self {
        Self {
            id,
            name,
            context,
            server_url,
            kubeconfig_id,
            created_at,
        }
    }
}

pub struct ClusterClient {
    pub id: String,
    pub name: String,
    pub context: String,
    pub server_url: String,
    pub kubeconfig_content: Arc<String>,
}

impl ClusterClient {
    pub fn new(
        id: String,
        name: String,
        context: String,
        server_url: String,
        kubeconfig_content: Arc<String>,
    ) -> Self {
        Self {
            id,
            name,
            context,
            server_url,
            kubeconfig_content,
        }
    }
}
