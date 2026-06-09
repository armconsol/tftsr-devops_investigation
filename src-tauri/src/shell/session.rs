// PTY Session Management
//
// This module manages the lifecycle of PTY sessions, providing:
// - Session creation and tracking
// - Bidirectional I/O streaming via Tauri events
// - Session cleanup and resource management
//
// Each session has a unique ID and runs in a background tokio task that:
// 1. Continuously reads from PTY stdout/stderr
// 2. Emits data to frontend via Tauri events
// 3. Monitors session liveness
// 4. Cleans up on exit or error

use crate::shell::pty::PtySession;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Session metadata and control
pub struct SessionInfo {
    pub id: String,
    pub cluster_id: String,
    pub namespace: String,
    pub pod: String,
    pub container: Option<String>,
    pub session_type: SessionType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Channel to send stdin data to the session task
    pub stdin_tx: mpsc::UnboundedSender<Vec<u8>>,
    /// Channel to send control commands
    pub control_tx: mpsc::UnboundedSender<ControlCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Exec,
    Attach,
}

#[derive(Debug)]
pub enum ControlCommand {
    Resize { rows: u16, cols: u16 },
    Terminate,
}

/// Parameters for starting a session
pub struct SessionParams {
    pub cluster_id: String,
    pub namespace: String,
    pub pod: String,
    pub container: Option<String>,
    pub kubectl_path: String,
    pub kubeconfig_path: Option<String>,
}

