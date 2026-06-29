//! RDP Client using IronRDP
//!
//! Full RDP protocol connection with TLS, SSH tunnel support, frame capture,
//! input handling, and dynamic resolution via Display Control Virtual Channel.

use super::ssh_tunnel::{SshTunnel, SshTunnelConfig};
use super::websocket_server::{RdpFrame, WebSocketServer};
use anyhow::{Context, Result};
use ironrdp::connector::{self, ConnectionResult, Credentials};
use ironrdp::pdu::gcc::KeyboardType;
use ironrdp::pdu::rdp::capability_sets::MajorPlatformType;
use ironrdp::pdu::rdp::client_info::PerformanceFlags;
use ironrdp::session::image::DecodedImage;
use ironrdp::session::{ActiveStage, ActiveStageOutput};
use ironrdp_input::{MouseButton, MousePosition, Scancode};
use ironrdp_pdu::input::fast_path::FastPathInputEvent;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// ─── Stream trait ────────────────────────────────────────────────────────────

trait RdpStream: Read + Write + Sync + Send {}
impl<T: Read + Write + Sync + Send> RdpStream for T {}

// ─── CredSSP network stub ─────────────────────────────────────────────────────

struct SimpleNetworkClient;

impl ironrdp::connector::sspi::network_client::NetworkClient for SimpleNetworkClient {
    fn send(
        &self,
        _request: &ironrdp::connector::sspi::NetworkRequest,
    ) -> Result<Vec<u8>, ironrdp::connector::sspi::Error> {
        Err(ironrdp::connector::sspi::Error::new(
            ironrdp::connector::sspi::ErrorKind::OperationNotSupported,
            "Network requests not implemented",
        ))
    }
}

