use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Listener, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedCredentials {
    pub cookies: Vec<Cookie>,
    pub service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub expires: Option<i64>,
}

/// Open an embedded browser window for the user to log in and extract cookies.
/// This approach works when user is off-VPN (can access web UI) but APIs require VPN.
pub async fn authenticate_with_webview(
    app_handle: AppHandle,
    service: &str,
    base_url: &str,
) -> Result<ExtractedCredentials, String> {
    let trimmed_base_url = base_url.trim_end_matches('/');
    let login_url = match service {
        "confluence" => format!("{trimmed_base_url}/login.action"),
        "azuredevops" => {
            // Azure DevOps login - user will be redirected through Microsoft SSO
            format!("{trimmed_base_url}/_signin")
        }
        "servicenow" => format!("{trimmed_base_url}/login.do"),
        _ => return Err(format!("Unknown service: {service}")),
    };

    tracing::info!(
        "Opening persistent browser for {} at {}",
        service,
        login_url
    );

    // Create persistent browser window (stays open for browsing and fresh cookie extraction)
    let webview_label = format!("{service}-auth");
    let webview = WebviewWindowBuilder::new(
        &app_handle,
        &webview_label,
        WebviewUrl::External(login_url.parse().map_err(|e| format!("Invalid URL: {e}"))?),
    )
    .title(format!("{service} Browser (TFTSR)"))
    .inner_size(1000.0, 800.0)
    .min_inner_size(800.0, 600.0)
    .resizable(true)
    .center()
    .focused(true)
    .visible(true)
    .build()
    .map_err(|e| format!("Failed to create webview: {e}"))?;

    // Focus the window
    webview
        .set_focus()
        .map_err(|e| tracing::warn!("Failed to focus webview: {e}"))
        .ok();

    // Wait for user to complete login
    // User will click "Complete Login" button in the UI after successful authentication
    // This function just opens the window - extraction happens in extract_cookies_via_ipc

    Ok(ExtractedCredentials {
        cookies: vec![],
        service: service.to_string(),
    })
}

/// Extract cookies from a webview using Tauri's IPC mechanism.
/// This is the most reliable cross-platform approach.
pub async fn extract_cookies_via_ipc<R: tauri::Runtime>(
    webview_window: &WebviewWindow<R>,
    app_handle: &AppHandle<R>,
) -> Result<Vec<Cookie>, String> {
    // Inject JavaScript that will send cookies via IPC
    // Note: We use window.__TAURI__ which is the Tauri 2.x API exposed to webviews
    let cookie_extraction_script = r#"
        (async function() {
            try {
                // Wait for Tauri API to be available
                if (typeof window.__TAURI__ === 'undefined') {
                    console.error('Tauri API not available');
                    return;
                }

                const cookieString = document.cookie;
                if (!cookieString || cookieString.trim() === '') {
                    await window.__TAURI__.event.emit('tftsr-cookies-extracted', { cookies: [] });
                    return;
                }

                const cookies = cookieString.split(';').map(c => c.trim()).filter(c => c.length > 0);
                const parsed = cookies.map(cookie => {
                    const equalIndex = cookie.indexOf('=');
                    if (equalIndex === -1) return null;

                    const name = cookie.substring(0, equalIndex).trim();
                    const value = cookie.substring(equalIndex + 1).trim();

                    return {
                        name: name,
                        value: value,
                        domain: window.location.hostname,
                        path: '/',
                        secure: window.location.protocol === 'https:',
                        http_only: false,
                        expires: null
                    };
                }).filter(c => c !== null);

                // Use Tauri's event API to send cookies back to Rust
                await window.__TAURI__.event.emit('tftsr-cookies-extracted', { cookies: parsed });
                console.log('Cookies extracted and emitted:', parsed.length);
            } catch (e) {
                console.error('Cookie extraction failed:', e);
                try {
                    await window.__TAURI__.event.emit('tftsr-cookies-extracted', { cookies: [], error: e.message });
                } catch (emitError) {
                    console.error('Failed to emit error:', emitError);
                }
            }
        })();
    "#;

    // Set up event listener first
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<Vec<Cookie>, String>>(1);

    // Listen for the custom event from the webview
    let listen_id = app_handle.listen("tftsr-cookies-extracted", move |event| {
        tracing::debug!("Received cookies-extracted event");

        let payload_str = event.payload();

        // Parse the payload JSON
        match serde_json::from_str::<serde_json::Value>(payload_str) {
            Ok(payload) => {
                if let Some(error_msg) = payload.get("error").and_then(|e| e.as_str()) {
                    let _ = tx.try_send(Err(format!("JavaScript error: {error_msg}")));
                    return;
                }

                if let Some(cookies_value) = payload.get("cookies") {
                    match serde_json::from_value::<Vec<Cookie>>(cookies_value.clone()) {
                        Ok(cookies) => {
                            tracing::info!("Parsed {} cookies from webview", cookies.len());
                            let _ = tx.try_send(Ok(cookies));
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse cookies: {e}");
                            let _ = tx.try_send(Err(format!("Failed to parse cookies: {e}")));
                        }
                    }
                } else {
                    let _ = tx.try_send(Err("No cookies field in payload".to_string()));
                }
            }
            Err(e) => {
                tracing::error!("Failed to parse event payload: {e}");
                let _ = tx.try_send(Err(format!("Failed to parse event payload: {e}")));
            }
        }
    });

    // Inject the script into the webview
    webview_window
        .eval(cookie_extraction_script)
        .map_err(|e| format!("Failed to inject cookie extraction script: {e}"))?;

    tracing::info!("Cookie extraction script injected, waiting for response...");

    // Wait for cookies with timeout
    let result = tokio::time::timeout(tokio::time::Duration::from_secs(10), rx.recv())
        .await
        .map_err(|_| {
            "Timeout waiting for cookies. Make sure you are logged in and on the correct page."
                .to_string()
        })?
        .ok_or_else(|| "Failed to receive cookies from webview".to_string())?;

    // Clean up event listener
    app_handle.unlisten(listen_id);

    result
}

/// Build cookie header string for HTTP requests
pub fn cookies_to_header(cookies: &[Cookie]) -> String {
    cookies
        .iter()
        .map(|c| format!("{name}={value}", name = c.name.as_str(), value = c.value.as_str()))
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookies_to_header() {
        let cookies = vec![
            Cookie {
                name: "JSESSIONID".to_string(),
                value: "abc123".to_string(),
                domain: "example.com".to_string(),
                path: "/".to_string(),
                secure: true,
                http_only: true,
                expires: None,
            },
            Cookie {
                name: "auth_token".to_string(),
                value: "xyz789".to_string(),
                domain: "example.com".to_string(),
                path: "/".to_string(),
                secure: true,
                http_only: false,
                expires: None,
            },
        ];

        let header = cookies_to_header(&cookies);
        assert_eq!(header, "JSESSIONID=abc123; auth_token=xyz789");
    }

    #[test]
    fn test_empty_cookies_to_header() {
        let cookies = vec![];
        let header = cookies_to_header(&cookies);
        assert_eq!(header, "");
    }

    #[test]
    fn test_cookie_json_serialization() {
        let cookies = vec![Cookie {
            name: "test".to_string(),
            value: "value123".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            secure: true,
            http_only: false,
            expires: None,
        }];

        let json = serde_json::to_string(&cookies).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"value\":\"value123\""));

        let deserialized: Vec<Cookie> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.len(), 1);
        assert_eq!(deserialized[0].name, "test");
    }
}
