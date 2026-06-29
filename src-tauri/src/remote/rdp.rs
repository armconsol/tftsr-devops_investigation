//! RDP Connection Management
//!
//! Provides RDP connection handling with SSH tunnel support and WebSocket streaming.
//! Uses IronRDP for the actual RDP protocol implementation.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};
use uuid::Uuid;

use super::rdp_client::{RdpConfig, RdpConnectionHandler, ResizeRequest};
use super::ssh_tunnel::SshTunnelConfig;
use super::websocket_server::{RdpFrame, WebSocketServer};
use crate::db::models::RemoteConnection;

/// Public RDP session info for API responses (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdpSession {
    pub id: String,
    pub connection_id: String,
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub resolution: String,
    pub color_depth: u32,
    pub websocket_port: u16,
    pub websocket_url: String,
    pub connected: bool,
    pub ssh_enabled: bool,
}

/// Internal RDP session state
#[derive(Clone)]
pub struct RdpSessionInternal {
    pub id: String,
    pub connection_id: String,
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub resolution: String,
    pub color_depth: u32,
    pub websocket_port: u16,
    pub ssh_config: Option<SshTunnelConfig>,
    pub connected: bool,
    /// Sender end of the resize channel; present while the session is running.
    pub resize_tx: Option<tokio::sync::mpsc::Sender<ResizeRequest>>,
}

/// RDP manager for handling multiple RDP sessions
#[derive(Clone)]
pub struct RdpManager {
    sessions: Arc<Mutex<HashMap<String, RdpSessionInternal>>>,
    pub websocket_server: Arc<WebSocketServer>,
    rdp_handler: Arc<RdpConnectionHandler>,
}

