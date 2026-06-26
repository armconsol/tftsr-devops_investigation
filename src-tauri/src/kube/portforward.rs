use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::process::Child;
use tokio::sync::Mutex as TokioMutex;

/// Background task handle for waiting on kubectl child process
pub struct ChildWaitHandle {
    pub join_handle: tokio::task::JoinHandle<()>,
    pub child: Arc<TokioMutex<Option<Child>>>,
}

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
    /// Join handle for the background task waiting on the kubectl child
    pub child_wait_handle: Option<Arc<TokioMutex<ChildWaitHandle>>>,
    pub is_stopped: Arc<AtomicBool>,
    pub error_message: Option<String>,
    pub shared_status: Arc<TokioMutex<PortForwardStatus>>,
    pub shared_error: Arc<TokioMutex<Option<String>>>,
    /// Path to temp kubeconfig file for cleanup
    pub temp_kubeconfig_path: Option<std::path::PathBuf>,
}

#[derive(Clone)]
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
    /// Path to temp kubeconfig file for cleanup
    pub temp_kubeconfig_path: Option<std::path::PathBuf>,
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
            child_wait_handle: None,
            is_stopped: Arc::new(AtomicBool::new(false)),
            error_message: None,
            shared_status: Arc::new(TokioMutex::new(PortForwardStatus::Active)),
            shared_error: Arc::new(TokioMutex::new(None)),
            temp_kubeconfig_path: config.temp_kubeconfig_path,
        }
    }

    /// Spawn a background task to wait on the kubectl child process
    /// and update session state on completion/error
    pub fn spawn_child_waiter(&mut self, child: Child) {
        let is_stopped = self.is_stopped.clone();
        let status_clone = self.shared_status.clone();
        let error_clone = self.shared_error.clone();

        // Store the child in an Arc<Mutex<Option<Child>>> so it can be accessed from the async task
        // and also from the stop() method
        let child_arc = Arc::new(TokioMutex::new(Some(child)));

        let child_for_task = child_arc.clone();
        let temp_path_clone = self.temp_kubeconfig_path.clone();
        let join_handle = tokio::spawn(async move {
            // Take the child from the Arc. If None, stop_async/close already took it and will
            // handle cleanup — nothing left to do here.
            let child_opt = child_for_task.lock().await.take();
            let mut child = match child_opt {
                Some(c) => c,
                None => return,
            };

            // Wait for the child process to complete
            let result = child.wait().await;

            // Clean up temp kubeconfig file after child completes
            if let Some(path) = &temp_path_clone {
                let _ = std::fs::remove_file(path);
            }

            // Only update if not already explicitly stopped
            if !is_stopped.load(Ordering::SeqCst) {
                match result {
                    Ok(status) if status.success() => {
                        *status_clone.lock().await = PortForwardStatus::Stopped;
                    }
                    Ok(status) => {
                        let error_msg = format!("kubectl process exited with status: {}", status);
                        *status_clone.lock().await = PortForwardStatus::Error(error_msg.clone());
                        *error_clone.lock().await = Some(error_msg);
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to wait for kubectl process: {}", e);
                        *status_clone.lock().await = PortForwardStatus::Error(error_msg.clone());
                        *error_clone.lock().await = Some(error_msg);
                    }
                }
            }
        });

        self.child_wait_handle = Some(Arc::new(TokioMutex::new(ChildWaitHandle {
            join_handle,
            child: child_arc,
        })));
    }

    pub fn stop(&mut self) {
        self.is_stopped.store(true, Ordering::SeqCst);
        self.status = PortForwardStatus::Stopped;
        if let Ok(mut s) = self.shared_status.try_lock() {
            *s = PortForwardStatus::Stopped;
        }
        self.child_wait_handle = None;
    }

    pub async fn stop_async(&mut self) {
        self.is_stopped.store(true, Ordering::SeqCst);
        self.status = PortForwardStatus::Stopped;
        *self.shared_status.lock().await = PortForwardStatus::Stopped;

        // Kill the child process if it exists
        if let Some(ref child_wait_handle) = self.child_wait_handle {
            let guard = child_wait_handle.lock().await;
            let child_opt = guard.child.lock().await.take();
            if let Some(mut child) = child_opt {
                let _ = child.kill().await;
            }
        }

        // Clean up the temp kubeconfig file. Taking the child above causes the background
        // task to exit early without reaching its own cleanup branch.
        if let Some(ref path) = self.temp_kubeconfig_path {
            let _ = std::fs::remove_file(path);
        }
    }

    pub async fn close(&mut self) {
        // Kill the child process if it exists
        if let Some(ref child_wait_handle) = self.child_wait_handle {
            let guard = child_wait_handle.lock().await;
            let child_opt = guard.child.lock().await.take();
            if let Some(mut child) = child_opt {
                let _ = child.kill().await;
            }
        }

        // Clean up the temp kubeconfig file. Taking the child above causes the background
        // task to exit early without reaching its own cleanup branch.
        if let Some(ref path) = self.temp_kubeconfig_path {
            let _ = std::fs::remove_file(path);
        }
    }

    pub fn set_error(&mut self, error: String) {
        self.status = PortForwardStatus::Error(error.clone());
        self.error_message = Some(error.clone());
        if let Ok(mut s) = self.shared_status.try_lock() {
            *s = PortForwardStatus::Error(error.clone());
        }
        if let Ok(mut e) = self.shared_error.try_lock() {
            *e = Some(error);
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

        // Drop the handle — detaches the background task. Called from sync context so
        // we cannot await kill(); the Child inside the task will be dropped by the OS.
        self.child_wait_handle = None;

        // Best-effort temp file cleanup on unexpected drop (e.g., panic paths).
        if let Some(ref path) = self.temp_kubeconfig_path {
            let _ = std::fs::remove_file(path);
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
            temp_kubeconfig_path: None,
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
            temp_kubeconfig_path: None,
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
            temp_kubeconfig_path: None,
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
            temp_kubeconfig_path: None,
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
            child_wait_handle: None,
            is_stopped: Arc::new(AtomicBool::new(false)),
            error_message: None,
            shared_status: Arc::new(TokioMutex::new(PortForwardStatus::Stopped)),
            shared_error: Arc::new(TokioMutex::new(None)),
            temp_kubeconfig_path: None,
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
            child_wait_handle: None,
            is_stopped: Arc::new(AtomicBool::new(false)),
            error_message: Some("error".to_string()),
            shared_status: Arc::new(TokioMutex::new(PortForwardStatus::Error(
                "error".to_string(),
            ))),
            shared_error: Arc::new(TokioMutex::new(Some("error".to_string()))),
            temp_kubeconfig_path: None,
        };
        assert!(!error_session.is_active());
    }
}
