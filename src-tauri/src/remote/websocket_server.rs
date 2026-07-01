//! WebSocket Server for RDP Frame Streaming
//!
//! Provides WebSocket server functionality for streaming RDP frames to clients.

use crate::remote::input::RawInputEvent;
use anyhow::{Context, Result};
use futures::SinkExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tracing::info;

/// RDP Frame for WebSocket streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdpFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub frame_number: u64,
}

/// Encode a frame into the binary wire format the browser client expects:
/// `[u32 LE width][u32 LE height][RGBA pixel bytes...]`.
///
/// The frontend decodes this layout directly (width at byte 0, height at byte 4,
/// pixels from byte 8). Sending any other representation (e.g. JSON) results in a
/// blank/black canvas because `putImageData` receives mismatched dimensions.
pub fn encode_frame(frame: &RdpFrame) -> Vec<u8> {
    let mut payload = Vec::with_capacity(8 + frame.data.len());
    payload.extend_from_slice(&frame.width.to_le_bytes());
    payload.extend_from_slice(&frame.height.to_le_bytes());
    payload.extend_from_slice(&frame.data);
    payload
}

/// Extract the RDP session id from a WebSocket request path of the form
/// `/rdp/{session_id}`. Returns an empty string when the path has no segment.
///
/// The session id must match the id used by `register_session` (and embedded in
/// the `ws://.../rdp/{id}` URL handed to the browser) so the parked frame
/// receiver can be routed to the connecting client. Using a freshly generated
/// id here instead silently breaks frame delivery, producing a black canvas.
pub fn session_id_from_path(path: &str) -> &str {
    path.trim_end_matches('/').rsplit('/').next().unwrap_or("")
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
    /// Per-session sink for decoded inbound input events. `handle_client`
    /// forwards each parsed `RawInputEvent` here; the RDP manager drains it and
    /// drives the session's input methods.
    input_senders: Arc<Mutex<HashMap<String, mpsc::Sender<RawInputEvent>>>>,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            frame_broadcasters: Arc::new(Mutex::new(HashMap::new())),
            pending_receivers: Arc::new(Mutex::new(HashMap::new())),
            input_senders: Arc::new(Mutex::new(HashMap::new())),
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
        let input_senders = self.input_senders.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let broadcast_clone = broadcast_map.clone();
                        let pending_clone = pending_receivers.clone();
                        let input_clone = input_senders.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_client(
                                stream,
                                broadcast_clone,
                                pending_clone,
                                input_clone,
                            )
                            .await
                            {
                                tracing::error!("WebSocket client error: {}", e);
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
        info!(
            "Registered session in frame_broadcasters: '{}', total sessions: {}",
            session_id,
            broadcast_map.len()
        );
        drop(broadcast_map);

        tx
    }

    /// Register the sink that decoded inbound input events for `session_id`
    /// should be forwarded to. Must be called before the browser connects so
    /// `handle_client` can route keystrokes and pointer events to the session.
    pub async fn register_input_sender(
        &self,
        session_id: &str,
        sender: mpsc::Sender<RawInputEvent>,
    ) {
        let mut senders = self.input_senders.lock().await;
        senders.insert(session_id.to_string(), sender);
    }

    /// Send a frame to a specific session
    pub async fn send_frame(&self, session_id: &str, frame: RdpFrame) -> Result<()> {
        let broadcast_map = self.frame_broadcasters.lock().await;

        if let Some(tx) = broadcast_map.get(session_id) {
            tx.send(frame)
                .await
                .map_err(|e| antml::<parameter name="anyhow!("Failed to send frame: {}", e))?;
            Ok(())
        } else {
            // Debug: log what sessions are actually registered
            let registered_sessions: Vec<String> = broadcast_map.keys().cloned().collect();
            warn!(
                "Session not found in frame_broadcasters: '{}'. Registered sessions: {:?}",
                session_id, registered_sessions
            );
            Err(antml::anyhow!("Session not found: {}", session_id))
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

        let mut input_senders = self.input_senders.lock().await;
        input_senders.remove(session_id);
        drop(input_senders);

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
        broadcast_map: Arc<Mutex<HashMap<String, mpsc::Sender<RdpFrame>>>>,
        pending_receivers: Arc<Mutex<HashMap<String, mpsc::Receiver<RdpFrame>>>>,
        input_senders: Arc<Mutex<HashMap<String, mpsc::Sender<RawInputEvent>>>>,
    ) -> Result<()> {
        use std::sync::Mutex as StdMutex;
        use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
        use tokio_tungstenite::tungstenite::http::header;

        // Capture the request path during the handshake so we can route frames
        // for the session id embedded in `/rdp/{session_id}`.
        let captured_path = Arc::new(StdMutex::new(String::new()));
        let path_for_cb = captured_path.clone();

        #[allow(clippy::result_large_err)]
        let callback = move |req: &Request, mut response: Response| {
            if let Ok(mut p) = path_for_cb.lock() {
                *p = req.uri().path().to_string();
            }
            let headers = response.headers_mut();
            headers.insert(header::SEC_WEBSOCKET_PROTOCOL, "binary".parse().unwrap());
            Ok(response)
        };

        // Cap message/frame sizes. Input events are tiny JSON; a 4 KiB ceiling
        // prevents a malicious local client from forcing large heap allocations
        // (the tungstenite default allows 64 MiB messages).
        let ws_config = tokio_tungstenite::tungstenite::protocol::WebSocketConfig {
            max_message_size: Some(4096),
            max_frame_size: Some(4096),
            ..Default::default()
        };
        let ws_stream =
            tokio_tungstenite::accept_hdr_async_with_config(stream, callback, Some(ws_config))
                .await?;

        let session_id = {
            let path = captured_path.lock().unwrap();
            session_id_from_path(&path).to_string()
        };

        if session_id.is_empty() {
            return Err(anyhow::anyhow!(
                "WebSocket request missing session id in path"
            ));
        }

        info!("WebSocket client connected for session: {}", session_id);

        let (mut ws_sink, mut ws_source) = futures::StreamExt::split(ws_stream);

        // Only accept connections for a pre-registered session. The id is
        // client-controlled (URL path), so refuse unknown ids rather than
        // creating a dangling broadcaster entry for an arbitrary session.
        let mut frame_rx = {
            let mut pending = pending_receivers.lock().await;
            match pending.remove(&session_id) {
                Some(rx) => rx,
                None => {
                    return Err(anyhow::anyhow!(
                        "WebSocket connection for unregistered session: {}",
                        session_id
                    ));
                }
            }
        };

        // Forward frames from RDP to WebSocket client.
        let session_id_for_logging = session_id.clone();
        let send_task = tokio::spawn(async move {
            let mut frame_count = 0u64;
            while let Some(frame) = frame_rx.recv().await {
                frame_count += 1;
                let payload = encode_frame(&frame);

                // Log first frame and every 100th frame
                if frame_count == 1 || frame_count.is_multiple_of(100) {
                    info!(
                        "WebSocket sending frame #{} for session {}: {}x{}, {} bytes",
                        frame_count,
                        session_id_for_logging,
                        frame.width,
                        frame.height,
                        payload.len()
                    );
                }

                if let Err(e) = ws_sink
                    .send(tokio_tungstenite::tungstenite::Message::Binary(payload))
                    .await
                {
                    tracing::error!("WebSocket send failed: {}", e);
                    break;
                }
            }
        });

        // Look up the input sink for this session (registered by the RDP
        // manager). Absent for view-only sessions, in which case inbound input
        // is simply ignored.
        let input_sender = {
            let senders = input_senders.lock().await;
            senders.get(&session_id).cloned()
        };

        // Drain inbound messages (input events) until the client disconnects.
        // The browser sends input as JSON text frames (see RemoteDesktopPage).
        loop {
            match ws_source.next().await {
                Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => {
                    tracing::debug!("Client sent close message");
                    break;
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                    if let (Some(sender), Some(event)) =
                        (input_sender.as_ref(), RawInputEvent::from_json(&text))
                    {
                        // Drop the event rather than block the read loop if the
                        // input channel is saturated.
                        let _ = sender.try_send(event);
                    } else {
                        tracing::trace!("Ignoring unparseable input frame");
                    }
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(data))) => {
                    tracing::debug!("Ignoring {} bytes of binary input", data.len());
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
        map.remove(&session_id);

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

    #[test]
    fn test_encode_frame_binary_layout() {
        // 2x1 image: two RGBA pixels.
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let frame = RdpFrame {
            width: 2,
            height: 1,
            data: data.clone(),
            timestamp: 123,
            frame_number: 7,
        };

        let payload = encode_frame(&frame);

        // Layout: [u32 LE width][u32 LE height][RGBA...]
        assert_eq!(payload.len(), 8 + data.len());
        assert_eq!(u32::from_le_bytes(payload[0..4].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(payload[4..8].try_into().unwrap()), 1);
        assert_eq!(&payload[8..], &data[..]);
    }

    #[test]
    fn test_session_id_from_path() {
        assert_eq!(
            session_id_from_path("/rdp/abc-123"),
            "abc-123",
            "extracts id from /rdp/{{id}}"
        );
        assert_eq!(
            session_id_from_path("/rdp/abc-123/"),
            "abc-123",
            "tolerates trailing slash"
        );
        assert_eq!(session_id_from_path("abc-123"), "abc-123");
        assert_eq!(session_id_from_path("/"), "");
        assert_eq!(session_id_from_path(""), "");
    }
}