impl RdpManager {
    /// Create a new RDP manager
    pub fn new() -> Self {
        let websocket_server = Arc::new(WebSocketServer::new());
        let rdp_handler = RdpConnectionHandler::new(websocket_server.clone());

        RdpManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            websocket_server,
            rdp_handler: Arc::new(rdp_handler),
        }
    }

    /// Create a new RDP session
    ///
    /// `ssh_password`, `ssh_private_key`, and `ssh_key_passphrase` are the
    /// decrypted SSH credentials fetched from the credentials store before
    /// calling this method. All three may be `None` when SSH is disabled or
    /// no SSH credentials have been saved for the connection.
    pub fn create_session(
        &self,
        connection: &RemoteConnection,
        _password: &str,
        ssh_password: Option<String>,
        ssh_private_key: Option<String>,
        ssh_key_passphrase: Option<String>,
    ) -> Result<RdpSessionInternal> {
        let session_id = Uuid::now_v7().to_string();

        // Create SSH config if needed
        let ssh_config = if connection.ssh_enabled {
            Some(SshTunnelConfig {
                hostname: connection.ssh_hostname.as_ref().unwrap().clone(),
                port: connection.ssh_port.unwrap_or(22),
                username: connection.ssh_username.clone().unwrap_or_default(),
                password: ssh_password,
                private_key_path: None,
                private_key_data: ssh_private_key,
                key_passphrase: ssh_key_passphrase,
            })
        } else {
            None
        };

        // Get a free port for WebSocket
        let websocket_port = self.get_free_port()?;

        let session = RdpSessionInternal {
            id: session_id.clone(),
            connection_id: connection.id.clone(),
            hostname: connection.hostname.clone(),
            port: connection.port,
            username: connection.username.clone().unwrap_or_default(),
            resolution: connection.resolution.clone(),
            color_depth: connection.color_depth,
            websocket_port,
            ssh_config,
            connected: false,
            resize_tx: None,
        };

        // Store the session
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id.clone(), session.clone());

        info!(
            "Created RDP session: {} for connection: {}",
            session_id, connection.id
        );
        Ok(session)
    }

    /// Start the RDP session (async version that actually connects)
    pub async fn start_session_async(&self, session_id: &str, password: &str) -> Result<String> {
        let (
            _connection_id,
            hostname,
            port,
            username,
            resolution,
            color_depth,
            _ssh_enabled,
            ssh_config,
        ) = {
            let sessions = self.sessions.lock().unwrap();
            let session = sessions
                .get(session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

            (
                session.connection_id.clone(),
                session.hostname.clone(),
                session.port,
                session.username.clone(),
                session.resolution.clone(),
                session.color_depth,
                session.ssh_config.is_some(),
                session.ssh_config.clone(),
            )
        };

        // Parse resolution
        let (width, height) = Self::parse_resolution(&resolution);

        let rdp_config = RdpConfig {
            host: hostname.clone(),
            port,
            username,
            password: password.to_string(),
            width,
            height,
            bit_depth: color_depth,
            domain: None,
            ssh_config: ssh_config.clone(),
        };

        // Start the WebSocket listener for this session and register the canonical
        // session id so the parked frame receiver is routed to the browser client
        // that connects to `ws://.../rdp/{session_id}`.
        let websocket_port = self.websocket_server.start_random_port().await?;
        self.websocket_server
            .register_session(session_id, session_id)
            .await;

        // Build the RdpClientSession bound to the canonical session id so its
        // frame-forwarding task sends frames keyed by that id.
        let rdp_session = self
            .rdp_handler
            .create_session_with_id(session_id.to_string(), rdp_config.clone())?;
        let resize_tx = rdp_session.resize_tx.clone();

        // Store resize_tx + websocket_port in the internal session for later use.
        {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(s) = sessions.get_mut(session_id) {
                s.resize_tx = Some(resize_tx);
                s.connected = true;
                s.websocket_port = websocket_port;
            }
        }

        // Spawn the actual connection task. `connect()` runs the full session
        // loop, so once it returns (cleanly or with an error) the session is no
        // longer live — flip `connected` to false so the UI/state don't report a
        // dead stream as active.
        let session_id_clone = session_id.to_string();
        let sessions_for_task = self.sessions.clone();
        tokio::spawn(async move {
            match rdp_session.connect().await {
                Ok(_) => info!("RDP session ended cleanly: {}", session_id_clone),
                Err(e) => error!("RDP session error: {}", e),
            }
            if let Some(s) = sessions_for_task.lock().unwrap().get_mut(&session_id_clone) {
                s.connected = false;
                s.resize_tx = None;
            }
        });

        let ws_url = format!("ws://127.0.0.1:{}/rdp/{}", websocket_port, session_id);

        info!("RDP session started: {}", session_id);
        Ok(ws_url)
    }

    /// Start the RDP session (legacy sync version, test-only).
    ///
    /// This does NOT establish an RDP connection or start a frame stream — it
    /// only flips local session state and returns a placeholder URL. It is gated
    /// behind `#[cfg(test)]` so production code cannot create a "ghost" session
    /// that reports as connected without a live stream; the real path is
    /// [`start_session_async`].
    #[cfg(test)]
    pub fn start_session(&self, session_id: &str) -> Result<String> {
        let websocket_port = self.get_free_port()?;

        // Mark as connected
        {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(session) = sessions.get_mut(session_id) {
                session.connected = true;
                session.websocket_port = websocket_port;
            }
        }

        // Generate WebSocket URL
        let ws_url = format!("ws://127.0.0.1:{}/rdp/{}", websocket_port, session_id);

        info!("RDP session started (sync): {}", session_id);
        Ok(ws_url)
    }

    /// Stop the RDP session
    pub fn stop_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();

        if let Some(session) = sessions.get_mut(session_id) {
            session.connected = false;
            session.resize_tx = None;
            info!("RDP session stopped: {}", session_id);
        }

        Ok(())
    }

    /// Send a dynamic resize request to a running session.
    pub async fn resize_session(&self, session_id: &str, width: u32, height: u32) -> Result<()> {
        let resize_tx = {
            let sessions = self.sessions.lock().unwrap();
            sessions
                .get(session_id)
                .and_then(|s| s.resize_tx.clone())
                .ok_or_else(|| {
                    anyhow::anyhow!("Session not found or not running: {}", session_id)
                })?
        };
        resize_tx
            .send(ResizeRequest { width, height })
            .await
            .context("send resize request")
    }

    /// Get session by ID and convert to public struct
    pub fn get_session(&self, session_id: &str) -> Option<RdpSession> {
        let sessions = self.sessions.lock().unwrap();

        sessions.get(session_id).map(|s| {
            let ws_url = format!("ws://127.0.0.1:{}/rdp/{}", s.websocket_port, session_id);
            RdpSession {
                id: s.id.clone(),
                connection_id: s.connection_id.clone(),
                hostname: s.hostname.clone(),
                port: s.port,
                username: s.username.clone(),
                resolution: s.resolution.clone(),
                color_depth: s.color_depth,
                websocket_port: s.websocket_port,
                websocket_url: ws_url,
                connected: s.connected,
                ssh_enabled: s.ssh_config.is_some(),
            }
        })
    }

    /// Delete session
    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        self.stop_session(session_id)?;

        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(session_id);

        info!("RDP session deleted: {}", session_id);
        Ok(())
    }

    /// Get a free port by binding to port 0.
    ///
    /// NOTE: There is an inherent TOCTOU window between dropping the listener
    /// and the caller binding to the returned port. Use `start_random_port`
    /// on `WebSocketServer` when possible to avoid this; this helper is
    /// retained for the sync `start_session` path which cannot await.
    fn get_free_port(&self) -> Result<u16> {
        use std::net::TcpListener;

        let listener =
            TcpListener::bind("127.0.0.1:0").context("Failed to bind to find free port")?;

        let port = listener.local_addr()?.port();
        drop(listener);

        Ok(port)
    }

    /// Parse resolution string into width and height
    fn parse_resolution(resolution: &str) -> (u32, u32) {
        let parts: Vec<&str> = resolution.split('x').collect();
        if parts.len() == 2 {
            (
                parts[0].parse().unwrap_or(1920),
                parts[1].parse().unwrap_or(1080),
            )
        } else {
            (1920, 1080) // Default resolution
        }
    }

    /// Send a test frame for simulation/testing purposes.
    ///
    /// Uses the raw `session_id` as the routing key to match
    /// [`WebSocketServer::register_session`] (which registers under the canonical
    /// id), so simulated frames reach the same client a real session would.
    pub fn send_test_frame(&self, session_id: &str, width: u32, height: u32) -> Result<()> {
        let ws_session_id = session_id.to_string();
        let frame = RdpFrame {
            width,
            height,
            data: vec![0u8; (width * height * 4) as usize], // RGBA frame
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            frame_number: 0,
        };

        let server = self.websocket_server.clone();
        tokio::spawn(async move {
            if let Err(e) = server.send_frame(&ws_session_id, frame).await {
                debug!("Failed to send test frame: {}", e);
            }
        });

        Ok(())
    }
}

