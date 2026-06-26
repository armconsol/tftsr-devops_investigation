/// Webview-based HTTP fetching that automatically includes HttpOnly cookies
/// Makes requests FROM the authenticated webview using JavaScript fetch API
///
/// This uses Tauri's window.location to pass results back (cross-document messaging)
use serde_json::Value;
use tauri::WebviewWindow;

use super::confluence_search::SearchResult;
use crate::integrations::query_expansion::expand_query;

/// Execute an HTTP request from within the webview context
/// This automatically includes all cookies (including HttpOnly) from the authenticated session
pub async fn fetch_from_webview<R: tauri::Runtime>(
    webview_window: &WebviewWindow<R>,
    url: &str,
    method: &str,
    body: Option<&str>,
) -> Result<Value, String> {
    let request_id = uuid::Uuid::now_v7().to_string();

    let (headers_js, body_js) = if let Some(b) = body {
        // For POST/PUT with JSON body
        (
            "headers: { 'Accept': 'application/json', 'Content-Type': 'application/json' }",
            format!(", body: JSON.stringify({b})"),
        )
    } else {
        // For GET requests
        ("headers: { 'Accept': 'application/json' }", String::new())
    };

    // Inject script that:
    // 1. Makes fetch request with credentials
    // 2. Uses window.location.hash to communicate results back
    let fetch_script = format!(
        r#"
        (async function() {{
            const requestId = '{request_id}';

            try {{
                const response = await fetch('{url}', {{
                    method: '{method}',
                    {headers_js},
                    credentials: 'include'{body_js}
                }});

                if (!response.ok) {{
                    window.location.hash = '#tftsr-error-' + requestId + '-' + encodeURIComponent(JSON.stringify({{
                        error: `HTTP ${{response.status}}: ${{response.statusText}}`
                    }}));
                    return;
                }}

                const data = await response.json();
                // Store in hash - we'll poll for this
                window.location.hash = '#tftsr-success-' + requestId + '-' + encodeURIComponent(JSON.stringify(data));
            }} catch (error) {{
                window.location.hash = '#tftsr-error-' + requestId + '-' + encodeURIComponent(JSON.stringify({{
                    error: error.message
                }}));
            }}
        }})();
        "#
    );

    // Execute the fetch
    webview_window
        .eval(&fetch_script)
        .map_err(|e| format!("Failed to execute fetch: {e}"))?;

    // Poll for result by checking window URL/hash
    for i in 0..50 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Get the current URL to check the hash
        if let Ok(url_str) = webview_window.url() {
            let url_string = url_str.to_string();

            // Check for success
            let success_marker = format!("#tftsr-success-{request_id}-");
            if url_string.contains(&success_marker) {
                // Extract the JSON from the hash
                if let Some(json_start) = url_string.find(&success_marker) {
                    let json_encoded = &url_string[json_start + success_marker.len()..];
                    if let Ok(decoded) = urlencoding::decode(json_encoded) {
                        // Clear the hash
                        webview_window.eval("window.location.hash = '';").ok();

                        // Parse JSON
                        if let Ok(result) = serde_json::from_str::<Value>(&decoded) {
                            tracing::info!("Webview fetch successful");
                            return Ok(result);
                        }
                    }
                }
            }

            // Check for error
            let error_marker = format!("#tftsr-error-{request_id}-");
            if url_string.contains(&error_marker) {
                if let Some(json_start) = url_string.find(&error_marker) {
                    let json_encoded = &url_string[json_start + error_marker.len()..];
                    if let Ok(decoded) = urlencoding::decode(json_encoded) {
                        // Clear the hash
                        webview_window.eval("window.location.hash = '';").ok();

                        return Err(format!("Webview fetch error: {decoded}"));
                    }
                }
            }
        }

        if i % 10 == 0 {
            tracing::debug!("Waiting for webview fetch... ({}s)", i / 10);
        }
    }

    Err("Timeout waiting for webview fetch response (5s)".to_string())
}

