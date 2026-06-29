//! Live RDP integration/diagnostic test.
//!
//! This test performs a *real* RDP connection against a reachable host, streams
//! frames through the exact production pipeline (RDP capture → frame channel →
//! `WebSocketServer` → browser-style WebSocket client) and asserts that a
//! decoded frame contains **non-black** pixels — i.e. a real desktop, not the
//! blank canvas the black-screen bug produced.
//!
//! It is `#[ignore]`d so normal `cargo test` / CI runs skip it (no live host in
//! CI). Credentials are read from the environment so **no secrets are committed**:
//!
//! ```bash
//! TFTSR_RDP_TEST_HOST=172.0.1.42 \
//! TFTSR_RDP_TEST_USER=sarman \
//! TFTSR_RDP_TEST_PASS=*** \
//! cargo test --manifest-path src-tauri/Cargo.toml --test rdp_live -- --ignored --nocapture
//! ```

use std::sync::Arc;
use std::time::Duration;

use futures::{SinkExt as _, StreamExt as _};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use trcaa_lib::remote::input::RawInputEvent;
use trcaa_lib::remote::rdp_client::{RdpClientSession, RdpConfig};
use trcaa_lib::remote::websocket_server::WebSocketServer;

fn env_config() -> Option<RdpConfig> {
    let host = std::env::var("TFTSR_RDP_TEST_HOST").ok()?;
    let username = std::env::var("TFTSR_RDP_TEST_USER").ok()?;
    let password = std::env::var("TFTSR_RDP_TEST_PASS").ok()?;
    let port = std::env::var("TFTSR_RDP_TEST_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3389u16);
    let domain = std::env::var("TFTSR_RDP_TEST_DOMAIN").ok();
    let width = std::env::var("TFTSR_RDP_TEST_WIDTH")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1280u32);
    let height = std::env::var("TFTSR_RDP_TEST_HEIGHT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(720u32);

    Some(RdpConfig {
        host,
        port,
        username,
        password,
        width,
        height,
        bit_depth: 32,
        domain,
        ssh_config: None,
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires a live RDP host; set TFTSR_RDP_TEST_* env vars"]
async fn live_rdp_connect_renders_non_black_frame() {
    let Some(config) = env_config() else {
        eprintln!("SKIP: TFTSR_RDP_TEST_HOST/USER/PASS not set");
        return;
    };

    eprintln!(
        "Connecting to {}:{} as {:?} ({}x{})",
        config.host, config.port, config.username, config.width, config.height
    );

    // Stand up the production WebSocket pipeline.
    let ws = Arc::new(WebSocketServer::new());
    let port = ws.start_random_port().await.expect("ws start");
    let session_id = "live-test";
    ws.register_session(session_id, session_id).await;

    let session = Arc::new(
        RdpClientSession::new_with_id(session_id.to_string(), config, ws.clone())
            .expect("session construction"),
    );

    // Wire the inbound input path exactly like the production RDP manager so the
    // test exercises WS text frame -> handle_client -> input channel ->
    // handle_input -> session input methods -> RDP server.
    let (in_tx, mut in_rx) = mpsc::channel::<RawInputEvent>(64);
    ws.register_input_sender(session_id, in_tx).await;
    let in_session = session.clone();
    tokio::spawn(async move {
        while let Some(ev) = in_rx.recv().await {
            let _ = in_session.handle_input(ev).await;
        }
    });

    let connect_session = session.clone();
    let connect_handle = tokio::spawn(async move { connect_session.connect().await });

    // Give the negotiation a moment, then connect a browser-style WS client.
    let url = format!("ws://127.0.0.1:{port}/rdp/{session_id}");
    let mut client = None;
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_millis(250)).await;
        match tokio_tungstenite::connect_async(&url).await {
            Ok((stream, _)) => {
                eprintln!("WS client connected to {url}");
                client = Some(stream);
                break;
            }
            Err(e) => eprintln!("ws connect attempt failed: {e}"),
        }
        if connect_handle.is_finished() {
            break;
        }
    }
    let mut client = client.expect("WebSocket client failed to connect");

    // Inject input through the real WebSocket inbound path. This must not crash
    // the session; we assert below that frames keep flowing afterwards.
    for payload in [
        r#"{"type":"mouse_move","x":100,"y":100}"#,
        r#"{"type":"mouse","x":100,"y":100,"button":0,"pressed":true}"#,
        r#"{"type":"mouse","x":100,"y":100,"button":0,"pressed":false}"#,
        r#"{"type":"keyboard","code":"KeyA","pressed":true}"#,
        r#"{"type":"keyboard","code":"KeyA","pressed":false}"#,
    ] {
        client.send(Message::Text(payload.into())).await.ok();
    }

    // Pull frames and look for one with real (non-zero) pixel content.
    let mut best_nonzero = 0usize;
    let mut got_frame = false;
    let mut any_msg = 0usize;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(25);
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_secs(3), client.next()).await {
            Ok(Some(Ok(msg))) => {
                any_msg += 1;
                if !msg.is_binary() {
                    eprintln!("non-binary msg: {msg:?}");
                    continue;
                }
                let data = msg.into_data();
                if data.len() <= 8 {
                    continue;
                }
                let width = u32::from_le_bytes(data[0..4].try_into().unwrap());
                let height = u32::from_le_bytes(data[4..8].try_into().unwrap());
                let pixels = &data[8..];
                let nonzero = pixels.iter().filter(|&&b| b != 0).count();
                best_nonzero = best_nonzero.max(nonzero);
                got_frame = true;
                eprintln!(
                    "frame {}x{} bytes={} nonzero={}",
                    width,
                    height,
                    pixels.len(),
                    nonzero
                );
                if nonzero > 0 {
                    break;
                }
            }
            Ok(Some(Err(e))) => {
                eprintln!("ws client error: {e}");
                break;
            }
            Ok(None) => {
                eprintln!("ws stream ended");
                break;
            }
            Err(_) => {} // read timeout; keep trying until deadline
        }
    }

    let st = session.get_state();
    eprintln!(
        "RESULT: connected={} frames_sent={} got_frame={} any_msg={} best_nonzero={}",
        st.connected, st.frames_sent, got_frame, any_msg, best_nonzero
    );

    connect_handle.abort();

    assert!(st.connected, "RDP negotiation did not complete");
    assert!(got_frame, "no frames reached the WebSocket client");
    assert!(
        best_nonzero > 0,
        "frames were entirely black ({best_nonzero} non-zero bytes) - black-screen bug"
    );
}
