use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct PortForwardSession {
    pub id: String,
    pub cluster_id: String,
    pub cluster_name: String,
    pub namespace: String,
    pub pod: String,
    pub container: Option<String>,
    pub ports: Vec<u16>,
    pub local_ports: Vec<u16>,
    pub status: PortForwardStatus,
    pub kubectl_child: Option<Arc<std::sync::Mutex<std::process::Child>>>,
    pub is_stopped: Arc<AtomicBool>,
}

pub enum PortForwardStatus {
    Active,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PortForwardSessionConfig {
    pub id: String,
    pub cluster_id: String,
    pub cluster_name: String,
    pub namespace: String,
    pub pod: String,
    pub container: Option<String>,
    pub ports: Vec<u16>,
    pub local_ports: Vec<u16>,
}

impl PortForwardSession {
    pub fn new(config: PortForwardSessionConfig) -> Self {
        Self {
            id: config.id,
            cluster_id: config.cluster_id,
            cluster_name: config.cluster_name,
            namespace: config.namespace,
            pod: config.pod,
            container: config.container,
            ports: config.ports,
            local_ports: config.local_ports,
            status: PortForwardStatus::Active,
            kubectl_child: None,
            is_stopped: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn stop(&mut self) {
        self.is_stopped.store(true, Ordering::SeqCst);
        self.status = PortForwardStatus::Stopped;

        if let Some(child_mutex) = &self.kubectl_child {
            let mut child = child_mutex.lock().unwrap();
            let _ = child.kill();
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, PortForwardStatus::Active)
    }
}

impl Drop for PortForwardSession {
    fn drop(&mut self) {
        if self.is_stopped.load(Ordering::SeqCst) {
            return;
        }

        if let Some(child_mutex) = &self.kubectl_child {
            let mut child = child_mutex.lock().unwrap();
            let _ = child.kill();
        }
    }
}
