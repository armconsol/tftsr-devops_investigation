// VNC console module
//
// Provides the building blocks for an in-app noVNC graphical console:
//   1. `vncproxy` — ask PVE to open a VNC proxy and return a one-time ticket
//      + a TCP port on the node.
//   2. A small local WebSocket proxy that bridges the in-app noVNC client
//      (which connects to `ws://127.0.0.1:<localport>`) to the PVE
//      `vncwebsocket` endpoint (`wss://host:port/.../vncwebsocket`), injecting
//      the `PVEAuthCookie` and accepting the node's self-signed TLS cert.

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::COOKIE;
use tokio_tungstenite::tungstenite::Message;

/// Information returned by the PVE `vncproxy` API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VncProxyInfo {
    /// One-time VNC ticket — used by noVNC as the RFB password.
    pub ticket: String,
    /// TCP port opened on the node for the proxied VNC session.
    pub port: String,
    /// The user the session belongs to.
    #[serde(default)]
    pub user: String,
    /// The task UPID (if returned).
    #[serde(default)]
    pub upid: String,
    /// Separate RFB/VNC password (PVE `vncshell` websocket mode returns this in
    /// addition to `ticket`; for qemu/lxc `vncproxy` it is absent and the
    /// `ticket` doubles as the password).
    #[serde(default)]
    pub password: Option<String>,
}

/// Parse the `vncproxy` response (the inner `data` object) into `VncProxyInfo`.
///
/// PVE returns `port` as either a string or an integer depending on version, so
/// we normalise it to a string. A missing ticket or port is an error.
pub fn parse_vncproxy_response(data: &serde_json::Value) -> Result<VncProxyInfo, String> {
    let ticket = data
        .get("ticket")
        .and_then(|t| t.as_str())
        .filter(|t| !t.is_empty())
        .ok_or_else(|| "vncproxy response missing 'ticket'".to_string())?
        .to_string();

    let port = match data.get("port") {
        Some(serde_json::Value::String(s)) if !s.is_empty() => s.clone(),
        Some(serde_json::Value::Number(n)) => n.to_string(),
        _ => return Err("vncproxy response missing 'port'".to_string()),
    };

    let user = data
        .get("user")
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();
    let upid = data
        .get("upid")
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();

    let password = data
        .get("password")
        .and_then(|p| p.as_str())
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string());

    Ok(VncProxyInfo {
        ticket,
        port,
        user,
        upid,
        password,
    })
}

/// Build the PVE `vncwebsocket` URL the proxy dials upstream.
///
/// `vnc_port` and `vnc_ticket` come from the `vncproxy` call. The ticket must
/// be URL-encoded because it contains characters such as `:` and `/`.
pub fn build_vncwebsocket_url(
    host: &str,
    port: u16,
    node: &str,
    vmid: u32,
    vnc_port: &str,
    vnc_ticket: &str,
) -> String {
    format!(
        "wss://{}:{}/api2/json/nodes/{}/qemu/{}/vncwebsocket?port={}&vncticket={}",
        host,
        port,
        node,
        vmid,
        urlencoding::encode(vnc_port),
        urlencoding::encode(vnc_ticket),
    )
}

/// Build the same URL for an LXC container console.
pub fn build_lxc_vncwebsocket_url(
    host: &str,
    port: u16,
    node: &str,
    vmid: u32,
    vnc_port: &str,
    vnc_ticket: &str,
) -> String {
    format!(
        "wss://{}:{}/api2/json/nodes/{}/lxc/{}/vncwebsocket?port={}&vncticket={}",
        host,
        port,
        node,
        vmid,
        urlencoding::encode(vnc_port),
        urlencoding::encode(vnc_ticket),
    )
}

/// Build the PVE/PBS node-shell `vncwebsocket` URL the proxy dials upstream.
pub fn build_node_vncwebsocket_url(
    host: &str,
    port: u16,
    node: &str,
    vnc_port: &str,
    vnc_ticket: &str,
) -> String {
    format!(
        "wss://{}:{}/api2/json/nodes/{}/vncwebsocket?port={}&vncticket={}",
        host,
        port,
        node,
        urlencoding::encode(vnc_port),
        urlencoding::encode(vnc_ticket),
    )
}

