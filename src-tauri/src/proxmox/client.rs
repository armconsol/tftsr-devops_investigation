use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Proxmox VE/PBS API client
/// Implements authentication and request handling for Proxmox APIs
pub struct ProxmoxClient {
    base_url: String,
    port: u16,
    username: String,
    api_token: Option<String>,
    pub ticket: Option<String>,
    pub csrf_token: Option<String>,
    /// When the current ticket was obtained — PVE tickets expire after ~2h,
    /// so long-lived cached clients must re-authenticate periodically.
    ticket_created: Option<std::time::Instant>,
    client: Client,
}

/// Re-authenticate once a ticket is older than this. PVE tickets live ~2h;
/// refreshing at 90 minutes leaves a comfortable margin.
const TICKET_REFRESH_AFTER_SECS: u64 = 90 * 60;

/// Whether a ticket of the given age should be refreshed.
pub fn ticket_age_requires_refresh(age_secs: u64) -> bool {
    age_secs >= TICKET_REFRESH_AFTER_SECS
}

/// Format an error with its full source chain. reqwest's top-level Display
/// (e.g. "error sending request for url (…)") hides the actual cause —
/// timeout, connection reset, TLS failure — which lives in the source chain.
pub fn describe_error_chain(e: &(dyn std::error::Error + 'static)) -> String {
    let mut msg = e.to_string();
    let mut source = e.source();
    while let Some(s) = source {
        msg.push_str(": ");
        msg.push_str(&s.to_string());
        source = s.source();
    }
    msg
}

/// Transport-level failures worth one retry on idempotent requests: a pooled
/// keep-alive connection that the server (pveproxy closes idle connections
/// aggressively) tore down surfaces as a connect/request error on first use.
fn is_transient_transport_error(e: &reqwest::Error) -> bool {
    e.is_connect() || e.is_timeout() || e.is_request()
}

/// Outer envelope wrapping every Proxmox API response.
#[derive(Debug, Deserialize)]
struct ProxmoxEnvelope<T> {
    data: T,
}

/// Authentication response from Proxmox (inner `data` object).
#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    /// Cookie value — `PVEAuthCookie=<ticket>`.
    pub ticket: String,
    pub username: String,
    /// Seconds since epoch when the ticket expires.
    #[serde(default)]
    pub expire: u64,
    /// Required on mutating requests as `CSRFPreventionToken` header.
    #[serde(rename = "CSRFPreventionToken")]
    pub csrf_prevention_token: Option<String>,
    /// Capability map — structure varies, only needed for display/debug.
    #[serde(default)]
    pub cap: Option<serde_json::Value>,
    /// Cluster name
    #[serde(default)]
    pub clustername: Option<String>,
}

/// API token for authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiToken {
    pub token_id: String,
    pub name: String,
    pub expire: u64,
    pub permissions: Vec<String>,
}

