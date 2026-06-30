//! RDP Client using IronRDP
//!
//! Full RDP protocol connection with TLS, SSH tunnel support, frame capture,
//! input handling, and dynamic resolution via Display Control Virtual Channel.

use super::input::{clamp_coord, scancode_for_code, RawInputEvent};
use super::ssh_tunnel::{SshTunnel, SshTunnelConfig};
use super::websocket_server::{RdpFrame, WebSocketServer};
use anyhow::{Context, Result};
use ironrdp::connector::{self, ConnectionResult, Credentials};
use ironrdp::pdu::gcc::KeyboardType;
use ironrdp::pdu::rdp::capability_sets::MajorPlatformType;
use ironrdp::pdu::rdp::client_info::PerformanceFlags;
use ironrdp::session::image::DecodedImage;
use ironrdp::session::{ActiveStage, ActiveStageOutput};
use ironrdp_input::{MouseButton, MousePosition, Scancode, WheelRotations};
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

/// Install the process-wide rustls `CryptoProvider` exactly once.
///
/// rustls 0.23 refuses to pick a provider automatically when more than one
/// (`ring` and `aws-lc-rs`) is present in the dependency tree, panicking inside
/// the TLS handshake. That panic kills the RDP connect task and leaves the
/// canvas black. We pin `aws-lc-rs` (the feature this crate enables) and ignore
/// the error returned when a provider is already installed.
fn ensure_crypto_provider() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

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
        // Forward captured frames to the WebSocket layer.
        //
        // This intentionally polls with `try_recv` rather than awaiting
        // `recv().await`. The RDP session loop in `connect()` performs *blocking*
        // socket reads directly on its tokio worker and never yields while a
        // burst of frames is decoded. A channel `recv()` wakeup raised from that
        // blocked worker is enqueued on its local run-queue, which it cannot
        // drain until the blocking read returns (up to the socket read timeout),
        // and work-stealing does not reliably pick it up in the meantime — so the
        // forwarder would stall and the client would only ever see a black
        // canvas. Polling decouples frame delivery from that starved waker.
        tokio::spawn(async move {
            loop {
                match frame_rx.try_recv() {
                    Ok(frame) => {
                        if let Err(e) = ws.send_frame(&sid, frame).await {
                            warn!("Failed to send frame: {}", e);
                        }
                    }
                    Err(mpsc::error::TryRecvError::Empty) => {
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => break,
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
        ensure_crypto_provider();
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
        // Phase 1 — open raw TCP and run connect_begin (pre-upgrade steps).
        //
        // A long read timeout is required for the *blocking* negotiation and
        // CredSSP/TLS handshake reads — `connect_begin`/`connect_finalize` do not
        // tolerate spurious `WouldBlock`/`TimedOut` errors the way the active
        // loop does. Once the session is established we shorten it (see below) so
        // the input/resize loop stays responsive while no graphics are arriving.
        let std_tcp = TcpStream::connect(addr).context(format!("TCP connect to {}", addr))?;
        std_tcp
            .set_read_timeout(Some(Duration::from_secs(30)))
            .context("set_read_timeout")?;

        // Keep a handle to the raw socket so we can retune its read timeout after
        // the handshake completes (the socket is otherwise owned by the TLS
        // stream inside `framed`).
        let timeout_socket = std_tcp.try_clone().context("clone TcpStream")?;

        let client_addr = std_tcp.local_addr().context("get local addr")?;
        let mut connector =
            connector::ClientConnector::new(self.build_connector_config(), client_addr);

        let tcp_clone = std_tcp.try_clone().context("clone TcpStream")?;
        let boxed_pre: Box<dyn RdpStream> = Box::new(tcp_clone);
        let mut pre_framed = ironrdp_blocking::Framed::new(boxed_pre);

        // `connect_begin` drives the X.224/negotiation exchange and only returns
        // once the server has requested the security (TLS) upgrade, handing back
        // a `ShouldUpgrade` token. We MUST thread that token into
        // `mark_as_upgraded` after the TLS handshake — calling
        // `mark_security_upgrade_as_done()` ourselves and then
        // `skip_connect_begin()` trips an assertion (the upgrade already looks
        // done) and panics the connect task, leaving a black canvas.
        let should_upgrade = ironrdp_blocking::connect_begin(&mut pre_framed, &mut connector)
            .context("connect_begin")?;

        // The pre-upgrade framed reader owns a clone of the socket; drop it so
        // the TLS stream is the sole owner before we take `std_tcp` by value.
        drop(pre_framed);

        // Phase 2 — TLS upgrade (RDP enhanced security is mandatory on modern
        // servers, and `connect_begin` only returns once it is requested).
        //
        // IronRDP's blocking connector wants a *synchronous* Read+Write stream.
        // We therefore perform the rustls handshake on the blocking socket
        // directly. The previous implementation upgraded with the async
        // `ironrdp_tls::upgrade` and wrapped the result in a stream that called
        // `Handle::block_on` for every read/write — but that runs inside the
        // tokio worker driving `connect()`, so it panicked ("Cannot start a
        // runtime from within a runtime") and left the canvas black.
        info!("Upgrading RDP transport to TLS for {}", self.config.host);

        let (tls_stream, server_pub_key) = blocking_tls_upgrade(std_tcp, &self.config.host)?;
        info!("TLS handshake OK, public key {} B", server_pub_key.len());

        let upgraded = ironrdp_blocking::mark_as_upgraded(should_upgrade, &mut connector);

        let boxed: Box<dyn RdpStream> = Box::new(tls_stream);
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

        // Handshake done: shorten the read timeout so the active loop wakes
        // frequently to drain queued input/resize events even when the server is
        // sending no graphics updates (otherwise input could stall for up to the
        // full timeout). Buffered partial PDUs are preserved across these short
        // timeouts by `ironrdp_blocking::Framed`, so this is safe.
        if let Err(e) = timeout_socket.set_read_timeout(Some(Duration::from_millis(50))) {
            warn!("Could not shorten read timeout for input loop: {}", e);
        }

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

        let should_upgrade = ironrdp_blocking::connect_begin(&mut framed, &mut connector)
            .context("connect_begin")?;
        let upgraded = ironrdp_blocking::mark_as_upgraded(should_upgrade, &mut connector);

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
            alternate_shell: String::new(),
            work_dir: String::new(),
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
            compression_type: None,
            multitransport_flags: None,
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

    /// Translate a decoded browser input event and queue it for the session
    /// loop. This is the single entry point used by the WebSocket input bridge.
    pub async fn handle_input(&self, event: RawInputEvent) -> Result<()> {
        match event {
            RawInputEvent::Keyboard { code, pressed } => {
                if let Some(scancode) = scancode_for_code(&code) {
                    self.send_keyboard_input(scancode, pressed).await?;
                } else {
                    debug!("Ignoring unmapped key code: {}", code);
                }
            }
            RawInputEvent::MouseMove { x, y } => {
                self.send_mouse_move(clamp_coord(x), clamp_coord(y)).await?;
            }
            RawInputEvent::Mouse {
                x,
                y,
                button,
                pressed,
            } => {
                self.send_mouse_input(clamp_coord(x), clamp_coord(y), button, pressed)
                    .await?;
            }
            RawInputEvent::Wheel { x, y, delta } => {
                self.send_mouse_wheel(clamp_coord(x), clamp_coord(y), delta)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn send_keyboard_input(&self, keycode: u16, pressed: bool) -> Result<()> {
        debug!("Keyboard: keycode={:#06x}, pressed={}", keycode, pressed);
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

    /// Move the pointer without changing button state.
    pub async fn send_mouse_move(&self, x: u16, y: u16) -> Result<()> {
        let op = ironrdp_input::Operation::MouseMove(MousePosition { x, y });
        let events = self.input_state.lock().unwrap().apply(std::iter::once(op));
        if !events.is_empty() {
            self.input_tx.send(events).await.ok();
        }
        Ok(())
    }

    pub async fn send_mouse_input(&self, x: u16, y: u16, button: u16, pressed: bool) -> Result<()> {
        debug!(
            "Mouse: x={}, y={}, button={}, pressed={}",
            x, y, button, pressed
        );
        let mouse_button = MouseButton::from_web_button(button as u8).unwrap_or(MouseButton::Left);
        // Position first so the button transition is applied at the cursor's
        // current location, then the press/release itself.
        let move_op = ironrdp_input::Operation::MouseMove(MousePosition { x, y });
        let button_op = if pressed {
            ironrdp_input::Operation::MouseButtonPressed(mouse_button)
        } else {
            ironrdp_input::Operation::MouseButtonReleased(mouse_button)
        };
        let events = {
            let mut db = self.input_state.lock().unwrap();
            db.apply([move_op, button_op])
        };
        if !events.is_empty() {
            self.input_tx.send(events).await.ok();
        }
        Ok(())
    }

    /// Vertical mouse wheel. `delta` follows the DOM sign convention
    /// (positive = scroll towards the user); RDP expects the opposite sign.
    pub async fn send_mouse_wheel(&self, x: u16, y: u16, delta: i32) -> Result<()> {
        // RDP wheel rotation sign is opposite the DOM deltaY. Use saturating_neg
        // so an attacker-controlled delta of i32::MIN cannot trigger an overflow
        // panic in debug builds.
        let rotation_units = delta
            .saturating_neg()
            .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        let move_op = ironrdp_input::Operation::MouseMove(MousePosition { x, y });
        let wheel_op = ironrdp_input::Operation::WheelRotations(WheelRotations {
            is_vertical: true,
            rotation_units,
        });
        let events = {
            let mut db = self.input_state.lock().unwrap();
            db.apply([move_op, wheel_op])
        };
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

// ─── Blocking TLS upgrade ─────────────────────────────────────────────────────

/// Perform a synchronous rustls TLS handshake over a blocking `TcpStream` and
/// return the resulting blocking stream plus the server's TLS public key.
///
/// IronRDP's `ironrdp_blocking` connector reads/writes synchronously, so the
/// transport must be a real blocking stream — not an async stream pumped via
/// `block_on` (which panics inside the tokio worker running the session).
///
/// The certificate is intentionally **not** validated: RDP servers routinely
/// present self-signed certificates, and IronRDP binds the channel to the
/// server's public key during CredSSP instead of relying on PKI trust.
fn blocking_tls_upgrade(
    tcp: TcpStream,
    server_name: &str,
) -> Result<(
    rustls::StreamOwned<rustls::ClientConnection, TcpStream>,
    Vec<u8>,
)> {
    let mut config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoCertificateVerification))
        .with_no_client_auth();

    // CredSSP does not support TLS session resumption.
    config.resumption = rustls::client::Resumption::disabled();

    let domain = rustls::pki_types::ServerName::try_from(server_name.to_owned())
        .map_err(|e| anyhow::anyhow!("invalid TLS server name '{server_name}': {e}"))?;

    let mut conn = rustls::ClientConnection::new(Arc::new(config), domain)
        .context("create rustls client connection")?;

    let mut tcp = tcp;
    // Drive the handshake to completion synchronously so peer certificates are
    // available before we hand the stream to IronRDP.
    conn.complete_io(&mut tcp).context("TLS handshake")?;

    let server_pub_key = {
        use x509_cert::der::Decode as _;
        match conn.peer_certificates().and_then(|certs| certs.first()) {
            Some(cert_der) => {
                let cert =
                    x509_cert::Certificate::from_der(cert_der).context("parse peer certificate")?;
                ironrdp_tls::extract_tls_server_public_key(&cert)
                    .map(|k| k.to_vec())
                    .unwrap_or_default()
            }
            None => Vec::new(),
        }
    };

    Ok((rustls::StreamOwned::new(conn, tcp), server_pub_key))
}

/// rustls verifier that accepts any server certificate (see `blocking_tls_upgrade`).
#[derive(Debug)]
struct NoCertificateVerification;

impl rustls::client::danger::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        use rustls::SignatureScheme;
        vec![
            SignatureScheme::RSA_PKCS1_SHA1,
            SignatureScheme::ECDSA_SHA1_Legacy,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
        ]
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
