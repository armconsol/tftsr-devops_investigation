//! WebSocket Server for RDP Frame Streaming
//!
//! Provides WebSocket server functionality for streaming RDP frames to clients.

use anyhow::{Context, Result};
use futures::SinkExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tracing::info;
use uuid::Uuid;

/// RDP Frame for WebSocket streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdpFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub frame_number: u64,
}

/// WebSocket session for a single client
#[derive(Debug, Clone)]
pub struct WebSocketSession {
    pub session_id: String,
    pub rdp_session_id: String,
    pub connected_at: u64,
}

/// WebSocket server for RDP frame streaming
pub struct WebSocketServer {
    sessions: Arc<Mutex<HashMap<String, WebSocketSession>>>,
    frame_broadcasters: Arc<Mutex<HashMap<String, mpsc::Sender<RdpFrame>>>>,
    /// Holds receivers for sessions that have been registered but whose
    /// WebSocket client has not yet connected.  `handle_client` takes
    /// ownership of the receiver and removes it from this map.
    pending_receivers: Arc<Mutex<HashMap<String, mpsc::Receiver<RdpFrame>>>>,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            frame_broadcasters: Arc::new(Mutex::new(HashMap::new())),
            pending_receivers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start the WebSocket server on a specific port
    pub async fn start(&self, port: u16) -> Result<()> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr)
            .await
            .context(format!("Failed to bind WebSocket server to {}", addr))?;

        info!("WebSocket server listening on {}", addr);

        let broadcast_map = self.frame_broadcasters.clone();
        let pending_receivers = self.pending_receivers.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let session_id = Uuid::now_v7().to_string();
                        let broadcast_clone = broadcast_map.clone();
                        let pending_clone = pending_receivers.clone();
                        let session_id_clone = session_id.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_client(
                                stream,
                                &session_id_clone,
                                broadcast_clone,
                                pending_clone,
                            )
                            .await
                            {
                                tracing::error!(
                                    "WebSocket error for session {}: {}",
                                    session_id_clone,
                                    e
                                );
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("WebSocket accept error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Start WebSocket server on a random available port
    pub async fn start_random_port(&self) -> Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        drop(listener);

        self.start(port).await?;
        Ok(port)
    }

    /// Register a session and return the broadcaster for sending frames
    pub async fn register_session(
        &self,
        session_id: &str,
        rdp_session_id: &str,
    ) -> mpsc::Sender<RdpFrame> {
        let (tx, rx) = mpsc::channel::<RdpFrame>(100);
        let tx_clone = tx.clone();

        let session = WebSocketSession {
            session_id: session_id.to_string(),
            rdp_session_id: rdp_session_id.to_string(),
            connected_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };

        // Store session metadata
        let mut sessions = self.sessions.lock().await;
        sessions.insert(session.session_id.clone(), session.clone());
        drop(sessions);

        // Park receiver until a WebSocket client connects and takes it
        let mut pending = self.pending_receivers.lock().await;
        pending.insert(session_id.to_string(), rx);
        drop(pending);

        // Pre-register broadcaster; handle_client replaces it with a live channel
        // when a WebSocket client connects.
        let mut broadcast_map = self.frame_broadcasters.lock().await;
        broadcast_map.insert(session_id.to_string(), tx_clone);
        drop(broadcast_map);

        tx
    }

    /// Send a frame to a specific session
    pub async fn send_frame(&self, session_id: &str, frame: RdpFrame) -> Result<()> {
        let broadcast_map = self.frame_broadcasters.lock().await;

        if let Some(tx) = broadcast_map.get(session_id) {
            tx.send(frame)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to send frame: {}", e))?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }

    /// Unregister a session
    pub async fn unregister_session(&self, session_id: &str) {
        let mut broadcast_map = self.frame_broadcasters.lock().await;
        broadcast_map.remove(session_id);
        drop(broadcast_map);

        let mut pending = self.pending_receivers.lock().await;
        pending.remove(session_id);
        drop(pending);

        let mut sessions = self.sessions.lock().await;
        sessions.remove(session_id);

        info!("WebSocket session unregistered: {}", session_id);
    }

    /// Get session info
    pub async fn get_session(&self, session_id: &str) -> Option<WebSocketSession> {
        let sessions = self.sessions.lock().await;
        sessions.get(session_id).cloned()
    }

    /// List all active sessions
    pub async fn list_sessions(&self) -> Vec<WebSocketSession> {
        let sessions = self.sessions.lock().await;
        sessions.values().cloned().collect()
    }

    async fn handle_client(
        stream: tokio::net::TcpStream,
        session_id: &str,
        broadcast_map: Arc<Mutex<HashMap<String, mpsc::Sender<RdpFrame>>>>,
        pending_receivers: Arc<Mutex<HashMap<String, mpsc::Receiver<RdpFrame>>>>,
    ) -> Result<()> {
        use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
        use tokio_tungstenite::tungstenite::http::header;

        #[allow(clippy::result_large_err)]
        let callback = |_req: &Request, mut response: Response| {
            let headers = response.headers_mut();
            headers.insert(header::SEC_WEBSOCKET_PROTOCOL, "binary".parse().unwrap());
            Ok(response)
        };

        let ws_stream = tokio_tungstenite::accept_hdr_async(stream, callback).await?;
        let (mut ws_sink, mut ws_source) = futures::StreamExt::split(ws_stream);

        // Take the pre-registered receiver if available; otherwise create a fresh channel.
        let mut frame_rx = {
            let mut pending = pending_receivers.lock().await;
            if let Some(rx) = pending.remove(session_id) {
                rx
            } else {
                let (frame_tx, rx) = mpsc::channel::<RdpFrame>(100);
                let mut map = broadcast_map.lock().await;
                map.insert(session_id.to_string(), frame_tx);
                rx
            }
        };

        // Forward frames from RDP to WebSocket client.
        let send_task = tokio::spawn(async move {
            while let Some(frame) = frame_rx.recv().await {
                let payload = match serde_json::to_vec(&frame) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!("Frame serialization failed: {}", e);
                        continue;
                    }
                };
                if let Err(e) = ws_sink
                    .send(tokio_tungstenite::tungstenite::Message::Binary(payload))
                    .await
                {
                    tracing::error!("WebSocket send failed: {}", e);
                    break;
                }
            }
        });

        // Drain inbound messages (input events) until the client disconnects.
        loop {
            match ws_source.next().await {
                Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => {
                    tracing::debug!("Client sent close message");
                    break;
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(data))) => {
                    tracing::debug!("Received input from client: {} bytes", data.len());
                }
                Some(Ok(_)) => {}
                Some(Err(e)) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
                None => break,
            }
        }

        send_task.abort();

        // Remove session from broadcast map on disconnect.
        let mut map = broadcast_map.lock().await;
        map.remove(session_id);

        Ok(())
    }
}