impl ProxmoxClient {
    /// Create a new Proxmox client
    pub fn new(base_url: &str, port: u16, username: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            port,
            username: username.to_string(),
            api_token: None,
            ticket: None,
            csrf_token: None,
            ticket_created: None,
            client: Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(Duration::from_secs(30))
                // pveproxy closes idle keep-alive connections after a few
                // seconds; keeping stale pooled connections around makes the
                // first request after idle fail with an opaque send error.
                .pool_idle_timeout(Duration::from_secs(5))
                .tcp_keepalive(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Set the ticket for cookie-based authentication.
    pub fn set_ticket(&mut self, ticket: &str) {
        self.ticket = Some(ticket.to_string());
        self.ticket_created = Some(std::time::Instant::now());
    }

    /// Whether the session ticket is missing or old enough that it may expire
    /// soon (PVE tickets live ~2h). Token-authenticated clients never need a
    /// refresh.
    pub fn ticket_needs_refresh(&self) -> bool {
        if self.api_token.is_some() {
            return false;
        }
        if self.ticket.is_none() {
            return true;
        }
        match self.ticket_created {
            Some(created) => ticket_age_requires_refresh(created.elapsed().as_secs()),
            None => true,
        }
    }

    /// Set the CSRF prevention token (required for mutating requests).
    pub fn set_csrf_token(&mut self, token: &str) {
        self.csrf_token = Some(token.to_string());
    }

    /// Authenticate with username + password.
    /// Stores the ticket and CSRF token on success; returns the ticket string.
    pub async fn authenticate(&mut self, password: &str) -> Result<String> {
        let url = format!(
            "https://{}:{}/api2/json/access/ticket",
            self.base_url, self.port
        );

        let params = vec![("username", self.username.as_str()), ("password", password)];

        let response = self
            .client
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(|e| anyhow!("Authentication request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Authentication failed with status {status}: {text}"
            ));
        }

        let envelope: ProxmoxEnvelope<AuthResponse> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse authentication response: {e}"))?;

        let auth = envelope.data;
        self.ticket = Some(auth.ticket.clone());
        self.ticket_created = Some(std::time::Instant::now());
        if let Some(csrf) = auth.csrf_prevention_token {
            self.csrf_token = Some(csrf);
        }

        Ok(auth.ticket)
    }

    /// Authenticate with API token
    pub fn authenticate_with_token(&mut self, token: &str) {
        self.api_token = Some(token.to_string());
    }

    /// Get the full API URL for a given path
    fn get_api_url(&self, path: &str) -> String {
        format!(
            "https://{}:{}/api2/json/{}",
            self.base_url,
            self.port,
            path.trim_start_matches('/')
        )
    }

    /// Build request headers with authentication.
    /// `include_csrf` should be true for POST / PUT / DELETE requests.
    fn build_headers(
        &self,
        ticket: Option<&str>,
        include_csrf: bool,
    ) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        if let Some(token) = &self.api_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("PVEAPIAuth {token}")
                    .parse()
                    .expect("Invalid auth header"),
            );
        } else if let Some(ticket) = ticket {
            headers.insert(
                "Cookie",
                format!("PVEAuthCookie={ticket}")
                    .parse()
                    .expect("Invalid cookie header"),
            );
            if include_csrf {
                if let Some(csrf) = &self.csrf_token {
                    headers.insert(
                        "CSRFPreventionToken",
                        csrf.parse().expect("Invalid CSRF token header"),
                    );
                }
            }
        }

        headers
    }

    /// Send an idempotent request, retrying once on transient transport
    /// failures (stale pooled connection, connect error, timeout).
    async fn send_idempotent(
        &self,
        builder: reqwest::RequestBuilder,
        verb: &str,
    ) -> Result<reqwest::Response> {
        let retry = builder.try_clone();
        match builder.send().await {
            Ok(response) => Ok(response),
            Err(e) if is_transient_transport_error(&e) => match retry {
                Some(second) => second
                    .send()
                    .await
                    .map_err(|e| anyhow!("{verb} request failed: {}", describe_error_chain(&e))),
                None => Err(anyhow!(
                    "{verb} request failed: {}",
                    describe_error_chain(&e)
                )),
            },
            Err(e) => Err(anyhow!(
                "{verb} request failed: {}",
                describe_error_chain(&e)
            )),
        }
    }

    /// GET request to Proxmox API
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        ticket: Option<&str>,
    ) -> Result<T> {
        let url = self.get_api_url(path);
        let headers = self.build_headers(ticket, false);

        let response = self
            .send_idempotent(self.client.get(&url).headers(headers), "GET")
            .await?;

        self.handle_response(response).await
    }

    /// GET request with query parameters (properly URL-encoded by reqwest).
    pub async fn get_with_params<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        params: &[(&str, &str)],
        ticket: Option<&str>,
    ) -> Result<T> {
        let url = self.get_api_url(path);
        let headers = self.build_headers(ticket, false);

        let response = self
            .send_idempotent(self.client.get(&url).headers(headers).query(params), "GET")
            .await?;

        self.handle_response(response).await
    }

    /// POST request to Proxmox API with JSON body
    pub async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        ticket: Option<&str>,
    ) -> Result<T> {
        let url = self.get_api_url(path);
        let headers = self.build_headers(ticket, true);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .map_err(|e| anyhow!("POST request failed: {}", describe_error_chain(&e)))?;

        self.handle_response(response).await
    }

    /// POST request to Proxmox API with form-encoded body
    pub async fn post_form<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        params: &[(&str, &str)],
        ticket: Option<&str>,
    ) -> Result<T> {
        let url = self.get_api_url(path);
        let headers = self.build_headers(ticket, true);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .form(params)
            .send()
            .await
            .map_err(|e| anyhow!("POST form request failed: {}", describe_error_chain(&e)))?;

        self.handle_response(response).await
    }

    /// PUT request to Proxmox API
    pub async fn put<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        ticket: Option<&str>,
    ) -> Result<T> {
        let url = self.get_api_url(path);
        let headers = self.build_headers(ticket, true);

        let response = self
            .client
            .put(&url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .map_err(|e| anyhow!("PUT request failed: {}", describe_error_chain(&e)))?;

        self.handle_response(response).await
    }

    /// POST multipart/form-data to Proxmox API (used for file uploads)
    pub async fn post_multipart<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
        ticket: Option<&str>,
    ) -> Result<T> {
        let url = self.get_api_url(path);
        let headers = self.build_headers(ticket, true);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                anyhow!(
                    "POST multipart request failed: {}",
                    describe_error_chain(&e)
                )
            })?;

        self.handle_response(response).await
    }

    /// DELETE request to Proxmox API
    pub async fn delete<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        ticket: Option<&str>,
    ) -> Result<T> {
        let url = self.get_api_url(path);
        let headers = self.build_headers(ticket, true);

        let response = self
            .client
            .delete(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| anyhow!("DELETE request failed: {}", describe_error_chain(&e)))?;

        self.handle_response(response).await
    }

    /// Handle API response
    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API request failed with status {status}: {text}"));
        }

        let data: HashMap<String, serde_json::Value> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse API response: {e}"))?;

        // Proxmox API wraps data in "data" field
        data.get("data")
            .ok_or_else(|| anyhow!("Response missing 'data' field"))
            .and_then(|d| {
                serde_json::from_value(d.clone())
                    .map_err(|e| anyhow!("Failed to deserialize data: {e}"))
            })
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the username
    pub fn username(&self) -> &str {
        &self.username
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The frontend strips the protocol via parseRemoteUrl before sending to the backend,
    // so ProxmoxClient always receives a bare hostname (no scheme, no port).
    // get_api_url() is responsible for constructing the full https URL with port.

    #[test]
    fn test_proxmox_client_new() {
        let client = ProxmoxClient::new("pve.example.com", 8006, "root@pam");
        assert_eq!(client.base_url(), "pve.example.com");
        assert_eq!(client.port(), 8006);
        assert_eq!(client.username(), "root@pam");
        assert!(client.ticket.is_none());
        assert!(client.csrf_token.is_none());
    }

    #[test]
    fn test_proxmox_client_with_trailing_slash() {
        let client = ProxmoxClient::new("pve.example.com/", 8006, "root@pam");
        assert_eq!(client.base_url(), "pve.example.com");
    }

    #[test]
    fn test_describe_error_chain_includes_sources() {
        use std::fmt;

        #[derive(Debug)]
        struct Inner;
        impl fmt::Display for Inner {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "operation timed out")
            }
        }
        impl std::error::Error for Inner {}

        #[derive(Debug)]
        struct Outer(Inner);
        impl fmt::Display for Outer {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "error sending request for url (https://x)")
            }
        }
        impl std::error::Error for Outer {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.0)
            }
        }

        let msg = describe_error_chain(&Outer(Inner));
        assert_eq!(
            msg,
            "error sending request for url (https://x): operation timed out"
        );
    }

    #[test]
    fn test_ticket_needs_refresh_without_ticket() {
        let client = ProxmoxClient::new("pve.example.com", 8006, "root@pam");
        assert!(client.ticket_needs_refresh(), "no ticket → refresh needed");
    }

    #[test]
    fn test_ticket_needs_refresh_false_after_set_ticket() {
        let mut client = ProxmoxClient::new("pve.example.com", 8006, "root@pam");
        client.set_ticket("PVE:root@pam:AAAA");
        assert!(!client.ticket_needs_refresh(), "fresh ticket → no refresh");
    }

    #[test]
    fn test_ticket_needs_refresh_false_with_api_token() {
        let mut client = ProxmoxClient::new("pve.example.com", 8006, "root@pam");
        client.authenticate_with_token("user@pam!token=secret");
        assert!(
            !client.ticket_needs_refresh(),
            "API tokens do not expire like tickets"
        );
    }

    #[test]
    fn test_ticket_age_refresh_threshold() {
        assert!(!ticket_age_requires_refresh(0));
        assert!(!ticket_age_requires_refresh(89 * 60));
        assert!(ticket_age_requires_refresh(90 * 60));
        assert!(ticket_age_requires_refresh(3 * 60 * 60));
    }

    #[test]
    fn test_get_api_url() {
        let client = ProxmoxClient::new("pve.example.com", 8006, "root@pam");
        assert_eq!(
            client.get_api_url("cluster/resources"),
            "https://pve.example.com:8006/api2/json/cluster/resources"
        );
        assert_eq!(
            client.get_api_url("/cluster/resources"),
            "https://pve.example.com:8006/api2/json/cluster/resources"
        );
    }

    #[test]
    fn test_auth_response_envelope_deserialization() {
        // Validates that the `{"data": {...}}` envelope Proxmox uses is parsed
        // correctly into ProxmoxEnvelope<AuthResponse>.
        // Note: Proxmox returns lowercase fields (ticket, username, clustername)
        // except for CSRFPreventionToken which is PascalCase.
        let json = r#"{
            "data": {
                "ticket": "PVE:root@pam:12345",
                "username": "root@pam",
                "expire": 1800,
                "CSRFPreventionToken": "abc123",
                "cap": null,
                "clustername": "TFTSR"
            }
        }"#;
        let envelope: ProxmoxEnvelope<AuthResponse> =
            serde_json::from_str(json).expect("envelope should parse");
        assert_eq!(envelope.data.ticket, "PVE:root@pam:12345");
        assert_eq!(
            envelope.data.csrf_prevention_token.as_deref(),
            Some("abc123")
        );
    }

    #[test]
    fn test_auth_response_envelope_no_csrf() {
        // Some Proxmox versions or API tokens may omit CSRFPreventionToken.
        let json = r#"{
            "data": {
                "ticket": "PVE:root@pam:99999",
                "username": "root@pam",
                "clustername": "TFTSR"
            }
        }"#;
        let envelope: ProxmoxEnvelope<AuthResponse> =
            serde_json::from_str(json).expect("envelope should parse without CSRF");
        assert_eq!(envelope.data.ticket, "PVE:root@pam:99999");
        assert!(envelope.data.csrf_prevention_token.is_none());
    }

    #[test]
    fn test_build_headers_get_omits_csrf() {
        let mut client = ProxmoxClient::new("pve.example.com", 8006, "root@pam");
        client.set_ticket("my-ticket");
        client.set_csrf_token("my-csrf");

        let headers = client.build_headers(Some("my-ticket"), false);
        assert!(!headers.contains_key("CSRFPreventionToken"));
        assert!(headers.contains_key("Cookie"));
    }

    #[test]
    fn test_build_headers_post_includes_csrf() {
        let mut client = ProxmoxClient::new("pve.example.com", 8006, "root@pam");
        client.set_ticket("my-ticket");
        client.set_csrf_token("my-csrf");

        let headers = client.build_headers(Some("my-ticket"), true);
        assert!(headers.contains_key("CSRFPreventionToken"));
        let csrf_val = headers
            .get("CSRFPreventionToken")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(csrf_val, "my-csrf");
    }

    #[test]
    fn test_set_ticket_and_csrf_token() {
        let mut client = ProxmoxClient::new("pve.example.com", 8006, "root@pam");
        client.set_ticket("ticket-value");
        client.set_csrf_token("csrf-value");
        assert_eq!(client.ticket.as_deref(), Some("ticket-value"));
        assert_eq!(client.csrf_token.as_deref(), Some("csrf-value"));
    }

    #[tokio::test]
    async fn test_real_proxmox_auth() {
        let password = match std::env::var("PROXMOX_PASSWORD") {
            Ok(p) => p,
            Err(_) => {
                println!("Skipping test: PROXMOX_PASSWORD env var not set");
                return;
            }
        };

        let mut client = ProxmoxClient::new("172.0.0.18", 8006, "root@pam");
        let result = client.authenticate(&password).await;
        match result {
            Ok(ticket) => {
                println!("✓ Authentication successful");
                println!("  Ticket: {}", &ticket[..50]);
                assert!(client.ticket.is_some());
                assert!(client.csrf_token.is_some());
            }
            Err(e) => {
                panic!("Authentication failed: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_real_proxmox_cluster_resources() {
        let password = match std::env::var("PROXMOX_PASSWORD") {
            Ok(p) => p,
            Err(_) => {
                println!("Skipping test: PROXMOX_PASSWORD env var not set");
                return;
            }
        };

        let mut client = ProxmoxClient::new("172.0.0.18", 8006, "root@pam");
        client
            .authenticate(&password)
            .await
            .expect("Authentication failed");

        #[derive(serde::Deserialize, Debug)]
        struct Resource {
            #[serde(default)]
            vmid: Option<u32>,
            name: Option<String>,
            r#type: Option<String>,
            node: Option<String>,
            status: Option<String>,
        }

        let result: Result<Vec<Resource>, _> = client
            .get("cluster/resources", client.ticket.as_deref())
            .await;
        match result {
            Ok(resources) => {
                println!("✓ Cluster resources fetched successfully");
                println!("  Found {} resources", resources.len());
            }
            Err(e) => {
                panic!("Failed to get cluster resources: {e}");
            }
        }
    }

    fn get_test_client() -> ProxmoxClient {
        let host = std::env::var("PROXMOX_HOST").unwrap_or_else(|_| "proxmox-server".to_string());
        ProxmoxClient::new(&host, 8006, "root@pam")
    }

    #[tokio::test]
    async fn test_real_proxmox_nodes() {
        let password = match std::env::var("PROXMOX_PASSWORD") {
            Ok(p) => p,
            Err(_) => {
                println!("Skipping test: PROXMOX_PASSWORD env var not set");
                return;
            }
        };

        let host = std::env::var("PROXMOX_HOST").unwrap_or_else(|_| "proxmox-server".to_string());
        let mut client = ProxmoxClient::new(&host, 8006, "root@pam");
        client
            .authenticate(&password)
            .await
            .expect("Authentication failed");

        #[derive(serde::Deserialize, Debug)]
        struct Node {
            node: String,
            status: String,
            #[serde(default)]
            level: String,
            #[serde(default)]
            cpu: f64,
            #[serde(default)]
            uptime: u64,
        }

        let result: Result<Vec<Node>, _> = client.get("nodes", client.ticket.as_deref()).await;
        match result {
            Ok(nodes) => {
                println!("✓ Nodes fetched successfully");
                for node in &nodes {
                    println!("  Node: {} - Status: {}", node.node, node.status);
                }
            }
            Err(e) => {
                panic!("Failed to get nodes: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_real_proxmox_vms() {
        let password = match std::env::var("PROXMOX_PASSWORD") {
            Ok(p) => p,
            Err(_) => {
                println!("Skipping test: PROXMOX_PASSWORD env var not set");
                return;
            }
        };

        let host = std::env::var("PROXMOX_HOST").unwrap_or_else(|_| "proxmox-server".to_string());
        let mut client = ProxmoxClient::new(&host, 8006, "root@pam");
        client
            .authenticate(&password)
            .await
            .expect("Authentication failed");

        #[derive(serde::Deserialize, Debug)]
        struct Resource {
            #[serde(default)]
            vmid: Option<u32>,
            name: Option<String>,
            r#type: Option<String>,
            status: Option<String>,
        }

        let result: Result<Vec<Resource>, _> = client
            .get("cluster/resources", client.ticket.as_deref())
            .await;
        match result {
            Ok(resources) => {
                let vms: Vec<_> = resources
                    .into_iter()
                    .filter(|r| r.r#type.as_deref() == Some("qemu"))
                    .collect();
                println!("✓ VMs fetched successfully");
                println!("  Found {} VMs", vms.len());
            }
            Err(e) => {
                panic!("Failed to get VMs: {e}");
            }
        }
    }
}
