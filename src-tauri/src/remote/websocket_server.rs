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
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio_stream::StreamExt;
use tracing::{debug, info, warn};

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
    pub auth_token: String,
    pub connected_at: u64,
}

/// WebSocket server for RDP frame streaming
pub struct WebSocketServer {
    sessions: Arc<Mutex<HashMap<String, WebSocketSession>>>,
    frame_broadcasters: Arc<Mutex<HashMap<String, broadcast::Sender<RdpFrame>>>>,
    active_connections: Arc<Mutex<HashMap<String, usize>>>,
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
            active_connections: Arc::new(Mutex::new(HashMap::new())),
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
        let sessions = self.sessions.clone();
        let active_connections = self.active_connections.clone();
        let input_senders = self.input_senders.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let broadcast_clone = broadcast_map.clone();
                        let sessions_clone = sessions.clone();
                        let active_connections_clone = active_connections.clone();
                        let input_clone = input_senders.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_client(
                                stream,
                                sessions_clone,
                                broadcast_clone,
                                active_connections_clone,
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
    ) -> (broadcast::Sender<RdpFrame>, String) {
        let (tx, _rx) = broadcast::channel::<RdpFrame>(4);
        let tx_clone = tx.clone();
        let auth_token = uuid::Uuid::now_v7().to_string();

        let session = WebSocketSession {
            session_id: session_id.to_string(),
            rdp_session_id: rdp_session_id.to_string(),
            auth_token: auth_token.clone(),
            connected_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };

        // Store session metadata
        let mut sessions = self.sessions.lock().await;
        sessions.insert(session.session_id.clone(), session.clone());
        drop(sessions);

        // Register a persistent per-session broadcaster.
        let mut broadcast_map = self.frame_broadcasters.lock().await;
        broadcast_map.insert(session_id.to_string(), tx_clone);
        drop(broadcast_map);

        (tx, auth_token)
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
            // If there are temporarily no WebSocket subscribers, drop the frame but
            // keep session routing intact so reconnect can resume streaming.
            if tx.send(frame).is_err() {
                debug!(
                    "Dropping frame for session {}: no active websocket clients",
                    session_id
                );
            }
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

        let mut input_senders = self.input_senders.lock().await;
        input_senders.remove(session_id);
        drop(input_senders);

        let mut active_connections = self.active_connections.lock().await;
        active_connections.remove(session_id);
        drop(active_connections);

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
        sessions: Arc<Mutex<HashMap<String, WebSocketSession>>>,
        broadcast_map: Arc<Mutex<HashMap<String, broadcast::Sender<RdpFrame>>>>,
        active_connections: Arc<Mutex<HashMap<String, usize>>>,
        input_senders: Arc<Mutex<HashMap<String, mpsc::Sender<RawInputEvent>>>>,
    ) -> Result<()> {
        use std::sync::Mutex as StdMutex;
        use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
        use tokio_tungstenite::tungstenite::http::{header, StatusCode};

        // Capture full URI (`/rdp/{session_id}?token=...`) during handshake.
        let captured_uri = Arc::new(StdMutex::new(String::new()));
        let uri_for_cb = captured_uri.clone();

        #[allow(clippy::result_large_err)]
        let callback = move |req: &Request, mut response: Response| {
            if let Ok(mut uri) = uri_for_cb.lock() {
                *uri = req.uri().to_string();
            }
            if !select_binary_subprotocol(
                req.headers()
                    .get(header::SEC_WEBSOCKET_PROTOCOL)
                    .and_then(|h| h.to_str().ok()),
            ) {
                let mut reject: ErrorResponse =
                    ErrorResponse::new(Some("Missing required websocket subprotocol".into()));
                *reject.status_mut() = StatusCode::BAD_REQUEST;
                return Err(reject);
            }
            response
                .headers_mut()
                .insert(header::SEC_WEBSOCKET_PROTOCOL, "binary".parse().unwrap());
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

        let (session_id, provided_token) = {
            let uri = captured_uri.lock().unwrap();
            let path = uri.split('?').next().unwrap_or("");
            let token = extract_query_token(&uri);
            (session_id_from_path(path).to_string(), token)
        };

        if session_id.is_empty() {
            return Err(anyhow::anyhow!(
                "WebSocket request missing session id in path"
            ));
        }

        // Require a per-session auth token to prevent cross-session hijacking.
        let expected_token = {
            let sessions = sessions.lock().await;
            sessions
                .get(&session_id)
                .map(|s| s.auth_token.clone())
                .ok_or_else(|| anyhow::anyhow!("Unknown session id: {}", session_id))?
        };
        if provided_token.as_deref() != Some(expected_token.as_str()) {
            return Err(anyhow::anyhow!(
                "WebSocket auth token mismatch for session {}",
                session_id
            ));
        }

        // Only accept connections for a pre-registered session. The id is
        // client-controlled (URL path), so refuse unknown ids.
        let mut frame_rx = {
            let broadcasters = broadcast_map.lock().await;
            match broadcasters.get(&session_id) {
                Some(tx) => tx.subscribe(),
                None => {
                    return Err(anyhow::anyhow!(
                        "WebSocket connection for unregistered session: {}",
                        session_id
                    ));
                }
            }
        };

        // Allow exactly one active controlling WebSocket client at a time.
        {
            let mut active = active_connections.lock().await;
            let count = active.entry(session_id.clone()).or_insert(0);
            if *count > 0 {
                return Err(anyhow::anyhow!(
                    "WebSocket session already has an active client: {}",
                    session_id
                ));
            }
            *count = 1;
        }

        info!("WebSocket client connected for session: {}", session_id);

        let (mut ws_sink, mut ws_source) = futures::StreamExt::split(ws_stream);

        // Forward frames from RDP to WebSocket client.
        let session_id_for_logging = session_id.clone();
        let send_task = tokio::spawn(async move {
            let mut frame_count = 0u64;
            loop {
                let frame = match frame_rx.recv().await {
                    Ok(frame) => frame,
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!(
                            "WebSocket frame consumer lagged for session {} (skipped {})",
                            session_id_for_logging, skipped
                        );
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                };
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

        let mut active = active_connections.lock().await;
        active.remove(&session_id);

        Ok(())
    }
}

fn select_binary_subprotocol(requested: Option<&str>) -> bool {
    requested
        .map(|list| {
            list.split(',')
                .any(|v| v.trim().eq_ignore_ascii_case("binary"))
        })
        .unwrap_or(false)
}

fn extract_query_token(uri: &str) -> Option<String> {
    let query = uri.split('?').nth(1)?;
    query
        .split('&')
        .find_map(|part| part.strip_prefix("token=").map(|v| v.to_string()))
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

        let (tx, token) = server
            .register_session("ws-session-123", rdp_session_id)
            .await;
        assert!(!token.is_empty());
        assert!(tx
            .send(RdpFrame {
                width: 1920,
                height: 1080,
                data: vec![0u8; 100],
                timestamp: 0,
                frame_number: 0,
            })
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

    #[test]
    fn test_select_binary_subprotocol() {
        assert!(select_binary_subprotocol(Some("binary")));
        assert!(select_binary_subprotocol(Some("json, binary")));
        assert!(select_binary_subprotocol(Some("Binary")));
        assert!(!select_binary_subprotocol(Some("json")));
        assert!(!select_binary_subprotocol(None));
    }

    #[test]
    fn test_extract_query_token() {
        assert_eq!(
            extract_query_token("/rdp/s1?token=abc123"),
            Some("abc123".to_string())
        );
        assert_eq!(
            extract_query_token("/rdp/s1?foo=1&token=xyz&bar=2"),
            Some("xyz".to_string())
        );
        assert_eq!(extract_query_token("/rdp/s1"), None);
    }

    #[tokio::test]
    async fn test_disconnect_does_not_drop_session_routing() {
        use tokio_tungstenite::connect_async;
        use tokio_tungstenite::tungstenite::client::IntoClientRequest;

        let server = WebSocketServer::new();
        let port = server
            .start_random_port()
            .await
            .expect("Should start server");

        let session_id = "reconnect-session";
        let (_, token) = server.register_session(session_id, "rdp-session").await;

        let mut request = format!("ws://127.0.0.1:{}/rdp/{}?token={}", port, session_id, token)
            .into_client_request()
            .expect("valid websocket request");
        request
            .headers_mut()
            .insert("Sec-WebSocket-Protocol", "binary".parse().unwrap());

        let (mut ws, _) = connect_async(request).await.expect("connect websocket");
        ws.close(None).await.expect("close websocket");
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let result = server
            .send_frame(
                session_id,
                RdpFrame {
                    width: 64,
                    height: 64,
                    data: vec![0u8; 64 * 64 * 4],
                    timestamp: 0,
                    frame_number: 1,
                },
            )
            .await;
        assert!(
            result.is_ok(),
            "routing should remain registered across websocket reconnect windows"
        );
    }

    #[test]
    fn test_encode_frame() {
        let frame = RdpFrame {
            width: 1920,
            height: 1080,
            data: vec![255u8; 1920 * 1080 * 4], // White RGBA frame
            timestamp: 1234567890,
            frame_number: 1,
        };

        let encoded = encode_frame(&frame);

        // Check header: first 4 bytes = width (LE), next 4 bytes = height (LE)
        assert_eq!(encoded.len(), 8 + (1920 * 1080 * 4));
        assert_eq!(
            u32::from_le_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]),
            1920
        );
        assert_eq!(
            u32::from_le_bytes([encoded[4], encoded[5], encoded[6], encoded[7]]),
            1080
        );

        // Check pixel data starts at offset 8
        assert_eq!(encoded[8], 255);
    }

    #[tokio::test]
    async fn test_websocket_server_lifecycle() {
        let server = WebSocketServer::new();

        // Start server on random port (returns only port, no receiver)
        let port = server
            .start_random_port()
            .await
            .expect("Should start server");
        assert!(port > 0, "Port should be assigned");

        // Register a session (returns frame sender)
        let session_id = "test-session-123";
        let rdp_session_id = "rdp-session-123";

        let _frame_tx = server.register_session(session_id, rdp_session_id).await;

        // Verify session is tracked (returns WebSocketSession structs)
        let sessions = server.list_sessions().await;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].rdp_session_id, rdp_session_id);

        // Unregister session (returns void)
        server.unregister_session(session_id).await;

        // Verify session is removed
        let sessions_after = server.list_sessions().await;
        assert_eq!(sessions_after.len(), 0);
    }
}