/// Global session registry
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a new kubectl exec session
    pub async fn start_exec_session(
        &self,
        app_handle: AppHandle,
        params: SessionParams,
    ) -> Result<String> {
        let session_id = Uuid::now_v7().to_string();

        // Spawn PTY session
        let pty_session = PtySession::spawn_kubectl_exec(
            &params.kubectl_path,
            &params.namespace,
            &params.pod,
            params.container.as_deref(),
            params.kubeconfig_path.as_deref(),
        )
        .context("Failed to spawn kubectl exec session")?;

        self.register_session(
            app_handle,
            session_id.clone(),
            params,
            SessionType::Exec,
            pty_session,
        )
        .await?;

        Ok(session_id)
    }

    /// Start a new kubectl attach session
    pub async fn start_attach_session(
        &self,
        app_handle: AppHandle,
        params: SessionParams,
    ) -> Result<String> {
        let session_id = Uuid::now_v7().to_string();

        // Spawn PTY session
        let pty_session = PtySession::spawn_kubectl_attach(
            &params.kubectl_path,
            &params.namespace,
            &params.pod,
            params.container.as_deref(),
            params.kubeconfig_path.as_deref(),
        )
        .context("Failed to spawn kubectl attach session")?;

        self.register_session(
            app_handle,
            session_id.clone(),
            params,
            SessionType::Attach,
            pty_session,
        )
        .await?;

        Ok(session_id)
    }

    /// Register and start managing a PTY session
    async fn register_session(
        &self,
        app_handle: AppHandle,
        session_id: String,
        params: SessionParams,
        session_type: SessionType,
        pty_session: PtySession,
    ) -> Result<()> {
        let (stdin_tx, stdin_rx) = mpsc::unbounded_channel();
        let (control_tx, control_rx) = mpsc::unbounded_channel();

        let info = SessionInfo {
            id: session_id.clone(),
            cluster_id: params.cluster_id,
            namespace: params.namespace,
            pod: params.pod,
            container: params.container,
            session_type,
            created_at: chrono::Utc::now(),
            stdin_tx,
            control_tx,
        };

        // Add to registry
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), info);
        }

        // Spawn session I/O task
        let sessions_clone = self.sessions.clone();
        let session_id_clone = session_id.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_session_io(
                app_handle,
                session_id_clone.clone(),
                pty_session,
                stdin_rx,
                control_rx,
            )
            .await
            {
                error!("Session {} I/O task failed: {}", session_id_clone, e);
            }

            // Remove from registry on exit
            let mut sessions = sessions_clone.write().await;
            sessions.remove(&session_id_clone);
            info!("Session {} removed from registry", session_id_clone);
        });

        info!("Session {} started: {:?}", session_id, session_type);
        Ok(())
    }

    /// Main I/O loop for a session
    async fn run_session_io(
        app_handle: AppHandle,
        session_id: String,
        mut pty_session: PtySession,
        mut stdin_rx: mpsc::UnboundedReceiver<Vec<u8>>,
        mut control_rx: mpsc::UnboundedReceiver<ControlCommand>,
    ) -> Result<()> {
        let mut poll_interval = interval(Duration::from_millis(50));

        loop {
            tokio::select! {
                // Read from PTY stdout/stderr
                _ = poll_interval.tick() => {
                    if !pty_session.is_alive() {
                        debug!("Session {} PTY process exited", session_id);
                        let _ = app_handle.emit(&format!("terminal-closed-{}", session_id), ());
                        break;
                    }

                    match pty_session.read() {
                        Ok(data) if !data.is_empty() => {
                            // Emit to frontend
                            if let Err(e) = app_handle.emit(&format!("terminal-output-{}", session_id), data) {
                                warn!("Failed to emit terminal output for session {}: {}", session_id, e);
                            }
                        }
                        Ok(_) => {
                            // No data available
                        }
                        Err(e) => {
                            error!("Failed to read from PTY for session {}: {}", session_id, e);
                            let _ = app_handle.emit(&format!("terminal-error-{}", session_id), e.to_string());
                            break;
                        }
                    }
                }

                // Handle stdin from frontend
                Some(data) = stdin_rx.recv() => {
                    if let Err(e) = pty_session.write(&data) {
                        error!("Failed to write to PTY for session {}: {}", session_id, e);
                        let _ = app_handle.emit(&format!("terminal-error-{}", session_id), e.to_string());
                        break;
                    }
                }

                // Handle control commands
                Some(cmd) = control_rx.recv() => {
                    match cmd {
                        ControlCommand::Resize { rows, cols } => {
                            if let Err(e) = pty_session.resize(rows, cols) {
                                warn!("Failed to resize PTY for session {}: {}", session_id, e);
                            }
                        }
                        ControlCommand::Terminate => {
                            info!("Session {} received terminate command", session_id);
                            let _ = pty_session.kill();
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Send stdin data to a session
    pub async fn send_stdin(&self, session_id: &str, data: Vec<u8>) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .context("Session not found")?;

        session
            .stdin_tx
            .send(data)
            .context("Failed to send stdin data to session task")?;

        Ok(())
    }

    /// Resize a session's PTY
    pub async fn resize_session(&self, session_id: &str, rows: u16, cols: u16) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .context("Session not found")?;

        session
            .control_tx
            .send(ControlCommand::Resize { rows, cols })
            .context("Failed to send resize command to session task")?;

        Ok(())
    }

    /// Terminate a session
    pub async fn terminate_session(&self, session_id: &str) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .context("Session not found")?;

        session
            .control_tx
            .send(ControlCommand::Terminate)
            .context("Failed to send terminate command to session task")?;

        Ok(())
    }

    /// List all active sessions
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Get session info
    pub async fn get_session(&self, session_id: &str) -> Option<SessionInfo> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }
}

impl Clone for SessionInfo {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            cluster_id: self.cluster_id.clone(),
            namespace: self.namespace.clone(),
            pod: self.pod.clone(),
            container: self.container.clone(),
            session_type: self.session_type,
            created_at: self.created_at,
            stdin_tx: self.stdin_tx.clone(),
            control_tx: self.control_tx.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let manager = SessionManager::new();
        let sessions = manager.list_sessions().await;
        assert_eq!(sessions.len(), 0, "New manager should have no sessions");
    }

    #[test]
    fn test_session_type_equality() {
        assert_eq!(SessionType::Exec, SessionType::Exec);
        assert_eq!(SessionType::Attach, SessionType::Attach);
        assert_ne!(SessionType::Exec, SessionType::Attach);
    }

    #[test]
    fn test_control_command_debug() {
        let cmd = ControlCommand::Resize { rows: 24, cols: 80 };
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Resize"));
    }
}
