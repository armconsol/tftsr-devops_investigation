use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

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
    let login_url = match service {
        "confluence" => format!("{}/login.action", base_url.trim_end_matches('/')),
        "azuredevops" => {
            // Azure DevOps login - user will be redirected through Microsoft SSO
            format!("{}/_signin", base_url.trim_end_matches('/'))
        }
        "servicenow" => format!("{}/login.do", base_url.trim_end_matches('/')),
        _ => return Err(format!("Unknown service: {}", service)),
    };

    tracing::info!("Opening embedded browser for {} at {}", service, login_url);

    // Create embedded webview window
    let webview_label = format!("{}-auth-window", service);
    let _webview = WebviewWindowBuilder::new(
        &app_handle,
        &webview_label,
        WebviewUrl::External(login_url.parse().map_err(|e| format!("Invalid URL: {}", e))?),
    )
    .title(format!("Login to {}", service))
    .inner_size(800.0, 700.0)
    .resizable(true)
    .center()
    .build()
    .map_err(|e| format!("Failed to create webview: {}", e))?;

    // Wait for user to complete login
    // We'll detect this by checking if they reached a success page or dashboard
    // For now, return a placeholder - actual implementation needs JS injection

    Ok(ExtractedCredentials {
        cookies: vec![],
        service: service.to_string(),
    })
}

/// Extract cookies from a webview after successful login.
/// This uses Tauri's webview cookie API to get session cookies.
pub async fn extract_cookies_from_webview(
    webview_label: &str,
    app_handle: &AppHandle,
    _service: &str,
) -> Result<Vec<Cookie>, String> {
    let webview = app_handle
        .get_webview_window(webview_label)
        .ok_or_else(|| "Webview window not found".to_string())?;

    // Get all cookies from the webview
    // Note: Tauri 2.x provides cookie manager via webview
    // We need to use eval_script to extract cookies via JavaScript

    let cookie_script = r#"
        (function() {
            const cookies = document.cookie.split(';').map(c => c.trim());
            const parsed = cookies.map(cookie => {
                const [nameValue, ...attrs] = cookie.split(';');
                const [name, value] = nameValue.split('=');
                return {
                    name: name.trim(),
                    value: value?.trim() || '',
                    domain: window.location.hostname,
                    path: '/',
                    secure: window.location.protocol === 'https:',
                    http_only: false,
                    expires: null
                };
            });
            return JSON.stringify(parsed);
        })();
    "#;

    let _result = webview
        .eval(cookie_script)
        .map_err(|e| format!("Failed to extract cookies: {}", e))?;

    // Parse the JSON result
    // TODO: Actually parse the cookies from the eval result
    // For now, return empty - needs proper implementation

    Ok(vec![])
}

/// Build cookie header string for HTTP requests
pub fn cookies_to_header(cookies: &[Cookie]) -> String {
    cookies
        .iter()
        .map(|c| format!("{}={}", c.name, c.value))
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
}