impl Default for RdpManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RdpManager {
    /// Start the RDP session asynchronously (public wrapper)
    pub async fn start_session_with_password(
        &self,
        session_id: &str,
        password: &str,
    ) -> Result<String> {
        self.start_session_async(session_id, password).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::RemoteProtocol;

    #[test]
    fn test_rdp_manager_creation() {
        let manager = RdpManager::new();
        assert!(manager.sessions.lock().unwrap().is_empty());
    }

    #[test]
    fn test_create_and_get_session() {
        let manager = RdpManager::new();

        let connection = RemoteConnection {
            id: "test-conn-1".to_string(),
            name: "Test RDP".to_string(),
            protocol: RemoteProtocol::Rdp,
            hostname: "192.168.1.100".to_string(),
            port: 3389,
            username: Some("admin".to_string()),
            domain: None,
            ssh_enabled: false,
            ssh_hostname: None,
            ssh_port: None,
            ssh_username: None,
            resolution: "1920x1080".to_string(),
            color_depth: 32,
            clipboard_sync: true,
            drive_redirect: false,
            multi_monitor: false,
            compression: true,
            quality: 80,
            auto_resize: true,
            stretch_to_fill: false,
            created_at: "2024-01-01 00:00:00".to_string(),
            updated_at: "2024-01-01 00:00:00".to_string(),
            last_connected_at: None,
        };

        let session = manager
            .create_session(&connection, "password123", None, None, None)
            .unwrap();
        assert_eq!(session.hostname, "192.168.1.100");
        assert_eq!(session.port, 3389);
        assert!(!session.connected);

        // Get the session (public API)
        let retrieved = manager.get_session(&session.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, session.id);
    }

    #[test]
    fn test_start_and_stop_session() {
        let manager = RdpManager::new();

        let connection = RemoteConnection {
            id: "test-conn-2".to_string(),
            name: "Test RDP 2".to_string(),
            protocol: RemoteProtocol::Rdp,
            hostname: "192.168.1.101".to_string(),
            port: 3389,
            username: Some("user".to_string()),
            domain: None,
            ssh_enabled: false,
            ssh_hostname: None,
            ssh_port: None,
            ssh_username: None,
            resolution: "1280x720".to_string(),
            color_depth: 24,
            clipboard_sync: false,
            drive_redirect: false,
            multi_monitor: false,
            compression: false,
            quality: 70,
            auto_resize: false,
            stretch_to_fill: false,
            created_at: "2024-01-01 00:00:00".to_string(),
            updated_at: "2024-01-01 00:00:00".to_string(),
            last_connected_at: None,
        };

        let session = manager
            .create_session(&connection, "password123", None, None, None)
            .unwrap();

        // Start session
        let ws_url = manager.start_session(&session.id).unwrap();
        assert!(ws_url.contains("ws://127.0.0.1:"));
        assert!(ws_url.contains("/rdp/"));

        let retrieved = manager.get_session(&session.id).unwrap();
        assert!(retrieved.connected);

        // Stop session
        manager.stop_session(&session.id).unwrap();

        let final_session = manager.get_session(&session.id).unwrap();
        assert!(!final_session.connected);
    }
}
