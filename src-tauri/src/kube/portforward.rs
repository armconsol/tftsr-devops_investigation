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
    pub kubectl_child: Option<Arc<std::sync::Mutex<tokio::process::Child>>>,
    pub is_stopped: Arc<AtomicBool>,
    pub error_message: Option<String>,
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
            error_message: None,
        }
    }

    pub fn stop(&mut self) {
        self.is_stopped.store(true, Ordering::SeqCst);
        self.status = PortForwardStatus::Stopped;

        if let Some(child_mutex) = &self.kubectl_child {
            let mut child = child_mutex.lock().unwrap();
            // Kill the child process - kill() returns a Future
            // We use std::mem::drop to ignore the Future result since we can't await here
            std::mem::drop(child.kill());
        }
    }

    pub fn set_error(&mut self, error: String) {
        self.status = PortForwardStatus::Error(error.clone());
        self.error_message = Some(error);
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, PortForwardStatus::Active)
    }
}

impl Drop for PortForwardSession {
    fn drop(&mut self) {
        // Only kill if not already stopped
        if self.is_stopped.load(Ordering::SeqCst) {
            return;
        }

        if let Some(child_mutex) = &self.kubectl_child {
            let mut child = child_mutex.lock().unwrap();
            // Kill the child process - kill() returns a Future
            // We use std::mem::drop to ignore the Future result since we can't await here
            std::mem::drop(child.kill());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_forward_session_new() {
        let config = PortForwardSessionConfig {
            id: "pf-1".to_string(),
            cluster_id: "cluster-1".to_string(),
            cluster_name: "Production".to_string(),
            namespace: "default".to_string(),
            pod: "my-pod".to_string(),
            container: None,
            ports: vec![8080],
            local_ports: vec![0],
        };

        let session = PortForwardSession::new(config);

        assert_eq!(session.id, "pf-1");
        assert_eq!(session.cluster_id, "cluster-1");
        assert_eq!(session.cluster_name, "Production");
        assert_eq!(session.namespace, "default");
        assert_eq!(session.pod, "my-pod");
        assert_eq!(session.ports, vec![8080]);
        assert_eq!(session.local_ports, vec![0]);
        assert!(matches!(session.status, PortForwardStatus::Active));
    }

    #[test]
    fn test_port_forward_session_stop() {
        let config = PortForwardSessionConfig {
            id: "pf-2".to_string(),
            cluster_id: "cluster-1".to_string(),
            cluster_name: "Test".to_string(),
            namespace: "default".to_string(),
            pod: "pod-1".to_string(),
            container: None,
            ports: vec![9000],
            local_ports: vec![0],
        };

        let mut session = PortForwardSession::new(config);
        assert!(matches!(session.status, PortForwardStatus::Active));

        session.stop();
        assert!(matches!(session.status, PortForwardStatus::Stopped));
    }

    #[test]
    fn test_port_forward_session_set_error() {
        let config = PortForwardSessionConfig {
            id: "pf-3".to_string(),
            cluster_id: "cluster-1".to_string(),
            cluster_name: "Test".to_string(),
            namespace: "default".to_string(),
            pod: "pod-1".to_string(),
            container: None,
            ports: vec![9000],
            local_ports: vec![0],
        };

        let mut session = PortForwardSession::new(config);
        assert!(matches!(session.status, PortForwardStatus::Active));

        session.set_error("connection refused".to_string());
        assert!(matches!(session.status, PortForwardStatus::Error(_)));
        assert_eq!(
            session.error_message,
            Some("connection refused".to_string())
        );
    }

    #[test]
    fn test_port_forward_session_is_active() {
        // Test Active status
        let config = PortForwardSessionConfig {
            id: "pf-4".to_string(),
            cluster_id: "cluster-1".to_string(),
            cluster_name: "Test".to_string(),
            namespace: "default".to_string(),
            pod: "pod-1".to_string(),
            container: None,
            ports: vec![9000],
            local_ports: vec![0],
        };

        let session = PortForwardSession::new(config);
        assert!(session.is_active());

        // Test Stopped status
        let stopped_session = PortForwardSession {
            id: "pf-5".to_string(),
            cluster_id: "cluster-1".to_string(),
            cluster_name: "Test".to_string(),
            namespace: "default".to_string(),
            pod: "pod-1".to_string(),
            container: None,
            ports: vec![9000],
            local_ports: vec![0],
            status: PortForwardStatus::Stopped,
            kubectl_child: None,
            is_stopped: Arc::new(AtomicBool::new(false)),
            error_message: None,
        };
        assert!(!stopped_session.is_active());

        // Test Error status
        let error_session = PortForwardSession {
            id: "pf-6".to_string(),
            cluster_id: "cluster-1".to_string(),
            cluster_name: "Test".to_string(),
            namespace: "default".to_string(),
            pod: "pod-1".to_string(),
            container: None,
            ports: vec![9000],
            local_ports: vec![0],
            status: PortForwardStatus::Error("error".to_string()),
            kubectl_child: None,
            is_stopped: Arc::new(AtomicBool::new(false)),
            error_message: Some("error".to_string()),
        };
        assert!(!error_session.is_active());
    }
}
