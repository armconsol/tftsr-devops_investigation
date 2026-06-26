use serde::{Deserialize, Serialize};
use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

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
    project_name: Option<&str>,
) -> Result<ExtractedCredentials, String> {
    let trimmed_base_url = base_url.trim_end_matches('/');

    tracing::info!(
        "authenticate_with_webview called: service={}, base_url={}, project_name={:?}",
        service,
        base_url,
        project_name
    );

    let login_url = match service {
        "confluence" => format!("{trimmed_base_url}/login.action"),
        "azuredevops" => {
            // Azure DevOps - go directly to project if provided, otherwise org home
            if let Some(project) = project_name {
                let url = format!("{trimmed_base_url}/{project}");
                tracing::info!("Azure DevOps URL with project: {}", url);
                url
            } else {
                tracing::info!("Azure DevOps URL without project: {}", trimmed_base_url);
                trimmed_base_url.to_string()
            }
        }
        "servicenow" => format!("{trimmed_base_url}/login.do"),
        _ => return Err(format!("Unknown service: {service}")),
    };

    tracing::info!("Final login_url for {} = {}", service, login_url);

    // Create persistent browser window (stays open for browsing and fresh cookie extraction)
    let webview_label = format!("{service}-auth");

    tracing::info!("Creating webview window with label: {}", webview_label);

    let parsed_url = login_url.parse().map_err(|e| {
        let err_msg = format!("Failed to parse URL '{login_url}': {e}");
        tracing::error!("{err_msg}");
        err_msg
    })?;

    tracing::info!("Parsed URL successfully: {:?}", parsed_url);

    let webview = WebviewWindowBuilder::new(
        &app_handle,
        &webview_label,
        WebviewUrl::External(parsed_url),
    )
    .title(format!(
        "{service} Browser (Troubleshooting and RCA Assistant)"
    ))
    .inner_size(1000.0, 800.0)
    .min_inner_size(800.0, 600.0)
    .resizable(true)
    .center()
    .focused(true)
    .visible(true)  // Show immediately - let user see loading
    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
    .zoom_hotkeys_enabled(true)
    .devtools(true)
    .initialization_script("console.log('Webview initialized');")
    .build()
    .map_err(|e| format!("Failed to create webview: {e}"))?;

    tracing::info!("Webview window created successfully, setting focus");

    // Ensure window is focused
    webview
        .set_focus()
        .map_err(|e| tracing::warn!("Failed to set focus: {}", e))
        .ok();

    // Wait for user to complete login
    // User will click "Complete Login" button in the UI after successful authentication
    // This function just opens the window - extraction happens in extract_cookies_via_ipc

    Ok(ExtractedCredentials {
        cookies: vec![],
        service: service.to_string(),
    })
}