/// Build the `Cookie` header value carrying the Proxmox auth ticket. The cookie
/// name differs by product: `PVEAuthCookie` for PVE, `PBSAuthCookie` for PBS.
pub fn build_auth_cookie(cookie_name: &str, auth_ticket: &str) -> String {
    format!("{}={}", cookie_name, auth_ticket)
}

/// Normalise a TLS fingerprint to a bare lower-case hex string (strips colons,
/// whitespace, and case) so two representations can be compared safely.
fn normalize_fp(fp: &str) -> String {
    fp.chars()
        .filter(|c| c.is_ascii_hexdigit())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Verify a peer certificate (DER bytes) against an expected SHA-256
/// fingerprint. The expected fingerprint may be in Proxmox's canonical
/// colon-separated upper-case form or any equivalent hex representation.
///
/// Returns `Ok(())` on match, or an error describing the mismatch.
pub fn verify_cert_fingerprint(expected: &str, der: &[u8]) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let actual = hex::encode(Sha256::digest(der));
    let expected_norm = normalize_fp(expected);
    if expected_norm.is_empty() {
        return Err("Expected fingerprint is empty".to_string());
    }
    if actual == expected_norm {
        Ok(())
    } else {
        Err(format!(
            "TLS fingerprint mismatch: expected {}, got {}",
            expected_norm, actual
        ))
    }
}

/// Details handed back to the frontend so noVNC can connect to the local proxy.
#[derive(Debug, Clone, Serialize)]
pub struct VncConsoleSession {
    /// Local websocket URL the noVNC client connects to.
    pub local_url: String,
    /// VNC ticket — passed to noVNC as the RFB password.
    pub ticket: String,
    /// Bound local port (for diagnostics).
    pub local_port: u16,
}

/// Start a local WebSocket proxy that bridges a single noVNC connection to the
/// PVE `vncwebsocket` endpoint. Binds to an ephemeral port on 127.0.0.1, spawns
/// a background task to accept exactly one client, and returns the local URL.
///
/// The upstream connection injects the `PVEAuthCookie` header and accepts the
/// node's self-signed TLS certificate.
pub async fn start_vnc_proxy(
    upstream_url: String,
    cookie_name: String,
    auth_ticket: String,
    vnc_ticket: String,
    expected_fingerprint: Option<String>,
) -> Result<VncConsoleSession, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind local VNC proxy socket: {}", e))?;
    let local_port = listener
        .local_addr()
        .map_err(|e| format!("Failed to read local proxy address: {}", e))?
        .port();

    let cookie = build_auth_cookie(&cookie_name, &auth_ticket);

    tokio::spawn(async move {
        // Accept exactly one inbound noVNC connection, then bridge it. A bounded
        // timeout ensures the listener socket and this task are released if the
        // client never connects (e.g. the user closed the console tab).
        match tokio::time::timeout(std::time::Duration::from_secs(30), listener.accept()).await {
            Ok(Ok((stream, _addr))) => {
                if let Err(e) = bridge_connection(
                    stream,
                    &upstream_url,
                    &cookie,
                    expected_fingerprint.as_deref(),
                )
                .await
                {
                    tracing::warn!("VNC proxy bridge ended: {}", e);
                }
            }
            Ok(Err(e)) => tracing::warn!("VNC proxy failed to accept connection: {}", e),
            Err(_) => tracing::warn!("VNC proxy timed out waiting for a client; releasing socket"),
        }
    });

    Ok(VncConsoleSession {
        local_url: format!("ws://127.0.0.1:{}", local_port),
        ticket: vnc_ticket,
        local_port,
    })
}

