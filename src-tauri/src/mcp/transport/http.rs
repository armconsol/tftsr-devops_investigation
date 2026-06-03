use http::{HeaderName, HeaderValue};
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::StreamableHttpClientTransport;
use std::collections::HashMap;
use std::sync::Arc;

/// Parse and validate custom headers, injecting default Accept header if needed.
/// Returns a HashMap of validated HTTP headers ready for use in transport config.
///
/// Invalid headers (bad names or values) are logged and skipped.
/// If no Accept header is provided, auto-injects "application/json, text/event-stream".
fn build_header_map(custom_headers: HashMap<String, String>) -> HashMap<HeaderName, HeaderValue> {
    let mut http_headers = HashMap::new();

    // Add custom headers from caller
    for (key, value) in custom_headers.iter() {
        let name_result = HeaderName::from_bytes(key.as_bytes());
        let value_result = HeaderValue::from_str(value);

        match (name_result, value_result) {
            (Ok(name), Ok(val)) => {
                http_headers.insert(name, val);
                tracing::debug!("Added custom header: {key}");
            }
            (Err(name_err), _) => {
                tracing::warn!(
                    "Invalid header name '{key}': {name_err}, skipping (value: <redacted>)"
                );
            }
            (Ok(_), Err(value_err)) => {
                tracing::warn!(
                    "Invalid header value for '{key}': {value_err}, skipping (value: <redacted>)"
                );
            }
        }
    }

    // Always add required Accept header for MCP servers that need both MIME types
    // (unless already provided by the caller)
    let accept_key = HeaderName::from_static("accept");
    http_headers.entry(accept_key).or_insert_with(|| {
        tracing::debug!("Added default Accept header for MCP compatibility");
        HeaderValue::from_static("application/json, text/event-stream")
    });

    http_headers
}

