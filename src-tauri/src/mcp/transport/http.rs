use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::StreamableHttpClientTransport;
use std::sync::Arc;

/// Build an HTTP (Streamable HTTP) transport from a URL.
/// Optionally attaches an Authorization bearer token.
pub fn build_http_transport(
    url: &str,
    auth_header: Option<&str>,
) -> impl rmcp::transport::Transport<rmcp::RoleClient> {
    let config = match auth_header {
        Some(token) => StreamableHttpClientTransportConfig::with_uri(Arc::from(url))
            .auth_header(token.to_string()),
        None => StreamableHttpClientTransportConfig::with_uri(Arc::from(url)),
    };
    StreamableHttpClientTransport::from_config(config)
}
