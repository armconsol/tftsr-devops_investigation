use serde::{Deserialize, Serialize};
use url::Url;

use super::query_expansion::expand_query;

const MAX_EXPANDED_QUERIES: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub excerpt: String,
    pub content: Option<String>,
    pub source: String,
}

fn canonicalize_url(url: &str) -> String {
    Url::parse(url)
        .ok()
        .map(|u| {
            let mut u = u.clone();
            u.set_fragment(None);
            u.set_query(None);
            u.to_string()
        })
        .unwrap_or_else(|| url.to_string())
}

fn escape_cql(s: &str) -> String {
    s.replace('"', "\\\"")
        .replace(')', "\\)")
        .replace('(', "\\(")
        .replace('~', "\\~")
        .replace('&', "\\&")
        .replace('|', "\\|")
        .replace('+', "\\+")
        .replace('-', "\\-")
}

/// Search Confluence for content matching the query
///
/// This function expands the user query with related terms, synonyms, and variations
/// to improve search coverage across Confluence spaces.
pub async fn search_confluence(
    base_url: &str,
    query: &str,
    cookies: &[crate::integrations::webview_auth::Cookie],
) -> Result<Vec<SearchResult>, String> {
    let cookie_header = crate::integrations::webview_auth::cookies_to_header(cookies);
    let client = reqwest::Client::new();

    let expanded_queries = expand_query(query);

    let mut all_results = Vec::new();

    for expanded_query in expanded_queries.iter().take(MAX_EXPANDED_QUERIES) {
        let safe_query = escape_cql(expanded_query);
        let search_url = format!(
            "{}/rest/api/search?cql=text~\"{}\"&limit=5",
            base_url.trim_end_matches('/'),
            urlencoding::encode(&safe_query)
        );

        tracing::info!("Searching Confluence with expanded query: {expanded_query}");

        let resp = client
            .get(&search_url)
            .header("Cookie", &cookie_header)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Confluence search request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!("Confluence search failed with status {status}: {text}");
            continue;
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Confluence search response: {e}"))?;

        if let Some(results_array) = json["results"].as_array() {
            for item in results_array.iter().take(MAX_EXPANDED_QUERIES) {
                let title = item["title"].as_str().unwrap_or("Untitled").to_string();

                let id = item["content"]["id"].as_str();
                let space_key = item["content"]["space"]["key"].as_str();

                let url = if let (Some(id_str), Some(space)) = (id, space_key) {
                    format!(
                        "{}/display/{space}/{id_str}",
                        base_url.trim_end_matches('/')
                    )
                } else {
                    base_url.to_string()
                };

                let excerpt = strip_html_tags(item["excerpt"].as_str().unwrap_or(""))
                    .chars()
                    .take(300)
                    .collect::<String>();

                let content = if let Some(content_id) = id {
                    fetch_page_content(base_url, content_id, &cookie_header)
                        .await
                        .ok()
                } else {
                    None
                };

                all_results.push(SearchResult {
                    title,
                    url,
                    excerpt,
                    content,
                    source: "Confluence".to_string(),
                });
            }
        }
    }

    all_results.sort_by_key(|a| canonicalize_url(&a.url));
    all_results.dedup_by(|a, b| canonicalize_url(&a.url) == canonicalize_url(&b.url));

    Ok(all_results)
}

/// Fetch full content of a Confluence page
async fn fetch_page_content(
    base_url: &str,
    page_id: &str,
    cookie_header: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let content_url = format!(
        "{}/rest/api/content/{page_id}?expand=body.storage",
        base_url.trim_end_matches('/')
    );

    let resp = client
        .get(&content_url)
        .header("Cookie", cookie_header)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch page content: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(format!("Failed to fetch page: {status}"));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse page content: {e}"))?;

    // Extract plain text from HTML storage format
    let html = json["body"]["storage"]["value"]
        .as_str()
        .unwrap_or("")
        .to_string();

    // Basic HTML tag stripping (for better results, use a proper HTML parser)
    let text = strip_html_tags(&html);

    // Truncate to reasonable length for AI context
    let truncated = if text.len() > 3000 {
        format!("{}...", &text[..3000])
    } else {
        text
    };

    Ok(truncated)
}

/// Basic HTML tag stripping
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    // Clean up whitespace
    result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html_tags() {
        let html = "<p>Hello <strong>world</strong>!</p>";
        assert_eq!(strip_html_tags(html), "Hello world!");

        let html2 = "<div><h1>Title</h1><p>Content</p></div>";
        assert_eq!(strip_html_tags(html2), "TitleContent");
    }

    #[test]
    fn test_escape_cql_escapes_special_chars() {
        assert_eq!(escape_cql("test\"quote"), r#"test\"quote"#);
        assert_eq!(escape_cql("test(paren"), r#"test\(paren"#);
        assert_eq!(escape_cql("test)paren"), r#"test\)paren"#);
        assert_eq!(escape_cql("test~tilde"), r#"test\~tilde"#);
        assert_eq!(escape_cql("test&and"), r#"test\&and"#);
        assert_eq!(escape_cql("test|or"), r#"test\|or"#);
        assert_eq!(escape_cql("test+plus"), r#"test\+plus"#);
        assert_eq!(escape_cql("test-minus"), r#"test\-minus"#);
    }

    #[test]
    fn test_escape_cql_no_special_chars() {
        assert_eq!(escape_cql("simple query"), "simple query");
    }

    #[test]
    fn test_canonicalize_url_removes_fragment() {
        assert_eq!(
            canonicalize_url("https://example.com/page#section"),
            "https://example.com/page"
        );
    }

    #[test]
    fn test_canonicalize_url_removes_query() {
        assert_eq!(
            canonicalize_url("https://example.com/page?param=value"),
            "https://example.com/page"
        );
    }

    #[test]
    fn test_canonicalize_url_handles_malformed() {
        // Malformed URLs fall back to original
        assert_eq!(canonicalize_url("not a url"), "not a url");
    }
}