/// Bridge a single accepted TCP stream (a noVNC websocket client) to the PVE
/// upstream `vncwebsocket`, piping binary frames in both directions.
async fn bridge_connection(
    inbound: tokio::net::TcpStream,
    upstream_url: &str,
    cookie: &str,
    expected_fingerprint: Option<&str>,
) -> Result<(), String> {
    // Accept the inbound websocket from noVNC.
    let inbound_ws = tokio_tungstenite::accept_async(inbound)
        .await
        .map_err(|e| format!("Failed to accept inbound websocket: {}", e))?;

    // Build the upstream request with the auth cookie header.
    let mut request = upstream_url
        .into_client_request()
        .map_err(|e| format!("Invalid upstream URL: {}", e))?;
    request.headers_mut().insert(
        COOKIE,
        cookie
            .parse()
            .map_err(|_| "Failed to build cookie header".to_string())?,
    );

    // Connect upstream. Proxmox nodes use self-signed certificates by default,
    // so the TLS connector tolerates them; when the cluster has a stored
    // `ssl_fingerprint` we additionally pin the peer certificate's SHA-256
    // against it to detect MITM / wrong-host connections.
    let tls = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .map_err(|e| format!("Failed to build TLS connector: {}", e))?;
    let connector = tokio_tungstenite::Connector::NativeTls(tls);

    let (upstream_ws, _resp) =
        tokio_tungstenite::connect_async_tls_with_config(request, None, false, Some(connector))
            .await
            .map_err(|e| format!("Failed to connect to PVE vncwebsocket: {}", e))?;

    // Pin the peer certificate fingerprint when one is configured.
    if let Some(expected) = expected_fingerprint.filter(|f| !f.trim().is_empty()) {
        let der = match upstream_ws.get_ref() {
            tokio_tungstenite::MaybeTlsStream::NativeTls(s) => s
                .get_ref()
                .peer_certificate()
                .map_err(|e| format!("Failed to read peer certificate: {}", e))?
                .map(|c| c.to_der().map_err(|e| e.to_string()))
                .transpose()?,
            _ => None,
        };
        let der = der.ok_or_else(|| {
            "No peer certificate available to verify the configured fingerprint".to_string()
        })?;
        verify_cert_fingerprint(expected, &der)?;
    }

    let (mut in_tx, mut in_rx) = inbound_ws.split();
    let (mut up_tx, mut up_rx) = upstream_ws.split();

    // noVNC -> PVE
    let client_to_server = async {
        while let Some(msg) = in_rx.next().await {
            match msg {
                Ok(m) => {
                    if up_tx.send(m).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = up_tx.send(Message::Close(None)).await;
    };

    // PVE -> noVNC
    let server_to_client = async {
        while let Some(msg) = up_rx.next().await {
            match msg {
                Ok(m) => {
                    if in_tx.send(m).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = in_tx.send(Message::Close(None)).await;
    };

    tokio::select! {
        _ = client_to_server => {},
        _ = server_to_client => {},
    }

    Ok(())
}

/// Request a VNC proxy for a QEMU VM.
/// POST /nodes/{node}/qemu/{vmid}/vncproxy
pub async fn vncproxy_vm(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<VncProxyInfo, String> {
    let path = format!("nodes/{}/qemu/{}/vncproxy", node, vmid);
    let params: &[(&str, &str)] = &[("websocket", "1")];
    let response: serde_json::Value = client
        .post_form(&path, params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to open VNC proxy for VM {}: {}", vmid, e))?;
    parse_vncproxy_response(&response)
}

/// Request a VNC proxy for an LXC container.
/// POST /nodes/{node}/lxc/{vmid}/vncproxy
pub async fn vncproxy_lxc(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    vmid: u32,
    ticket: &str,
) -> Result<VncProxyInfo, String> {
    let path = format!("nodes/{}/lxc/{}/vncproxy", node, vmid);
    let params: &[(&str, &str)] = &[("websocket", "1")];
    let response: serde_json::Value = client
        .post_form(&path, params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to open VNC proxy for container {}: {}", vmid, e))?;
    parse_vncproxy_response(&response)
}

/// Request a graphical VNC shell for a PVE node (host shell).
/// POST /nodes/{node}/vncshell
pub async fn vncshell_node(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<VncProxyInfo, String> {
    let path = format!("nodes/{}/vncshell", node);
    let params: &[(&str, &str)] = &[("websocket", "1")];
    let response: serde_json::Value = client
        .post_form(&path, params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to open node shell for {}: {}", node, e))?;
    parse_vncproxy_response(&response)
}

/// Request an xterm.js terminal proxy for a node (used for PBS host shell).
/// POST /nodes/{node}/termproxy
pub async fn termproxy_node(
    client: &crate::proxmox::client::ProxmoxClient,
    node: &str,
    ticket: &str,
) -> Result<VncProxyInfo, String> {
    let path = format!("nodes/{}/termproxy", node);
    let params: &[(&str, &str)] = &[];
    let response: serde_json::Value = client
        .post_form(&path, params, Some(ticket))
        .await
        .map_err(|e| format!("Failed to open terminal proxy for {}: {}", node, e))?;
    parse_vncproxy_response(&response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// The in-app noVNC/xterm consoles connect to the local proxy over
    /// `ws://127.0.0.1:<ephemeral-port>`. WebKitGTK throws
    /// `SecurityError: The operation is insecure` when a WebSocket violates the
    /// app CSP `connect-src`, so the bundled config must explicitly allow
    /// loopback `ws://`. This guard fails if that allowance is ever dropped.
    #[test]
    fn test_capabilities_allow_clipboard_text() {
        // Console copy/paste relies on tauri-plugin-clipboard-manager. The
        // bundled capability set must grant text read+write or the frontend
        // clipboard calls are rejected by the Tauri ACL at runtime.
        let caps = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/capabilities/default.json"
        ));
        let parsed: serde_json::Value =
            serde_json::from_str(caps).expect("capabilities/default.json must be valid JSON");
        let perms = parsed
            .get("permissions")
            .and_then(|p| p.as_array())
            .expect("capabilities must define a permissions array");
        let has = |needle: &str| perms.iter().filter_map(|v| v.as_str()).any(|p| p == needle);
        assert!(
            has("clipboard-manager:allow-read-text"),
            "capabilities must grant clipboard-manager:allow-read-text for console paste"
        );
        assert!(
            has("clipboard-manager:allow-write-text"),
            "capabilities must grant clipboard-manager:allow-write-text for console copy"
        );
    }

    #[test]
    fn test_csp_allows_loopback_websocket() {
        let conf = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json"));
        let parsed: serde_json::Value =
            serde_json::from_str(conf).expect("tauri.conf.json must be valid JSON");
        let csp = parsed
            .pointer("/app/security/csp")
            .and_then(|v| v.as_str())
            .expect("tauri.conf.json must define app.security.csp");
        assert!(
            csp.contains("ws://127.0.0.1:*"),
            "CSP connect-src must allow ws://127.0.0.1:* for the local console proxy; got: {csp}"
        );
        assert!(
            csp.contains("ws://localhost:*"),
            "CSP connect-src must allow ws://localhost:* for the local console proxy; got: {csp}"
        );
    }

    #[test]
    fn test_parse_vncproxy_string_port() {
        let data = json!({
            "ticket": "PVEVNC:ABC123",
            "port": "5900",
            "user": "root@pam",
            "upid": "UPID:pve:..."
        });
        let info = parse_vncproxy_response(&data).unwrap();
        assert_eq!(info.ticket, "PVEVNC:ABC123");
        assert_eq!(info.port, "5900");
        assert_eq!(info.user, "root@pam");
        assert_eq!(info.upid, "UPID:pve:...");
        assert_eq!(info.password, None);
    }

    #[test]
    fn test_parse_vncshell_with_password() {
        // PVE vncshell (websocket=1) returns a separate `password` for RFB.
        let data = json!({
            "ticket": "PVEVNC:SHELL",
            "port": "5900",
            "user": "root@pam",
            "cert": "-----BEGIN CERTIFICATE-----",
            "password": "s3cr3t-rfb"
        });
        let info = parse_vncproxy_response(&data).unwrap();
        assert_eq!(info.ticket, "PVEVNC:SHELL");
        assert_eq!(info.password.as_deref(), Some("s3cr3t-rfb"));
    }

    #[test]
    fn test_parse_termproxy_response() {
        // PBS termproxy returns ticket/port/user/upid, no password.
        let data = json!({
            "ticket": "PBS:TERM",
            "port": 5900,
            "user": "root@pam",
            "upid": "UPID:pbs:..."
        });
        let info = parse_vncproxy_response(&data).unwrap();
        assert_eq!(info.ticket, "PBS:TERM");
        assert_eq!(info.port, "5900");
        assert_eq!(info.password, None);
    }

    #[test]
    fn test_parse_vncproxy_numeric_port() {
        let data = json!({ "ticket": "T", "port": 5901 });
        let info = parse_vncproxy_response(&data).unwrap();
        assert_eq!(info.port, "5901");
        assert_eq!(info.user, "");
    }

    #[test]
    fn test_parse_vncproxy_missing_ticket() {
        let data = json!({ "port": "5900" });
        assert!(parse_vncproxy_response(&data).is_err());
    }

    #[test]
    fn test_parse_vncproxy_missing_port() {
        let data = json!({ "ticket": "T" });
        assert!(parse_vncproxy_response(&data).is_err());
    }

    #[test]
    fn test_build_vncwebsocket_url_encodes_ticket() {
        let url = build_vncwebsocket_url("172.0.0.21", 8006, "pve", 100, "5900", "PVEVNC:AB/CD+EF");
        assert_eq!(
            url,
            "wss://172.0.0.21:8006/api2/json/nodes/pve/qemu/100/vncwebsocket?port=5900&vncticket=PVEVNC%3AAB%2FCD%2BEF"
        );
    }

    #[test]
    fn test_build_lxc_vncwebsocket_url() {
        let url = build_lxc_vncwebsocket_url("h", 8006, "pve", 200, "5901", "T");
        assert_eq!(
            url,
            "wss://h:8006/api2/json/nodes/pve/lxc/200/vncwebsocket?port=5901&vncticket=T"
        );
    }

    #[test]
    fn test_build_node_vncwebsocket_url() {
        let url =
            build_node_vncwebsocket_url("172.0.0.21", 8006, "vmhost4", "5900", "PVEVNC:AB/CD");
        assert_eq!(
            url,
            "wss://172.0.0.21:8006/api2/json/nodes/vmhost4/vncwebsocket?port=5900&vncticket=PVEVNC%3AAB%2FCD"
        );
    }

    #[test]
    fn test_build_auth_cookie() {
        assert_eq!(
            build_auth_cookie("PVEAuthCookie", "XYZ"),
            "PVEAuthCookie=XYZ"
        );
        assert_eq!(
            build_auth_cookie("PBSAuthCookie", "XYZ"),
            "PBSAuthCookie=XYZ"
        );
    }

    #[test]
    fn test_verify_cert_fingerprint_match() {
        use sha2::{Digest, Sha256};
        let der = b"fake-certificate-bytes";
        let digest = hex::encode(Sha256::digest(der));
        // Canonical Proxmox form: upper-case, colon-separated pairs.
        let canonical = digest
            .as_bytes()
            .chunks(2)
            .map(|c| String::from_utf8_lossy(c).to_uppercase())
            .collect::<Vec<_>>()
            .join(":");
        assert!(verify_cert_fingerprint(&canonical, der).is_ok());
        // A plain lower-case hex string must also match.
        assert!(verify_cert_fingerprint(&digest, der).is_ok());
    }

    #[test]
    fn test_verify_cert_fingerprint_mismatch() {
        let der = b"the-real-cert";
        let wrong = "AA:BB:CC:DD";
        assert!(verify_cert_fingerprint(wrong, der).is_err());
    }

    #[test]
    fn test_verify_cert_fingerprint_empty_expected() {
        assert!(verify_cert_fingerprint("   ", b"anything").is_err());
    }
}