/// Build an HTTP (Streamable HTTP) transport from a URL with optional custom headers.
/// Optionally attaches an Authorization bearer token.
///
/// Custom headers are now fully supported via rmcp's `.custom_headers()` method.
pub fn build_http_transport(
    url: &str,
    auth_header: Option<&str>,
    custom_headers: HashMap<String, String>,
) -> impl rmcp::transport::Transport<rmcp::RoleClient> {
    let http_headers = build_header_map(custom_headers);

    // Build config with auth header and custom headers
    let mut config = StreamableHttpClientTransportConfig::with_uri(Arc::from(url));

    if let Some(token) = auth_header {
        config = config.auth_header(token.to_string());
    }

    config = config.custom_headers(http_headers);

    StreamableHttpClientTransport::from_config(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_accept_header_injected() {
        let headers = HashMap::new();
        let result = build_header_map(headers);

        let accept = HeaderName::from_static("accept");
        assert!(result.contains_key(&accept), "Should inject Accept header");
        assert_eq!(
            result.get(&accept).unwrap(),
            "application/json, text/event-stream"
        );
    }

    #[test]
    fn test_caller_accept_header_respected() {
        let mut headers = HashMap::new();
        headers.insert("Accept".to_string(), "application/json".to_string());

        let result = build_header_map(headers);

        let accept = HeaderName::from_static("accept");
        assert_eq!(
            result.get(&accept).unwrap(),
            "application/json",
            "Should use caller's Accept header, not default"
        );
    }

    #[test]
    fn test_case_insensitive_accept_preserved() {
        let mut headers = HashMap::new();
        headers.insert("accept".to_string(), "text/plain".to_string());

        let result = build_header_map(headers);

        let accept = HeaderName::from_static("accept");
        assert_eq!(
            result.get(&accept).unwrap(),
            "text/plain",
            "Lowercase 'accept' should be preserved"
        );
    }

    #[test]
    fn test_adds_valid_custom_header() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());

        let result = build_header_map(headers);

        let custom = HeaderName::from_static("x-custom-header");
        assert!(result.contains_key(&custom));
        assert_eq!(result.get(&custom).unwrap(), "custom-value");
    }

    #[test]
    fn test_adds_multiple_custom_headers() {
        let mut headers = HashMap::new();
        headers.insert("X-Header-One".to_string(), "value1".to_string());
        headers.insert("X-Header-Two".to_string(), "value2".to_string());
        headers.insert("X-Header-Three".to_string(), "value3".to_string());

        let result = build_header_map(headers);

        // Plus Accept = 4 total
        assert_eq!(result.len(), 4);
        assert!(result.contains_key(&HeaderName::from_static("x-header-one")));
        assert!(result.contains_key(&HeaderName::from_static("x-header-two")));
        assert!(result.contains_key(&HeaderName::from_static("x-header-three")));
    }

    #[test]
    fn test_skips_invalid_header_name() {
        let mut headers = HashMap::new();
        headers.insert("Invalid Header Name".to_string(), "value".to_string()); // spaces invalid
        headers.insert("Valid-Header".to_string(), "valid".to_string());

        let result = build_header_map(headers);

        // Should have valid header + Accept, but not invalid
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&HeaderName::from_static("valid-header")));
        assert!(result.contains_key(&HeaderName::from_static("accept")));
    }

    #[test]
    fn test_skips_invalid_header_value() {
        let mut headers = HashMap::new();
        headers.insert("X-Valid-Name".to_string(), "invalid\nvalue".to_string()); // newline invalid
        headers.insert("X-Another".to_string(), "valid".to_string());

        let result = build_header_map(headers);

        // Should have valid header + Accept, but not invalid
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&HeaderName::from_static("x-another")));
        assert_eq!(
            result.get(&HeaderName::from_static("x-another")).unwrap(),
            "valid"
        );
    }

    #[test]
    fn test_skips_header_with_null_byte_in_name() {
        let mut headers = HashMap::new();
        headers.insert("X-Bad\0Header".to_string(), "value".to_string());
        headers.insert("X-Good-Header".to_string(), "value".to_string());

        let result = build_header_map(headers);

        // Should have good header + Accept, but not bad
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&HeaderName::from_static("x-good-header")));
    }

    #[test]
    fn test_skips_header_with_null_byte_in_value() {
        let mut headers = HashMap::new();
        headers.insert("X-Header".to_string(), "bad\0value".to_string());
        headers.insert("X-Good".to_string(), "goodvalue".to_string());

        let result = build_header_map(headers);

        // Should have good header + Accept, but not bad
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&HeaderName::from_static("x-good")));
    }

    #[test]
    fn test_empty_custom_headers_only_has_accept() {
        let headers = HashMap::new();
        let result = build_header_map(headers);

        // Should only have the default Accept header
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&HeaderName::from_static("accept")));
    }

    #[test]
    fn test_empty_string_value_allowed() {
        let mut headers = HashMap::new();
        headers.insert("X-Empty-Value".to_string(), "".to_string());

        let result = build_header_map(headers);

        // Empty string is valid
        assert!(result.contains_key(&HeaderName::from_static("x-empty-value")));
        assert_eq!(
            result
                .get(&HeaderName::from_static("x-empty-value"))
                .unwrap(),
            ""
        );
    }

    #[test]
    fn test_unicode_in_header_value_accepted() {
        let mut headers = HashMap::new();
        headers.insert("X-Unicode".to_string(), "café".to_string()); // UTF-8 is valid in HTTP header values
        headers.insert("X-Valid".to_string(), "ascii".to_string());

        let result = build_header_map(headers);

        // HeaderValue accepts valid UTF-8
        assert_eq!(result.len(), 3); // unicode + valid + accept
        assert!(result.contains_key(&HeaderName::from_static("x-valid")));
        assert!(result.contains_key(&HeaderName::from_static("x-unicode")));
    }

    #[test]
    fn test_confluence_compatible_headers() {
        let mut headers = HashMap::new();
        headers.insert(
            "Accept".to_string(),
            "application/json, text/event-stream".to_string(),
        );

        let result = build_header_map(headers);

        // Should use the Confluence-required Accept header
        assert_eq!(
            result.get(&HeaderName::from_static("accept")).unwrap(),
            "application/json, text/event-stream"
        );
    }

    // Transport building tests (verify no panics with Tokio runtime)
    #[test]
    fn test_builds_transport_with_http() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let _transport = build_http_transport("http://localhost:8080", None, HashMap::new());
    }

    #[test]
    fn test_builds_transport_with_https() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let _transport = build_http_transport("https://example.com/mcp", None, HashMap::new());
    }

    #[test]
    fn test_builds_transport_with_auth() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let _transport = build_http_transport(
            "http://localhost:8080",
            Some("Bearer token123"),
            HashMap::new(),
        );
    }
}
