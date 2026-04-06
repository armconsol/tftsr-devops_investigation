use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub excerpt: String,
    pub content: Option<String>,
    pub source: String, // "confluence", "servicenow", "azuredevops"
}

/// Search Confluence for content matching the query
pub async fn search_confluence(
    base_url: &str,
    query: &str,
    cookies: &[crate::integrations::webview_auth::Cookie],
) -> Result<Vec<SearchResult>, String> {
    let cookie_header = crate::integrations::webview_auth::cookies_to_header(cookies);
    let client = reqwest::Client::new();

    // Use Confluence CQL search
    let search_url = format!(
        "{}/rest/api/search?cql=text~\"{}\"&limit=5",
        base_url.trim_end_matches('/'),
        urlencoding::encode(query)
    );

    tracing::info!("Searching Confluence: {}", search_url);

    let resp = client
        .get(&search_url)
        .header("Cookie", &cookie_header)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Confluence search request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Confluence search failed with status {}: {}",
            status, text
        ));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Confluence search response: {}", e))?;

    let mut results = Vec::new();

    if let Some(results_array) = json["results"].as_array() {
        for item in results_array.iter().take(3) {
            // Take top 3 results
            let title = item["title"].as_str().unwrap_or("Untitled").to_string();

            let id = item["content"]["id"].as_str();
            let space_key = item["content"]["space"]["key"].as_str();

            // Build URL
            let url = if let (Some(id_str), Some(space)) = (id, space_key) {
                format!(
                    "{}/display/{}/{}",
                    base_url.trim_end_matches('/'),
                    space,
                    id_str
                )
            } else {
                base_url.to_string()
            };

            // Get excerpt from search result
            let excerpt = item["excerpt"]
                .as_str()
                .unwrap_or("")
                .to_string()
                .replace("<span class=\"highlight\">", "")
                .replace("</span>", "");

            // Fetch full page content
            let content = if let Some(content_id) = id {
                fetch_page_content(base_url, content_id, &cookie_header)
                    .await
                    .ok()
            } else {
                None
            };

            results.push(SearchResult {
                title,
                url,
                excerpt,
                content,
                source: "Confluence".to_string(),
            });
        }
    }

    Ok(results)
}

/// Fetch full content of a Confluence page
async fn fetch_page_content(
    base_url: &str,
    page_id: &str,
    cookie_header: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let content_url = format!(
        "{}/rest/api/content/{}?expand=body.storage",
        base_url.trim_end_matches('/'),
        page_id
    );

    let resp = client
        .get(&content_url)
        .header("Cookie", cookie_header)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch page content: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Failed to fetch page: {}", resp.status()));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse page content: {}", e))?;

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
}
