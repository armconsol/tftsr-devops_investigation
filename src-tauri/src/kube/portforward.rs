use std::process::{Child, Command, Stdio};
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
    pub kubectl_child: Option<Arc<std::sync::Mutex<Child>>>,
    pub is_stopped: Arc<AtomicBool>,
}

pub enum PortForwardStatus {
    Active,
    Stopped,
    Error(String),
}

impl PortForwardSession {
    pub fn new(
        id: String,
        cluster_id: String,
        cluster_name: String,
        namespace: String,
        pod: String,
        container: Option<String>,
        ports: Vec<u16>,
        local_ports: Vec<u16>,
    ) -> Self {
        Self {
            id,
            cluster_id,
            cluster_name,
            namespace,
            pod,
            container,
            ports,
            local_ports,
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