/// Extract cookies from a webview using localStorage as intermediary.
/// This works for external URLs where window.__TAURI__ is not available.
pub async fn extract_cookies_via_ipc<R: tauri::Runtime>(
    webview_window: &WebviewWindow<R>,
    _app_handle: &AppHandle<R>,
) -> Result<Vec<Cookie>, String> {
    // Step 1: Inject JavaScript to extract cookies and store in a global variable
    // We can't use __TAURI__ for external URLs, so we use a polling approach
    let cookie_extraction_script = r#"
        (function() {
            try {
                const cookieString = document.cookie;
                const cookies = [];

                if (cookieString && cookieString.trim() !== '') {
                    const cookieList = cookieString.split(';').map(c => c.trim()).filter(c => c.length > 0);
                    for (const cookie of cookieList) {
                        const equalIndex = cookie.indexOf('=');
                        if (equalIndex === -1) continue;

                        const name = cookie.substring(0, equalIndex).trim();
                        const value = cookie.substring(equalIndex + 1).trim();

                        cookies.push({
                            name: name,
                            value: value,
                            domain: window.location.hostname,
                            path: '/',
                            secure: window.location.protocol === 'https:',
                            http_only: false,
                            expires: null
                        });
                    }
                }

                // Store in a global variable that Rust can read
                window.__TRCAA_COOKIES__ = cookies;
                console.log('[TRCAA] Extracted', cookies.length, 'cookies');
                return cookies.length;
            } catch (e) {
                console.error('[TRCAA] Cookie extraction failed:', e);
                window.__TRCAA_COOKIES__ = [];
                window.__TRCAA_ERROR__ = e.message;
                return -1;
            }
        })();
    "#;

    // Inject the extraction script
    webview_window
        .eval(cookie_extraction_script)
        .map_err(|e| format!("Failed to inject cookie extraction script: {e}"))?;

    tracing::info!("Cookie extraction script injected, waiting for cookies...");

    // Give JavaScript a moment to execute
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Step 2: Poll for the extracted cookies using document.title as communication channel
    let mut attempts = 0;
    let max_attempts = 20; // 10 seconds total (500ms * 20)

    loop {
        attempts += 1;

        // Store result in localStorage, then copy to document.title for Rust to read
        let check_and_signal_script = r#"
            try {
                if (typeof window.__TRCAA_ERROR__ !== 'undefined') {
                    window.localStorage.setItem('tftsr_result', JSON.stringify({ error: window.__TRCAA_ERROR__ }));
                } else if (typeof window.__TRCAA_COOKIES__ !== 'undefined' && window.__TRCAA_COOKIES__.length > 0) {
                    window.localStorage.setItem('tftsr_result', JSON.stringify({ cookies: window.__TRCAA_COOKIES__ }));
                } else if (typeof window.__TRCAA_COOKIES__ !== 'undefined') {
                    window.localStorage.setItem('tftsr_result', JSON.stringify({ cookies: [] }));
                }
            } catch (e) {
                window.localStorage.setItem('tftsr_result', JSON.stringify({ error: e.message }));
            }
        "#;

        webview_window.eval(check_and_signal_script).ok();

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // We can't get return values from eval(), so let's use a different approach:
        // Execute script that sets document.title temporarily
        let read_via_title = r#"
            (function() {
                const result = window.localStorage.getItem('tftsr_result');
                if (result) {
                    window.localStorage.removeItem('tftsr_result');
                    // Store in title temporarily for Rust to read
                    window.__TRCAA_ORIGINAL_TITLE__ = document.title;
                    document.title = 'TRCAA_RESULT:' + result;
                }
            })();
        "#;

        webview_window.eval(read_via_title).ok();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Read the title
        if let Ok(title) = webview_window.title() {
            if let Some(json_str) = title.strip_prefix("TRCAA_RESULT:") {
                // Restore original title
                let restore_title = r#"
                    if (typeof window.__TRCAA_ORIGINAL_TITLE__ !== 'undefined') {
                        document.title = window.__TRCAA_ORIGINAL_TITLE__;
                    }
                "#;
                webview_window.eval(restore_title).ok();

                // Parse the JSON
                match serde_json::from_str::<serde_json::Value>(json_str) {
                    Ok(result) => {
                        if let Some(error) = result.get("error").and_then(|e| e.as_str()) {
                            return Err(format!("Cookie extraction error: {error}"));
                        }

                        if let Some(cookies_value) = result.get("cookies") {
                            match serde_json::from_value::<Vec<Cookie>>(cookies_value.clone()) {
                                Ok(cookies) => {
                                    tracing::info!(
                                        "Successfully extracted {} cookies",
                                        cookies.len()
                                    );
                                    return Ok(cookies);
                                }
                                Err(e) => {
                                    return Err(format!("Failed to parse cookies: {e}"));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse result JSON: {e}");
                    }
                }
            }
        }

        if attempts >= max_attempts {
            return Err(
                "Timeout extracting cookies. This may be because:\n\
                1. Confluence uses HttpOnly cookies that JavaScript cannot access\n\
                2. You're not logged in yet\n\
                3. The page hasn't finished loading\n\n\
                Recommendation: Use 'Manual Token' authentication with a Confluence Personal Access Token instead."
                    .to_string(),
            );
        }
    }
}

/// Build cookie header string for HTTP requests
pub fn cookies_to_header(cookies: &[Cookie]) -> String {
    cookies
        .iter()
        .map(|c| {
            format!(
                "{name}={value}",
                name = c.name.as_str(),
                value = c.value.as_str()
            )
        })
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
