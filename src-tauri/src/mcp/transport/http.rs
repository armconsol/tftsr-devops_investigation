use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::StreamableHttpClientTransport;
use std::collections::HashMap;
use std::sync::Arc;

/// Build an HTTP (Streamable HTTP) transport from a URL with optional custom headers.
/// Optionally attaches an Authorization bearer token.
///
/// NOTE: Custom headers are parsed but not yet applied due to rmcp v1.7.0 API limitations.
/// The rmcp library's StreamableHttpClientTransportConfig does not expose a .header() method.
/// Custom headers support is deferred until rmcp adds this capability or we find an alternative.
pub fn build_http_transport(
    url: &str,
    auth_header: Option<&str>,
    custom_headers: HashMap<String, String>,
) -> impl rmcp::transport::Transport<rmcp::RoleClient> {
    // Log warning if custom headers are provided (not yet supported)
    if !custom_headers.is_empty() {
        tracing::warn!(
            "Custom HTTP headers provided but not supported by rmcp v1.7.0: {:?}",
            custom_headers.keys().collect::<Vec<_>>()
        );
    }

    let config = match auth_header {
        Some(token) => StreamableHttpClientTransportConfig::with_uri(Arc::from(url))
            .auth_header(token.to_string()),
        None => StreamableHttpClientTransportConfig::with_uri(Arc::from(url)),
    };

    StreamableHttpClientTransport::from_config(config)
}