// ─── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub width: u32,
    pub height: u32,
    pub bit_depth: u32,
    pub domain: Option<String>,
    pub ssh_config: Option<SshTunnelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdpSessionState {
    pub connected: bool,
    pub width: u32,
    pub height: u32,
    pub bit_depth: u32,
    pub frames_sent: u64,
    pub last_frame_time: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct ResizeRequest {
    pub width: u32,
    pub height: u32,
}

type FrameBroadcaster = mpsc::Sender<RdpFrame>;

// ─── RdpClientSession ─────────────────────────────────────────────────────────

pub struct RdpClientSession {
    pub session_id: String,
    pub config: RdpConfig,
    pub state: Arc<Mutex<RdpSessionState>>,
    frame_sender: Option<FrameBroadcaster>,
    input_state: Arc<Mutex<ironrdp_input::Database>>,
    pub resize_tx: mpsc::Sender<ResizeRequest>,
    resize_rx: Arc<Mutex<mpsc::Receiver<ResizeRequest>>>,
    input_tx: mpsc::Sender<SmallVec<[FastPathInputEvent; 2]>>,
    input_rx: Arc<Mutex<mpsc::Receiver<SmallVec<[FastPathInputEvent; 2]>>>>,
}

impl RdpClientSession {
    pub fn new(config: RdpConfig, websocket_server: Arc<WebSocketServer>) -> Result<Self> {
        let session_id = Uuid::now_v7().to_string();
        Self::new_with_id(session_id, config, websocket_server)
    }

    /// Create a session bound to a caller-provided `session_id`.
    ///
    /// The id is used as the key the frame-forwarding task sends under
    /// (`WebSocketServer::send_frame`). It MUST match the id registered via
    /// `WebSocketServer::register_session` and embedded in the `ws://.../rdp/{id}`
    /// URL handed to the browser, otherwise frames are dropped and the canvas
    /// stays black.
    pub fn new_with_id(
        session_id: String,
        config: RdpConfig,
        websocket_server: Arc<WebSocketServer>,
    ) -> Result<Self> {
        let (frame_tx, mut frame_rx) = mpsc::channel::<RdpFrame>(100);
        let (resize_tx, resize_rx) = mpsc::channel::<ResizeRequest>(8);
        let (input_tx, input_rx) = mpsc::channel::<SmallVec<[FastPathInputEvent; 2]>>(64);

        let ws = websocket_server.clone();
        let sid = session_id.clone();
        tokio::spawn(async move {
            while let Some(frame) = frame_rx.recv().await {
                if let Err(e) = ws.send_frame(&sid, frame).await {
                    warn!("Failed to send frame: {}", e);
                }
            }
        });

        Ok(Self {
            session_id,
            state: Arc::new(Mutex::new(RdpSessionState {
                connected: false,
                width: config.width,
                height: config.height,
                bit_depth: config.bit_depth,
                frames_sent: 0,
                last_frame_time: 0,
            })),
            config: config.clone(),
            frame_sender: Some(frame_tx),
            input_state: Arc::new(Mutex::new(ironrdp_input::Database::new())),
            resize_tx,
            resize_rx: Arc::new(Mutex::new(resize_rx)),
            input_tx,
            input_rx: Arc::new(Mutex::new(input_rx)),
        })
    }

    // ── Entry point ───────────────────────────────────────────────────────────

    pub async fn connect(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        info!("Connecting to RDP server at {}", addr);

        let broadcaster = self
            .frame_sender
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Frame sender not initialized"))?;

        let mut image = DecodedImage::new(
            ironrdp_graphics::image_processing::PixelFormat::RgbA32,
            self.config.width as u16,
            self.config.height as u16,
        );

        let (connection_result, mut framed) = if let Some(ssh_config) = &self.config.ssh_config {
            self.connect_via_ssh_tunnel(ssh_config, &addr).await?
        } else {
            self.connect_direct(&addr).await?
        };

        info!("RDP connection established, beginning active stage");

        {
            let mut st = self.state.lock().unwrap();
            st.connected = true;
        }

        let mut active_stage = ActiveStage::new(connection_result);

        'outer: loop {
            // Non-blocking check for resize requests
            if let Ok(req) = self.resize_rx.lock().unwrap().try_recv() {
                match active_stage.encode_resize(req.width, req.height, Some(100), None) {
                    Some(Ok(bytes)) => {
                        if let Err(e) = framed.write_all(&bytes) {
                            warn!("Failed to write resize PDU: {}", e);
                        } else {
                            image = DecodedImage::new(
                                ironrdp_graphics::image_processing::PixelFormat::RgbA32,
                                req.width as u16,
                                req.height as u16,
                            );
                            let mut st = self.state.lock().unwrap();
                            st.width = req.width;
                            st.height = req.height;
                            info!("Display resized to {}x{}", req.width, req.height);
                        }
                    }
                    Some(Err(e)) => warn!("encode_resize error: {:?}", e),
                    None => debug!("Display Control channel not ready yet"),
                }
            }

            // Non-blocking drain of pending input events
            while let Ok(events) = self.input_rx.lock().unwrap().try_recv() {
                match active_stage.process_fastpath_input(&mut image, &events) {
                    Ok(outputs) => {
                        for out in outputs {
                            if let ActiveStageOutput::ResponseFrame(data) = out {
                                if let Err(e) = framed.write_all(&data) {
                                    warn!("Failed to write input PDU: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => warn!("process_fastpath_input error: {:?}", e),
                }
            }

            match framed.read_pdu() {
                Ok((action, payload)) => {
                    let outputs = active_stage.process(&mut image, action, &payload)?;
                    for out in outputs {
                        match out {
                            ActiveStageOutput::ResponseFrame(data) => {
                                framed.write_all(&data).context("write response")?;
                            }
                            ActiveStageOutput::GraphicsUpdate(_) => {
                                self.capture_and_send_frame(&image, &broadcaster)?;
                            }
                            ActiveStageOutput::PointerBitmap(_)
                            | ActiveStageOutput::PointerDefault
                            | ActiveStageOutput::PointerHidden
                            | ActiveStageOutput::PointerPosition { .. } => {}
                            ActiveStageOutput::Terminate(_) => {
                                info!("RDP session terminated by server");
                                break 'outer;
                            }
                            _ => {}
                        }
                    }
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(e) => {
                    error!("Error reading PDU: {}", e);
                    break;
                }
            }
        }

        {
            let mut st = self.state.lock().unwrap();
            st.connected = false;
        }
        Ok(())
    }

    // ── Direct TCP + TLS ──────────────────────────────────────────────────────

    async fn connect_direct(
        &self,
        addr: &str,
    ) -> Result<(
        ConnectionResult,
        ironrdp_blocking::Framed<Box<dyn RdpStream>>,
    )> {
        // Phase 1 — open raw TCP and run connect_begin (pre-upgrade steps)
        let std_tcp = TcpStream::connect(addr).context(format!("TCP connect to {}", addr))?;
        std_tcp
            .set_read_timeout(Some(Duration::from_secs(30)))
            .context("set_read_timeout")?;

        let client_addr = std_tcp.local_addr().context("get local addr")?;
        let mut connector =
            connector::ClientConnector::new(self.build_connector_config(), client_addr);

        let tcp_clone = std_tcp.try_clone().context("clone TcpStream")?;
        let boxed_pre: Box<dyn RdpStream> = Box::new(tcp_clone);
        let mut pre_framed = ironrdp_blocking::Framed::new(boxed_pre);

        ironrdp_blocking::connect_begin(&mut pre_framed, &mut connector)
            .context("connect_begin")?;

        // Phase 2 — TLS upgrade if the server asked for it
        if connector.should_perform_security_upgrade() {
            info!("Upgrading RDP transport to TLS for {}", self.config.host);

            // Convert to tokio stream for the async TLS handshake
            let tokio_tcp =
                tokio::net::TcpStream::from_std(std_tcp).context("std → tokio TcpStream")?;

            match ironrdp_tls::upgrade(tokio_tcp, &self.config.host).await {
                Ok((tls_stream, cert)) => {
                    let server_pub_key = ironrdp_tls::extract_tls_server_public_key(&cert)
                        .map(|k| k.to_vec())
                        .unwrap_or_default();
                    info!("TLS handshake OK, public key {} B", server_pub_key.len());

                    connector.mark_security_upgrade_as_done();
                    let upgraded = ironrdp_blocking::mark_as_upgraded(
                        ironrdp_blocking::skip_connect_begin(&mut connector),
                        &mut connector,
                    );

                    let sync_tls = SyncTlsStream::new(tls_stream);
                    let boxed: Box<dyn RdpStream> = Box::new(sync_tls);
                    let mut framed = ironrdp_blocking::Framed::new(boxed);

                    let conn = {
                        let mut nc = SimpleNetworkClient;
                        ironrdp_blocking::connect_finalize(
                            upgraded,
                            connector,
                            &mut framed,
                            &mut nc,
                            self.config.host.clone().into(),
                            server_pub_key,
                            None,
                        )
                        .context("connect_finalize (TLS)")?
                    };
                    return Ok((conn, framed));
                }
                Err(e) => {
                    warn!("TLS handshake failed ({}); falling back to plain", e);
                    // can't reuse the moved tokio_tcp — fall through to plain below
                }
            }
        }

        // Phase 3 — plain (no TLS, or TLS failed)
        connector.mark_security_upgrade_as_done();
        let upgraded = ironrdp_blocking::mark_as_upgraded(
            ironrdp_blocking::skip_connect_begin(&mut connector),
            &mut connector,
        );
        let stream = pre_framed.into_inner_no_leftover();
        let mut framed = ironrdp_blocking::Framed::new(stream);
        let conn = {
            let mut nc = SimpleNetworkClient;
            ironrdp_blocking::connect_finalize(
                upgraded,
                connector,
                &mut framed,
                &mut nc,
                self.config.host.clone().into(),
                vec![],
                None,
            )
            .context("connect_finalize (plain)")?
        };
        Ok((conn, framed))
    }

    // ── SSH tunnel connection ─────────────────────────────────────────────────

    async fn connect_via_ssh_tunnel(
        &self,
        ssh_config: &SshTunnelConfig,
        rdp_addr: &str,
    ) -> Result<(
        ConnectionResult,
        ironrdp_blocking::Framed<Box<dyn RdpStream>>,
    )> {
        info!(
            "Connecting via SSH tunnel to {} (SSH: {}:{})",
            rdp_addr, ssh_config.hostname, ssh_config.port
        );

        let mut ssh_tunnel = SshTunnel::new(ssh_config.clone());
        ssh_tunnel.connect().await?;

        let parts: Vec<&str> = rdp_addr.split(':').collect();
        if parts.len() != 2 {
            let _ = ssh_tunnel.disconnect().await;
            return Err(anyhow::anyhow!("Invalid RDP address format: {}", rdp_addr));
        }
        let rdp_host = parts[0];
        let rdp_port: u16 = parts[1].parse()?;

        let ssh_stream = ssh_tunnel.create_tcp_stream(rdp_host, rdp_port).await?;
        info!("SSH TCP channel open to {}:{}", rdp_host, rdp_port);

        // SSH tunnel carries its own encryption so outer TLS is unnecessary;
        // run the plain IronRDP connection sequence directly.
        let client_addr = std::net::SocketAddr::V4(std::net::SocketAddrV4::new(
            std::net::Ipv4Addr::LOCALHOST,
            0,
        ));
        let mut connector =
            connector::ClientConnector::new(self.build_connector_config(), client_addr);

        let boxed: Box<dyn RdpStream> = Box::new(ssh_stream);
        let mut framed = ironrdp_blocking::Framed::new(boxed);

        ironrdp_blocking::connect_begin(&mut framed, &mut connector).context("connect_begin")?;
        connector.mark_security_upgrade_as_done();
        let upgraded = ironrdp_blocking::mark_as_upgraded(
            ironrdp_blocking::skip_connect_begin(&mut connector),
            &mut connector,
        );

        let conn = {
            let mut nc = SimpleNetworkClient;
            ironrdp_blocking::connect_finalize(
                upgraded,
                connector,
                &mut framed,
                &mut nc,
                self.config.host.clone().into(),
                vec![],
                None,
            )
            .context("connect_finalize (SSH)")?
        };
        Ok((conn, framed))
    }

    // ── Connector config ──────────────────────────────────────────────────────

    fn build_connector_config(&self) -> connector::Config {
        connector::Config {
            credentials: Credentials::UsernamePassword {
                username: self.config.username.clone(),
                password: self.config.password.clone(),
            },
            domain: self.config.domain.clone(),
            enable_tls: true,
            enable_credssp: true,
            keyboard_type: KeyboardType::IbmEnhanced,
            keyboard_subtype: 0,
            keyboard_layout: 0,
            keyboard_functional_keys_count: 12,
            ime_file_name: String::new(),
            dig_product_id: String::new(),
            desktop_size: connector::DesktopSize {
                width: self.config.width as u16,
                height: self.config.height as u16,
            },
            bitmap: None,
            client_build: 0,
            client_name: "tftsr-rdp-client".to_owned(),
            client_dir: "C:\\Windows\\System32\\mstscax.dll".to_owned(),
            platform: MajorPlatformType::UNIX,
            enable_server_pointer: true,
            request_data: None,
            autologon: false,
            enable_audio_playback: false,
            pointer_software_rendering: true,
            performance_flags: PerformanceFlags::default(),
            desktop_scale_factor: 0,
            hardware_id: None,
            license_cache: None,
            timezone_info: ironrdp_pdu::rdp::client_info::TimezoneInfo::default(),
        }
    }

    // ── Frame capture ─────────────────────────────────────────────────────────

    fn capture_and_send_frame(
        &self,
        image: &DecodedImage,
        broadcaster: &FrameBroadcaster,
    ) -> Result<()> {
        let frame_number = {
            let mut st = self.state.lock().unwrap();
            st.frames_sent += 1;
            st.last_frame_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            st.frames_sent
        };

        let frame = RdpFrame {
            width: image.width() as u32,
            height: image.height() as u32,
            data: image.data().to_vec(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            frame_number,
        };

        if broadcaster.try_send(frame).is_err() {
            debug!("Frame queue full, dropping frame");
        }
        Ok(())
    }

    // ── Input ─────────────────────────────────────────────────────────────────

    pub async fn send_keyboard_input(&self, keycode: u16, pressed: bool) -> Result<()> {
        debug!("Keyboard: keycode={}, pressed={}", keycode, pressed);
        let scancode = Scancode::from_u16(keycode);
        let op = if pressed {
            ironrdp_input::Operation::KeyPressed(scancode)
        } else {
            ironrdp_input::Operation::KeyReleased(scancode)
        };
        let events = self.input_state.lock().unwrap().apply(std::iter::once(op));
        if !events.is_empty() {
            self.input_tx.send(events).await.ok();
        }
        Ok(())
    }

    pub async fn send_mouse_input(&self, x: i16, y: i16, button: u16, pressed: bool) -> Result<()> {
        debug!(
            "Mouse: x={}, y={}, button={}, pressed={}",
            x, y, button, pressed
        );
        let mouse_button = MouseButton::from_native_button(button).unwrap_or(MouseButton::Left);
        let position = MousePosition {
            x: x as u16,
            y: y as u16,
        };
        let op = if pressed {
            ironrdp_input::Operation::MouseButtonPressed(mouse_button)
        } else {
            ironrdp_input::Operation::MouseButtonReleased(mouse_button)
        };
        let mut events = self.input_state.lock().unwrap().apply(std::iter::once(op));
        let move_op = ironrdp_input::Operation::MouseMove(position);
        events.extend(
            self.input_state
                .lock()
                .unwrap()
                .apply(std::iter::once(move_op)),
        );
        if !events.is_empty() {
            self.input_tx.send(events).await.ok();
        }
        Ok(())
    }

    /// Queue a dynamic resolution change; picked up on the next PDU loop tick.
    pub async fn request_resize(&self, width: u32, height: u32) -> Result<()> {
        self.resize_tx
            .send(ResizeRequest { width, height })
            .await
            .context("send resize request")
    }

    pub fn get_state(&self) -> RdpSessionState {
        self.state.lock().unwrap().clone()
    }
}

// ─── SyncTlsStream ────────────────────────────────────────────────────────────

/// Wraps an async TLS stream and presents it as sync `Read + Write`
/// by driving the current Tokio runtime handle with `block_on`.
struct SyncTlsStream {
    inner: tokio_rustls::client::TlsStream<tokio::net::TcpStream>,
    rt: tokio::runtime::Handle,
}

impl SyncTlsStream {
    fn new(inner: tokio_rustls::client::TlsStream<tokio::net::TcpStream>) -> Self {
        Self {
            inner,
            rt: tokio::runtime::Handle::current(),
        }
    }
}

impl Read for SyncTlsStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use tokio::io::AsyncReadExt;
        let inner = &mut self.inner;
        self.rt.block_on(async { inner.read(buf).await })
    }
}

impl Write for SyncTlsStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        use tokio::io::AsyncWriteExt;
        let inner = &mut self.inner;
        self.rt.block_on(async { inner.write(buf).await })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        use tokio::io::AsyncWriteExt;
        let inner = &mut self.inner;
        self.rt.block_on(async { inner.flush().await })
    }
}

// ─── RdpConnectionHandler ─────────────────────────────────────────────────────

pub struct RdpConnectionHandler {
    websocket_server: Arc<WebSocketServer>,
}

impl RdpConnectionHandler {
    pub fn new(websocket_server: Arc<WebSocketServer>) -> Self {
        Self { websocket_server }
    }

    pub fn create_session(&self, config: RdpConfig) -> Result<RdpClientSession> {
        RdpClientSession::new(config, self.websocket_server.clone())
    }

    /// Create a session bound to a caller-provided `session_id` so frames route
    /// to the matching WebSocket client (see `RdpClientSession::new_with_id`).
    pub fn create_session_with_id(
        &self,
        session_id: String,
        config: RdpConfig,
    ) -> Result<RdpClientSession> {
        RdpClientSession::new_with_id(session_id, config, self.websocket_server.clone())
    }

    pub async fn start_session(&self, session_id: &str, config: RdpConfig) -> Result<()> {
        info!("Starting RDP session: {}", session_id);
        // Bind the client session to the provided id so its frame-forwarding task
        // sends under the same id used for WebSocket registration/routing.
        let session = self.create_session_with_id(session_id.to_string(), config)?;
        session.connect().await?;
        info!("RDP session ended: {}", session_id);
        Ok(())
    }

    pub async fn stop_session(&self, session_id: &str) -> Result<()> {
        info!("Stopping RDP session: {}", session_id);
        self.websocket_server.unregister_session(session_id).await;
        Ok(())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> RdpConfig {
        RdpConfig {
            host: "127.0.0.1".to_string(),
            port: 3389,
            username: "test".to_string(),
            password: "test".to_string(),
            width: 1920,
            height: 1080,
            bit_depth: 32,
            domain: None,
            ssh_config: None,
        }
    }

    #[tokio::test]
    async fn test_rdp_session_creation() {
        let session = RdpClientSession::new(make_config(), Arc::new(WebSocketServer::new()));
        assert!(session.is_ok());
        let s = session.unwrap();
        assert!(!s.session_id.is_empty());
        assert_eq!(s.config.width, 1920);
        assert_eq!(s.config.height, 1080);
    }

    #[tokio::test]
    async fn test_rdp_connection_handler() {
        let handler = RdpConnectionHandler::new(Arc::new(WebSocketServer::new()));
        assert!(handler.create_session(make_config()).is_ok());
    }

    #[tokio::test]
    async fn test_build_connector_config() {
        let mut cfg = make_config();
        cfg.domain = Some("WORKGROUP".to_string());
        let session = RdpClientSession::new(cfg, Arc::new(WebSocketServer::new())).unwrap();
        let cc = session.build_connector_config();
        assert_eq!(cc.desktop_size.width, 1920);
        assert_eq!(cc.desktop_size.height, 1080);
        assert!(cc.enable_tls);
    }

    #[tokio::test]
    async fn test_resize_channel() {
        let session =
            RdpClientSession::new(make_config(), Arc::new(WebSocketServer::new())).unwrap();
        assert!(session.request_resize(1280, 720).await.is_ok());
    }

    #[tokio::test]
    async fn test_keyboard_input() {
        let session =
            RdpClientSession::new(make_config(), Arc::new(WebSocketServer::new())).unwrap();
        assert!(session.send_keyboard_input(65, true).await.is_ok());
        assert!(session.send_keyboard_input(65, false).await.is_ok());
    }

    #[tokio::test]
    async fn test_mouse_input() {
        let session =
            RdpClientSession::new(make_config(), Arc::new(WebSocketServer::new())).unwrap();
        assert!(session.send_mouse_input(100, 200, 1, true).await.is_ok());
        assert!(session.send_mouse_input(100, 200, 1, false).await.is_ok());
    }

    #[test]
    fn test_ssh_config_in_rdp_config() {
        let ssh = SshTunnelConfig {
            hostname: "gateway.example.com".to_string(),
            port: 22,
            username: "admin".to_string(),
            password: None,
            private_key_path: Some("/home/user/.ssh/id_ed25519".to_string()),
            private_key_data: None,
            key_passphrase: None,
        };
        let cfg = RdpConfig {
            host: "192.168.1.10".to_string(),
            port: 3389,
            username: "win_user".to_string(),
            password: "secret".to_string(),
            width: 1920,
            height: 1080,
            bit_depth: 32,
            domain: None,
            ssh_config: Some(ssh),
        };
        let sc = cfg.ssh_config.unwrap();
        assert_eq!(
            sc.private_key_path,
            Some("/home/user/.ssh/id_ed25519".to_string())
        );
        assert!(sc.password.is_none());
    }
}