/// Search Confluence using webview fetch (includes HttpOnly cookies automatically)
pub async fn search_confluence_webview<R: tauri::Runtime>(
    webview_window: &WebviewWindow<R>,
    base_url: &str,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    let expanded_queries = expand_query(query);

    let mut all_results = Vec::new();

    for expanded_query in expanded_queries.iter().take(3) {
        // Extract keywords from the query for better search
        // Remove common words and extract important terms
        let keywords = extract_keywords(expanded_query);

        // Build CQL query with OR logic for keywords
        let cql = if keywords.len() > 1 {
            // Multiple keywords - search for any of them
            let keyword_conditions: Vec<String> =
                keywords.iter().map(|k| format!("text ~ \"{k}\"")).collect();
            keyword_conditions.join(" OR ")
        } else if !keywords.is_empty() {
            // Single keyword
            let keyword = &keywords[0];
            format!("text ~ \"{keyword}\"")
        } else {
            // Fallback to expanded query
            format!("text ~ \"{expanded_query}\"")
        };

        let search_url = format!(
            "{}/rest/api/search?cql={}&limit=10",
            base_url.trim_end_matches('/'),
            urlencoding::encode(&cql)
        );

        tracing::info!("Executing Confluence search via webview with CQL: {}", cql);

        let response = fetch_from_webview(webview_window, &search_url, "GET", None).await?;

        if let Some(results_array) = response.get("results").and_then(|v| v.as_array()) {
            for item in results_array.iter().take(5) {
                let title = item["title"].as_str().unwrap_or("Untitled").to_string();
                let content_id = item["content"]["id"].as_str();
                let space_key = item["content"]["space"]["key"].as_str();

                let url = if let (Some(id), Some(space)) = (content_id, space_key) {
                    format!(
                        "{}/display/{}/{}",
                        base_url.trim_end_matches('/'),
                        space,
                        id
                    )
                } else {
                    base_url.to_string()
                };

                let excerpt = item["excerpt"]
                    .as_str()
                    .unwrap_or("")
                    .replace("<span class=\"highlight\">", "")
                    .replace("</span>", "");

                // Fetch full page content
                let content = if let Some(id) = content_id {
                    let content_url = format!(
                        "{}/rest/api/content/{id}?expand=body.storage",
                        base_url.trim_end_matches('/')
                    );
                    if let Ok(content_resp) =
                        fetch_from_webview(webview_window, &content_url, "GET", None).await
                    {
                        if let Some(body) = content_resp
                            .get("body")
                            .and_then(|b| b.get("storage"))
                            .and_then(|s| s.get("value"))
                            .and_then(|v| v.as_str())
                        {
                            let text = strip_html_simple(body);
                            Some(if text.len() > 3000 {
                                format!("{}...", &text[..3000])
                            } else {
                                text
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                all_results.push(SearchResult {
                    title,
                    url,
                    excerpt: excerpt.chars().take(300).collect(),
                    content,
                    source: "Confluence".to_string(),
                });
            }
        }
    }

    all_results.sort_by(|a, b| a.url.cmp(&b.url));
    all_results.dedup_by(|a, b| a.url == b.url);

    tracing::info!(
        "Confluence webview search returned {} results",
        all_results.len()
    );
    Ok(all_results)
}

/// Extract keywords from a search query
/// Removes stop words and extracts important terms
fn extract_keywords(query: &str) -> Vec<String> {
    // Common stop words to filter out
    let stop_words = vec![
        "how", "do", "i", "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "having", "do", "does", "did", "doing", "will", "would", "should",
        "could", "can", "may", "might", "must", "to", "from", "in", "on", "at", "by", "for",
        "with", "about", "as", "of", "or", "and", "but", "not", "what", "when", "where", "which",
        "who",
    ];

    let mut keywords = Vec::new();

    // Split on whitespace and punctuation
    for word in query.split(|c: char| c.is_whitespace() || c == '?' || c == '!' || c == '.') {
        let cleaned = word.trim().to_lowercase();

        // Skip if empty, too short, or a stop word
        if cleaned.is_empty() || cleaned.len() < 2 || stop_words.contains(&cleaned.as_str()) {
            continue;
        }

        // Keep version numbers (e.g., "1.0.12")
        if cleaned.contains('.') && cleaned.chars().any(|c| c.is_numeric()) {
            keywords.push(cleaned);
            continue;
        }

        // Keep ticket numbers and IDs (pure numbers >= 3 digits)
        if cleaned.chars().all(|c| c.is_numeric()) && cleaned.len() >= 3 {
            keywords.push(cleaned);
            continue;
        }

        // Keep if it has letters
        if cleaned.chars().any(|c| c.is_alphabetic()) {
            keywords.push(cleaned);
        }
    }

    // Deduplicate
    keywords.sort();
    keywords.dedup();

    keywords
}

/// Simple HTML tag stripping (for content preview)
fn strip_html_simple(html: &str) -> String {
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

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Search ServiceNow using webview fetch
pub async fn search_servicenow_webview<R: tauri::Runtime>(
    webview_window: &WebviewWindow<R>,
    instance_url: &str,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    let expanded_queries = expand_query(query);

    let mut all_results = Vec::new();

    for expanded_query in expanded_queries.iter().take(3) {
        // Search knowledge base
        let kb_url = format!(
            "{}/api/now/table/kb_knowledge?sysparm_query=textLIKE{}^ORshort_descriptionLIKE{}&sysparm_limit=3",
            instance_url.trim_end_matches('/'),
            urlencoding::encode(expanded_query),
            urlencoding::encode(expanded_query)
        );

        tracing::info!("Executing ServiceNow KB search via webview with expanded query");

        if let Ok(kb_response) = fetch_from_webview(webview_window, &kb_url, "GET", None).await {
            if let Some(kb_array) = kb_response.get("result").and_then(|v| v.as_array()) {
                for item in kb_array {
                    let title = item["short_description"]
                        .as_str()
                        .unwrap_or("Untitled")
                        .to_string();
                    let sys_id = item["sys_id"].as_str().unwrap_or("");
                    let url = format!(
                        "{}/kb_view.do?sysparm_article={sys_id}",
                        instance_url.trim_end_matches('/')
                    );
                    let text = item["text"].as_str().unwrap_or("");
                    let excerpt = text.chars().take(300).collect();
                    let content = Some(if text.len() > 3000 {
                        format!("{}...", &text[..3000])
                    } else {
                        text.to_string()
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

        // Search incidents
        let inc_url = format!(
            "{}/api/now/table/incident?sysparm_query=short_descriptionLIKE{}^ORdescriptionLIKE{}&sysparm_limit=3&sysparm_display_value=true",
            instance_url.trim_end_matches('/'),
            urlencoding::encode(expanded_query),
            urlencoding::encode(expanded_query)
        );

        if let Ok(inc_response) = fetch_from_webview(webview_window, &inc_url, "GET", None).await {
            if let Some(inc_array) = inc_response.get("result").and_then(|v| v.as_array()) {
                for item in inc_array {
                    let number = item["number"].as_str().unwrap_or("Unknown");
                    let title = format!(
                        "Incident {}: {}",
                        number,
                        item["short_description"].as_str().unwrap_or("No title")
                    );
                    let sys_id = item["sys_id"].as_str().unwrap_or("");
                    let url = format!(
                        "{}/incident.do?sys_id={sys_id}",
                        instance_url.trim_end_matches('/')
                    );
                    let description = item["description"].as_str().unwrap_or("");
                    let resolution = item["close_notes"].as_str().unwrap_or("");
                    let content = format!("Description: {description}\nResolution: {resolution}");
                    let excerpt = content.chars().take(200).collect();

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
    }

    all_results.sort_by(|a, b| a.url.cmp(&b.url));
    all_results.dedup_by(|a, b| a.url == b.url);

    tracing::info!(
        "ServiceNow webview search returned {} results",
        all_results.len()
    );
    Ok(all_results)
}

/// Search Azure DevOps wiki using webview fetch
pub async fn search_azuredevops_wiki_webview<R: tauri::Runtime>(
    webview_window: &WebviewWindow<R>,
    org_url: &str,
    project: &str,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    let expanded_queries = expand_query(query);

    let mut all_results = Vec::new();

    for expanded_query in expanded_queries.iter().take(3) {
        // Extract keywords for better search
        let keywords = extract_keywords(expanded_query);

        let search_text = if !keywords.is_empty() {
            keywords.join(" ")
        } else {
            expanded_query.clone()
        };

        // Azure DevOps wiki search API
        let search_url = format!(
            "{}/{}/_apis/wiki/wikis?api-version=7.0",
            org_url.trim_end_matches('/'),
            urlencoding::encode(project)
        );

        tracing::info!(
            "Executing Azure DevOps wiki search via webview for: {}",
            search_text
        );

        // First, get list of wikis
        let wikis_response = fetch_from_webview(webview_window, &search_url, "GET", None).await?;

        if let Some(wikis_array) = wikis_response.get("value").and_then(|v| v.as_array()) {
            // Search each wiki
            for wiki in wikis_array.iter().take(3) {
                let wiki_id = wiki["id"].as_str().unwrap_or("");

                if wiki_id.is_empty() {
                    continue;
                }

                // Search wiki pages
                let pages_url = format!(
                    "{}/{}/_apis/wiki/wikis/{}/pages?recursionLevel=Full&includeContent=true&api-version=7.0",
                    org_url.trim_end_matches('/'),
                    urlencoding::encode(project),
                    urlencoding::encode(wiki_id)
                );

                if let Ok(pages_response) =
                    fetch_from_webview(webview_window, &pages_url, "GET", None).await
                {
                    // Try to get "page" field, or use the response itself if it's the page object
                    if let Some(page) = pages_response.get("page") {
                        search_page_recursive(
                            page,
                            &search_text,
                            org_url,
                            project,
                            wiki_id,
                            &mut all_results,
                        );
                    } else {
                        // Response might be the page object itself
                        search_page_recursive(
                            &pages_response,
                            &search_text,
                            org_url,
                            project,
                            wiki_id,
                            &mut all_results,
                        );
                    }
                }
            }
        }
    }

    all_results.sort_by(|a, b| a.url.cmp(&b.url));
    all_results.dedup_by(|a, b| a.url == b.url);

    tracing::info!(
        "Azure DevOps wiki webview search returned {} results",
        all_results.len()
    );
    Ok(all_results)
}

/// Recursively search through wiki pages for matching content
fn search_page_recursive(
    page: &Value,
    search_text: &str,
    org_url: &str,
    _project: &str,
    wiki_id: &str,
    results: &mut Vec<SearchResult>,
) {
    let search_lower = search_text.to_lowercase();

    // Check current page
    if let Some(path) = page.get("path").and_then(|p| p.as_str()) {
        let content = page.get("content").and_then(|c| c.as_str()).unwrap_or("");
        let content_lower = content.to_lowercase();

        // Simple relevance check
        let matches = search_lower
            .split_whitespace()
            .filter(|word| content_lower.contains(word))
            .count();

        if matches > 0 {
            let page_id = page.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
            let title = path.trim_start_matches('/').replace('/', " > ");
            let url = format!(
                "{}/_wiki/wikis/{}/{}/{}",
                org_url.trim_end_matches('/'),
                urlencoding::encode(wiki_id),
                page_id,
                urlencoding::encode(path.trim_start_matches('/'))
            );

            // Create excerpt from first occurrence
            let excerpt = if let Some(pos) =
                content_lower.find(search_lower.split_whitespace().next().unwrap_or(""))
            {
                let start = pos.saturating_sub(50);
                let end = (pos + 200).min(content.len());
                format!("...{}", &content[start..end])
            } else {
                content.chars().take(200).collect()
            };

            let result_content = if content.len() > 3000 {
                format!("{}...", &content[..3000])
            } else {
                content.to_string()
            };

            results.push(SearchResult {
                title,
                url,
                excerpt,
                content: Some(result_content),
                source: "Azure DevOps Wiki".to_string(),
            });
        }
    }

    // Recurse into subpages
    if let Some(subpages) = page.get("subPages").and_then(|s| s.as_array()) {
        for subpage in subpages {
            search_page_recursive(subpage, search_text, org_url, _project, wiki_id, results);
        }
    }
}

/// Search Azure DevOps work items using webview fetch
pub async fn search_azuredevops_workitems_webview<R: tauri::Runtime>(
    webview_window: &WebviewWindow<R>,
    org_url: &str,
    project: &str,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    let expanded_queries = expand_query(query);

    let mut all_results = Vec::new();

    for expanded_query in expanded_queries.iter().take(3) {
        // Extract keywords
        let keywords = extract_keywords(expanded_query);

        // Check if query contains a work item ID (pure number)
        let work_item_id: Option<i64> = keywords
            .iter()
            .filter(|k| k.chars().all(|c| c.is_numeric()))
            .filter_map(|k| k.parse::<i64>().ok())
            .next();

        // Build WIQL query
        let wiql_query = if let Some(id) = work_item_id {
            // Search by specific ID
            format!(
                "SELECT [System.Id], [System.Title], [System.Description], [System.WorkItemType] \
                 FROM WorkItems WHERE [System.Id] = {id}"
            )
        } else {
            // Search by text in title/description
            let search_terms = if !keywords.is_empty() {
                keywords.join(" ")
            } else {
                expanded_query.clone()
            };

            // Use CONTAINS for text search (case-insensitive)
            format!(
                "SELECT [System.Id], [System.Title], [System.Description], [System.WorkItemType] \
                 FROM WorkItems WHERE [System.TeamProject] = '{project}' \
                 AND ([System.Title] CONTAINS '{search_terms}' OR [System.Description] CONTAINS '{search_terms}') \
                 ORDER BY [System.ChangedDate] DESC"
            )
        };

        let wiql_url = format!(
            "{}/{}/_apis/wit/wiql?api-version=7.0",
            org_url.trim_end_matches('/'),
            urlencoding::encode(project)
        );

        let body = serde_json::json!({
            "query": wiql_query
        })
        .to_string();

        tracing::info!("Executing Azure DevOps work item search via webview");
        tracing::debug!("WIQL query: {}", wiql_query);
        tracing::debug!("Request URL: {}", wiql_url);

        let wiql_response =
            fetch_from_webview(webview_window, &wiql_url, "POST", Some(&body)).await?;

        if let Some(work_items) = wiql_response.get("workItems").and_then(|v| v.as_array()) {
            // Fetch details for first 5 work items
            for item in work_items.iter().take(5) {
                if let Some(id) = item.get("id").and_then(|i| i.as_i64()) {
                    let details_url = format!(
                        "{}/_apis/wit/workitems/{}?api-version=7.0",
                        org_url.trim_end_matches('/'),
                        id
                    );

                    if let Ok(details) =
                        fetch_from_webview(webview_window, &details_url, "GET", None).await
                    {
                        if let Some(fields) = details.get("fields") {
                            let title = fields
                                .get("System.Title")
                                .and_then(|t| t.as_str())
                                .unwrap_or("Untitled");
                            let work_item_type = fields
                                .get("System.WorkItemType")
                                .and_then(|t| t.as_str())
                                .unwrap_or("Item");
                            let description = fields
                                .get("System.Description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("");

                            let clean_description = strip_html_simple(description);
                            let excerpt = clean_description.chars().take(200).collect();

                            let url =
                                format!("{}/_workitems/edit/{id}", org_url.trim_end_matches('/'));

                            let full_content = if clean_description.len() > 3000 {
                                format!("{}...", &clean_description[..3000])
                            } else {
                                clean_description.clone()
                            };

                            all_results.push(SearchResult {
                                title: format!("{work_item_type} #{id}: {title}"),
                                url,
                                excerpt,
                                content: Some(full_content),
                                source: "Azure DevOps".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    all_results.sort_by(|a, b| a.url.cmp(&b.url));
    all_results.dedup_by(|a, b| a.url == b.url);

    tracing::info!(
        "Azure DevOps work items webview search returned {} results",
        all_results.len()
    );
    Ok(all_results)
}

/// Add a comment to an Azure DevOps work item
pub async fn add_azuredevops_comment_webview<R: tauri::Runtime>(
    webview_window: &WebviewWindow<R>,
    org_url: &str,
    work_item_id: i64,
    comment_text: &str,
) -> Result<String, String> {
    let comment_url = format!(
        "{}/_apis/wit/workitems/{work_item_id}/comments?api-version=7.0",
        org_url.trim_end_matches('/')
    );

    let body = serde_json::json!({
        "text": comment_text
    })
    .to_string();

    tracing::info!("Adding comment to Azure DevOps work item {}", work_item_id);

    let response = fetch_from_webview(webview_window, &comment_url, "POST", Some(&body)).await?;

    // Extract comment ID from response
    let comment_id = response
        .get("id")
        .and_then(|id| id.as_i64())
        .ok_or_else(|| "Failed to get comment ID from response".to_string())?;

    tracing::info!("Successfully added comment {comment_id} to work item {work_item_id}");
    Ok(format!("Comment added successfully (ID: {comment_id})"))
}
