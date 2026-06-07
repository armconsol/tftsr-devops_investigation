use crate::state::AppState;
use anyhow::Result;
use tokio::sync::mpsc;
use tracing::info;

pub struct Watcher {
    cluster_id: String,
    namespace: String,
    resource_type: String,
    #[allow(dead_code)]
    tx: mpsc::Sender<serde_json::Value>,
}

impl Watcher {
    pub fn new(
        cluster_id: String,
        namespace: String,
        resource_type: String,
        tx: mpsc::Sender<serde_json::Value>,
    ) -> Self {
        Self {
            cluster_id,
            namespace,
            resource_type,
            tx,
        }
    }

    pub async fn start(self) -> Result<()> {
        info!(
            "Starting watcher for {}/{} in namespace {}",
            self.resource_type, self.cluster_id, self.namespace
        );

        // TODO: implement real watch stream via k8s-openapi + tokio-stream
        tracing::warn!(
            resource_type = %self.resource_type,
            cluster_id = %self.cluster_id,
            namespace = %self.namespace,
            "Watcher is a stub — no events will be emitted until k8s watch stream is implemented"
        );
        Ok(())
    }
}

pub async fn start_resource_watcher(
    _app_state: &AppState,
    cluster_id: String,
    namespace: String,
    resource_type: String,
) -> Result<mpsc::Receiver<serde_json::Value>> {
    let (tx, rx) = mpsc::channel(100);

    let watcher_tx = tx.clone();
    let cluster_id = cluster_id.clone();
    let namespace = namespace.clone();
    let resource_type = resource_type.clone();

    tokio::spawn(async move {
        let watcher = Watcher::new(cluster_id, namespace, resource_type, watcher_tx);
        if let Err(e) = watcher.start().await {
            tracing::error!("Watcher failed: {}", e);
        }
    });

    Ok(rx)
}

pub async fn start_all_resources_watcher(
    _app_state: &AppState,
    cluster_id: String,
) -> Result<mpsc::Receiver<serde_json::Value>> {
    let (tx, rx) = mpsc::channel(100);

    let resources = vec![
        "pods",
        "services",
        "deployments",
        "replicasets",
        "daemonsets",
    ];

    for resource_type in resources {
        let watcher_tx = tx.clone();
        let cluster_id = cluster_id.clone();
        let namespace = "default".to_string();

        tokio::spawn(async move {
            let watcher =
                Watcher::new(cluster_id, namespace, resource_type.to_string(), watcher_tx);
            if let Err(e) = watcher.start().await {
                tracing::error!("Watcher for {} failed: {}", resource_type, e);
            }
        });
    }

    Ok(rx)
}