impl Default for WebSocketServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_server_creation() {
        let server = WebSocketServer::new();
        assert!(server.sessions.lock().await.is_empty());
    }

    #[tokio::test]
    async fn test_register_session() {
        let server = WebSocketServer::new();
        let rdp_session_id = "rdp-session-123";

        let tx = server
            .register_session("ws-session-123", rdp_session_id)
            .await;
        assert!(tx
            .send(RdpFrame {
                width: 1920,
                height: 1080,
                data: vec![0u8; 100],
                timestamp: 0,
                frame_number: 0,
            })
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_send_frame() {
        let server = WebSocketServer::new();
        let session_id = "test-session";

        server.register_session(session_id, "rdp-session").await;

        let frame = RdpFrame {
            width: 1920,
            height: 1080,
            data: vec![0u8; 100],
            timestamp: 0,
            frame_number: 0,
        };

        let result = server.send_frame(session_id, frame).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unregister_session() {
        let server = WebSocketServer::new();
        let session_id = "test-session";

        server.register_session(session_id, "rdp-session").await;
        assert!(server.get_session(session_id).await.is_some());

        server.unregister_session(session_id).await;
        assert!(server.get_session(session_id).await.is_none());
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let server = WebSocketServer::new();

        server.register_session("session-1", "rdp-1").await;
        server.register_session("session-2", "rdp-2").await;

        let sessions = server.list_sessions().await;
        assert_eq!(sessions.len(), 2);
    }
}
