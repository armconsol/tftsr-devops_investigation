/// macOS-specific native cookie extraction using WKWebView's HTTPCookieStore
/// This can access HttpOnly cookies that JavaScript cannot

#[cfg(target_os = "macos")]
use super::webview_auth::Cookie;

#[cfg(target_os = "macos")]
pub async fn extract_cookies_native(
    webview_label: &str,
    domain: &str,
) -> Result<Vec<Cookie>, String> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSArray, NSString};
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    tracing::info!("Attempting native cookie extraction for {} on domain {}", webview_label, domain);

    unsafe {
        // Get the WKWebsiteDataStore (where cookies are stored)
        let wk_websitedata_store_class = Class::get("WKWebsiteDataStore").ok_or("WKWebsiteDataStore class not found")?;
        let data_store: id = msg_send![wk_websitedata_store_class, defaultDataStore];

        if data_store == nil {
            return Err("Failed to get WKWebsiteDataStore".to_string());
        }

        // Get the HTTPCookieStore
        let cookie_store: id = msg_send![data_store, httpCookieStore];

        if cookie_store == nil {
            return Err("Failed to get HTTPCookieStore".to_string());
        }

        // Unfortunately, WKHTTPCookieStore's getAllCookies method requires a completion handler
        // which is complex to bridge from Rust. For now, we'll document this limitation
        // and suggest using the Tauri cookie plugin when it's available.

        tracing::warn!("Native cookie extraction requires async completion handler - not yet fully implemented");
        Err("Native cookie extraction requires Tauri cookie plugin (coming in future Tauri version)".to_string())
    }
}

#[cfg(not(target_os = "macos"))]
pub async fn extract_cookies_native(
    _webview_label: &str,
    _domain: &str,
) -> Result<Vec<super::webview_auth::Cookie>, String> {
    Err("Native cookie extraction only supported on macOS".to_string())
}
