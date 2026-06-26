use super::confluence_search::SearchResult;
use crate::integrations::query_expansion::expand_query;

const MAX_EXPANDED_QUERIES: usize = 3;

/// Search ServiceNow Knowledge Base for content matching the query
pub async fn search_servicenow(
    instance_url: &str,
    query: &str,
    cookies: &[crate::integrations::webview_auth::Cookie],
) -> Result<Vec<SearchResult>, String> {
    let cookie_header = crate::integrations::webview_auth::cookies_to_header(cookies);
    let client = reqwest::Client::new();

    let expanded_queries = expand_query(query);

    let mut all_results = Vec::new();

    for expanded_query in expanded_queries.iter().take(MAX_EXPANDED_QUERIES) {
        // Search Knowledge Base articles
        let search_url = format!(
            "{}/api/now/table/kb_knowledge?sysparm_query=textLIKE{}^ORshort_descriptionLIKE{}&sysparm_limit=5",
            instance_url.trim_end_matches('/'),
            urlencoding::encode(expanded_query),
            urlencoding::encode(expanded_query)
        );

        tracing::info!("Searching ServiceNow with query: {expanded_query}");

        let resp = client
            .get(&search_url)
            .header("Cookie", &cookie_header)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("ServiceNow search request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!("ServiceNow search failed with status {status}: {text}");
            continue;
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse ServiceNow search response: {e}"))?;

        if let Some(result_array) = json["result"].as_array() {
            for item in result_array.iter().take(MAX_EXPANDED_QUERIES) {
                // Take top 3 results
                let title = item["short_description"]
                    .as_str()
                    .unwrap_or("Untitled")
                    .to_string();

                let sys_id = item["sys_id"].as_str().unwrap_or("").to_string();

                let url = format!(
                    "{}/kb_view.do?sysparm_article={sys_id}",
                    instance_url.trim_end_matches('/')
                );

                let excerpt = item["text"]
                    .as_str()
                    .unwrap_or("")
                    .chars()
                    .take(300)
                    .collect::<String>();

                // Get full article content
                let content = item["text"].as_str().map(|text| {
                    if text.len() > 3000 {
                        format!("{}...", &text[..3000])
                    } else {
                        text.to_string()
                    }
                });

                all_results.push(SearchResult {
                    title,
                    url,
                    excerpt,
                    content,
                    source: "ServiceNow".to_string(),
                });
            }
        }
    }

    all_results.sort_by(|a, b| a.url.cmp(&b.url));
    all_results.dedup_by(|a, b| a.url == b.url);

    Ok(all_results)
}

/// Search ServiceNow Incidents for related issues
pub async fn search_incidents(
    instance_url: &str,
    query: &str,
    cookies: &[crate::integrations::webview_auth::Cookie],
) -> Result<Vec<SearchResult>, String> {
    let cookie_header = crate::integrations::webview_auth::cookies_to_header(cookies);
    let client = reqwest::Client::new();

    let expanded_queries = expand_query(query);

    let mut all_results = Vec::new();

    for expanded_query in expanded_queries.iter().take(MAX_EXPANDED_QUERIES) {
        // Search incidents
        let search_url = format!(
            "{}/api/now/table/incident?sysparm_query=short_descriptionLIKE{}^ORdescriptionLIKE{}&sysparm_limit=3&sysparm_display_value=true",
            instance_url.trim_end_matches('/'),
            urlencoding::encode(expanded_query),
            urlencoding::encode(expanded_query)
        );

        tracing::info!("Searching ServiceNow incidents with query: {expanded_query}");

        let resp = client
            .get(&search_url)
            .header("Cookie", &cookie_header)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("ServiceNow incident search failed: {e}"))?;

        if !resp.status().is_success() {
            continue; // Don't fail if incident search fails
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|_| "Failed to parse incident response".to_string())?;

        if let Some(result_array) = json["result"].as_array() {
            for item in result_array.iter() {
                let number = item["number"].as_str().unwrap_or("Unknown");
                let title = format!(
                    "Incident {number}: {}",
                    item["short_description"].as_str().unwrap_or("No title")
                );

                let sys_id = item["sys_id"].as_str().unwrap_or("");
                let url = format!(
                    "{}/incident.do?sys_id={sys_id}",
                    instance_url.trim_end_matches('/')
                );

                let description = item["description"].as_str().unwrap_or("").to_string();

                let resolution = item["close_notes"].as_str().unwrap_or("").to_string();

                let content = format!("Description: {description}\nResolution: {resolution}");

                let excerpt = content.chars().take(200).collect::<String>();

                all_results.push(SearchResult {
                    title,
                    url,
                    excerpt,
                    content: Some(content),
                    source: "ServiceNow".to_string(),
                });
            }
        }
    }

    all_results.sort_by(|a, b| a.url.cmp(&b.url));
    all_results.dedup_by(|a, b| a.url == b.url);

    Ok(all_results)
}
