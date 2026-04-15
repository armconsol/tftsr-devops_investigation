use super::confluence_search::SearchResult;
use crate::integrations::query_expansion::expand_query;

/// Search Azure DevOps Wiki for content matching the query
pub async fn search_wiki(
    org_url: &str,
    project: &str,
    query: &str,
    cookies: &[crate::integrations::webview_auth::Cookie],
) -> Result<Vec<SearchResult>, String> {
    let cookie_header = crate::integrations::webview_auth::cookies_to_header(cookies);
    let client = reqwest::Client::new();

    let expanded_queries = expand_query(query);

    let mut all_results = Vec::new();

    for expanded_query in expanded_queries.iter().take(3) {
        // Use Azure DevOps Search API
        let search_url = format!(
            "{}/_apis/search/wikisearchresults?api-version=7.0",
            org_url.trim_end_matches('/')
        );

        let search_body = serde_json::json!({
            "searchText": expanded_query,
            "$top": 5,
            "filters": {
                "ProjectFilters": [project]
            }
        });

        tracing::info!(
            "Searching Azure DevOps Wiki with expanded query: {}",
            search_url
        );

        let resp = client
            .post(&search_url)
            .header("Cookie", &cookie_header)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&search_body)
            .send()
            .await
            .map_err(|e| format!("Azure DevOps wiki search failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!("Azure DevOps wiki search failed with status {status}: {text}");
            continue;
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse ADO wiki search response: {e}"))?;

        if let Some(results_array) = json["results"].as_array() {
            for item in results_array.iter().take(3) {
                let title = item["fileName"].as_str().unwrap_or("Untitled").to_string();

                let path = item["path"].as_str().unwrap_or("");
                let url = format!(
                    "{}/_wiki/wikis/{}/{}",
                    org_url.trim_end_matches('/'),
                    project,
                    path
                );

                let excerpt = item["content"]
                    .as_str()
                    .unwrap_or("")
                    .chars()
                    .take(300)
                    .collect::<String>();

                // Fetch full wiki page content
                let content = if let Some(wiki_id) = item["wiki"]["id"].as_str() {
                    if let Some(page_path) = item["path"].as_str() {
                        fetch_wiki_page(org_url, wiki_id, page_path, &cookie_header)
                            .await
                            .ok()
                    } else {
                        None
                    }
                } else {
                    None
                };

                all_results.push(SearchResult {
                    title,
                    url,
                    excerpt,
                    content,
                    source: "Azure DevOps".to_string(),
                });
            }
        }
    }

    all_results.sort_by(|a, b| a.url.cmp(&b.url));
    all_results.dedup_by(|a, b| a.url == b.url);

    Ok(all_results)
}

/// Fetch full wiki page content
async fn fetch_wiki_page(
    org_url: &str,
    wiki_id: &str,
    page_path: &str,
    cookie_header: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let page_url = format!(
        "{}/_apis/wiki/wikis/{}/pages?path={}&api-version=7.0&includeContent=true",
        org_url.trim_end_matches('/'),
        wiki_id,
        urlencoding::encode(page_path)
    );

    let resp = client
        .get(&page_url)
        .header("Cookie", cookie_header)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch wiki page: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(format!("Failed to fetch wiki page: {status}"));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse wiki page: {e}"))?;

    let content = json["content"].as_str().unwrap_or("").to_string();

    // Truncate to reasonable length
    let truncated = if content.len() > 3000 {
        format!("{}...", &content[..3000])
    } else {
        content
    };

    Ok(truncated)
}

/// Search Azure DevOps Work Items for related issues
pub async fn search_work_items(
    org_url: &str,
    project: &str,
    query: &str,
    cookies: &[crate::integrations::webview_auth::Cookie],
) -> Result<Vec<SearchResult>, String> {
    let cookie_header = crate::integrations::webview_auth::cookies_to_header(cookies);
    let client = reqwest::Client::new();

    let expanded_queries = expand_query(query);

    let mut all_results = Vec::new();

    for expanded_query in expanded_queries.iter().take(3) {
        // Use WIQL (Work Item Query Language)
        let wiql_url = format!(
            "{}/_apis/wit/wiql?api-version=7.0",
            org_url.trim_end_matches('/')
        );

        let wiql_query = format!(
            "SELECT [System.Id], [System.Title], [System.Description], [System.State] FROM WorkItems WHERE [System.TeamProject] = '{project}' AND ([System.Title] CONTAINS '{expanded_query}' OR [System.Description] CONTAINS '{expanded_query}') ORDER BY [System.ChangedDate] DESC"
        );

        let wiql_body = serde_json::json!({
            "query": wiql_query
        });

        tracing::info!("Searching Azure DevOps work items with expanded query");

        let resp = client
            .post(&wiql_url)
            .header("Cookie", &cookie_header)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&wiql_body)
            .send()
            .await
            .map_err(|e| format!("ADO work item search failed: {e}"))?;

        if !resp.status().is_success() {
            continue; // Don't fail if work item search fails
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|_| "Failed to parse work item response".to_string())?;

        if let Some(work_items) = json["workItems"].as_array() {
            // Fetch details for top 3 work items
            for item in work_items.iter().take(3) {
                if let Some(id) = item["id"].as_i64() {
                    if let Ok(work_item) =
                        fetch_work_item_details(org_url, id, &cookie_header).await
                    {
                        all_results.push(work_item);
                    }
                }
            }
        }
    }

    all_results.sort_by(|a, b| a.url.cmp(&b.url));
    all_results.dedup_by(|a, b| a.url == b.url);

    Ok(all_results)
}

/// Fetch work item details
async fn fetch_work_item_details(
    org_url: &str,
    id: i64,
    cookie_header: &str,
) -> Result<SearchResult, String> {
    let client = reqwest::Client::new();
    let item_url = format!(
        "{}/_apis/wit/workitems/{}?api-version=7.0",
        org_url.trim_end_matches('/'),
        id
    );

    let resp = client
        .get(&item_url)
        .header("Cookie", cookie_header)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch work item: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(format!("Failed to fetch work item: {status}"));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse work item: {e}"))?;

    let fields = &json["fields"];
    let title = format!(
        "Work Item {}: {}",
        id,
        fields["System.Title"].as_str().unwrap_or("No title")
    );

    let url = json["_links"]["html"]["href"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let description = fields["System.Description"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let state = fields["System.State"].as_str().unwrap_or("Unknown");
    let content = format!("State: {state}\n\nDescription: {description}");

    let excerpt = content.chars().take(200).collect::<String>();

    Ok(SearchResult {
        title,
        url,
        excerpt,
        content: Some(content),
        source: "Azure DevOps".to_string(),
    })
}
