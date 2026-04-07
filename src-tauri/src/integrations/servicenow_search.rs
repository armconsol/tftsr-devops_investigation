use super::confluence_search::SearchResult;

/// Search ServiceNow Knowledge Base for content matching the query
pub async fn search_servicenow(
    instance_url: &str,
    query: &str,
    cookies: &[crate::integrations::webview_auth::Cookie],
) -> Result<Vec<SearchResult>, String> {
    let cookie_header = crate::integrations::webview_auth::cookies_to_header(cookies);
    let client = reqwest::Client::new();

    // Search Knowledge Base articles
    let search_url = format!(
        "{}/api/now/table/kb_knowledge?sysparm_query=textLIKE{}^ORshort_descriptionLIKE{}&sysparm_limit=5",
        instance_url.trim_end_matches('/'),
        urlencoding::encode(query),
        urlencoding::encode(query)
    );

    tracing::info!("Searching ServiceNow: {}", search_url);

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
        return Err(format!(
            "ServiceNow search failed with status {status}: {text}"
        ));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse ServiceNow search response: {e}"))?;

    let mut results = Vec::new();

    if let Some(result_array) = json["result"].as_array() {
        for item in result_array.iter().take(3) {
            // Take top 3 results
            let title = item["short_description"]
                .as_str()
                .unwrap_or("Untitled")
                .to_string();

            let sys_id = item["sys_id"].as_str().unwrap_or("").to_string();

            let url = format!(
                "{}/kb_view.do?sysparm_article={}",
                instance_url.trim_end_matches('/'),
                sys_id
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

            results.push(SearchResult {
                title,
                url,
                excerpt,
                content,
                source: "ServiceNow".to_string(),
            });
        }
    }

    Ok(results)
}

/// Search ServiceNow Incidents for related issues
pub async fn search_incidents(
    instance_url: &str,
    query: &str,
    cookies: &[crate::integrations::webview_auth::Cookie],
) -> Result<Vec<SearchResult>, String> {
    let cookie_header = crate::integrations::webview_auth::cookies_to_header(cookies);
    let client = reqwest::Client::new();

    // Search incidents
    let search_url = format!(
        "{}/api/now/table/incident?sysparm_query=short_descriptionLIKE{}^ORdescriptionLIKE{}&sysparm_limit=3&sysparm_display_value=true",
        instance_url.trim_end_matches('/'),
        urlencoding::encode(query),
        urlencoding::encode(query)
    );

    tracing::info!("Searching ServiceNow incidents: {}", search_url);

    let resp = client
        .get(&search_url)
        .header("Cookie", &cookie_header)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("ServiceNow incident search failed: {e}"))?;

    if !resp.status().is_success() {
        return Ok(Vec::new()); // Don't fail if incident search fails
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|_| "Failed to parse incident response".to_string())?;

    let mut results = Vec::new();

    if let Some(result_array) = json["result"].as_array() {
        for item in result_array.iter() {
            let number = item["number"].as_str().unwrap_or("Unknown");
            let title = format!(
                "Incident {}: {}",
                number,
                item["short_description"].as_str().unwrap_or("No title")
            );

            let sys_id = item["sys_id"].as_str().unwrap_or("");
            let url = format!(
                "{}/incident.do?sys_id={}",
                instance_url.trim_end_matches('/'),
                sys_id
            );

            let description = item["description"].as_str().unwrap_or("").to_string();

            let resolution = item["close_notes"].as_str().unwrap_or("").to_string();

            let content = format!("Description: {description}\nResolution: {resolution}");

            let excerpt = content.chars().take(200).collect::<String>();

            results.push(SearchResult {
                title,
                url,
                excerpt,
                content: Some(content),
                source: "ServiceNow".to_string(),
            });
        }
    }

    Ok(results)
}
