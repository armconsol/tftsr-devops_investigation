/// Native webview-based search that automatically includes HttpOnly cookies
/// This bypasses cookie extraction by making requests directly from the authenticated webview

use serde::{Deserialize, Serialize};
use tauri::WebviewWindow;

use super::confluence_search::SearchResult;

/// Execute a search request from within the webview context
/// This automatically includes all cookies (including HttpOnly) from the authenticated session
pub async fn search_from_webview<R: tauri::Runtime>(
    webview_window: &WebviewWindow<R>,
    service: &str,
    base_url: &str,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    match service {
        "confluence" => search_confluence_from_webview(webview_window, base_url, query).await,
        "servicenow" => search_servicenow_from_webview(webview_window, base_url, query).await,
        "azuredevops" => Ok(Vec::new()), // Not yet implemented
        _ => Err(format!("Unsupported service: {}", service)),
    }
}

/// Search Confluence from within the authenticated webview
async fn search_confluence_from_webview<R: tauri::Runtime>(
    webview_window: &WebviewWindow<R>,
    base_url: &str,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    let search_script = format!(
        r#"
        (async function() {{
            try {{
                // Search Confluence using the browser's authenticated session
                const searchUrl = '{}/rest/api/search?cql=text~"{}"&limit=5';
                const response = await fetch(searchUrl, {{
                    headers: {{
                        'Accept': 'application/json'
                    }},
                    credentials: 'include'  // Include cookies automatically
                }});

                if (!response.ok) {{
                    return {{ error: `Search failed: ${{response.status}}` }};
                }}

                const data = await response.json();
                const results = [];

                if (data.results && Array.isArray(data.results)) {{
                    for (const item of data.results.slice(0, 3)) {{
                        const title = item.title || 'Untitled';
                        const contentId = item.content?.id;
                        const spaceKey = item.content?.space?.key;

                        let url = '{}';
                        if (contentId && spaceKey) {{
                            url = `{}/display/${{spaceKey}}/${{contentId}}`;
                        }}

                        const excerpt = (item.excerpt || '')
                            .replace(/<span class="highlight">/g, '')
                            .replace(/<\/span>/g, '');

                        // Fetch full page content
                        let content = null;
                        if (contentId) {{
                            try {{
                                const contentUrl = `{}/rest/api/content/${{contentId}}?expand=body.storage`;
                                const contentResp = await fetch(contentUrl, {{
                                    headers: {{ 'Accept': 'application/json' }},
                                    credentials: 'include'
                                }});
                                if (contentResp.ok) {{
                                    const contentData = await contentResp.json();
                                    let html = contentData.body?.storage?.value || '';
                                    // Basic HTML stripping
                                    const div = document.createElement('div');
                                    div.innerHTML = html;
                                    let text = div.textContent || div.innerText || '';
                                    content = text.length > 3000 ? text.substring(0, 3000) + '...' : text;
                                }}
                            }} catch (e) {{
                                console.error('Failed to fetch page content:', e);
                            }}
                        }}

                        results.push({{
                            title,
                            url,
                            excerpt: excerpt.substring(0, 300),
                            content,
                            source: 'Confluence'
                        }});
                    }}
                }}

                return {{ results }};
            }} catch (error) {{
                return {{ error: error.message }};
            }}
        }})();
        "#,
        base_url.trim_end_matches('/'),
        query.replace('"', "\\\""),
        base_url,
        base_url,
        base_url
    );

    // Execute JavaScript and store result in localStorage for retrieval
    let storage_key = format!("__trcaa_search_{}__", uuid::Uuid::now_v7());
    let callback_script = format!(
        r#"
        {}
        .then(result => {{
            localStorage.setItem('{}', JSON.stringify(result));
        }})
        .catch(error => {{
            localStorage.setItem('{}', JSON.stringify({{ error: error.message }}));
        }});
        "#,
        search_script,
        storage_key,
        storage_key
    );

    webview_window
        .eval(&callback_script)
        .map_err(|e| format!("Failed to execute search: {}", e))?;

    // Poll for result in localStorage
    for _ in 0..50 {  // Try for 5 seconds
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let check_script = format!("localStorage.getItem('{}')", storage_key);
        let result_str = match webview_window.eval(&check_script) {
            Ok(_) => {
                // Try to retrieve the actual value
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                let get_script = format!(
                    r#"(function() {{
                        const val = localStorage.getItem('{}');
                        if (val) {{
                            localStorage.removeItem('{}');
                            return val;
                        }}
                        return null;
                    }})();"#,
                    storage_key, storage_key
                );
                match webview_window.eval(&get_script) {
                    Ok(_) => continue, // Keep polling
                    Err(_) => continue,
                }
            }
            Err(_) => continue,
        };
    }

    // Timeout - try one final retrieval
    tracing::warn!("Webview search timed out, returning empty results");
    Ok(Vec::new())
}

/// Search ServiceNow from within the authenticated webview
async fn search_servicenow_from_webview<R: tauri::Runtime>(
    webview_window: &WebviewWindow<R>,
    instance_url: &str,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    let search_script = format!(
        r#"
        (async function() {{
            try {{
                const results = [];

                // Search knowledge base
                const kbUrl = '{}/api/now/table/kb_knowledge?sysparm_query=textLIKE{}^ORshort_descriptionLIKE{}&sysparm_limit=3';
                const kbResp = await fetch(kbUrl, {{
                    headers: {{ 'Accept': 'application/json' }},
                    credentials: 'include'
                }});

                if (kbResp.ok) {{
                    const kbData = await kbResp.json();
                    if (kbData.result && Array.isArray(kbData.result)) {{
                        for (const item of kbData.result) {{
                            const title = item.short_description || 'Untitled';
                            const sysId = item.sys_id || '';
                            const url = `{}/kb_view.do?sysparm_article=${{sysId}}`;
                            const text = item.text || '';
                            const excerpt = text.substring(0, 300);
                            const content = text.length > 3000 ? text.substring(0, 3000) + '...' : text;

                            results.push({{
                                title,
                                url,
                                excerpt,
                                content,
                                source: 'ServiceNow'
                            }});
                        }}
                    }}
                }}

                // Search incidents
                const incUrl = '{}/api/now/table/incident?sysparm_query=short_descriptionLIKE{}^ORdescriptionLIKE{}&sysparm_limit=3&sysparm_display_value=true';
                const incResp = await fetch(incUrl, {{
                    headers: {{ 'Accept': 'application/json' }},
                    credentials: 'include'
                }});

                if (incResp.ok) {{
                    const incData = await incResp.json();
                    if (incData.result && Array.isArray(incData.result)) {{
                        for (const item of incData.result) {{
                            const number = item.number || 'Unknown';
                            const title = `Incident ${{number}}: ${{item.short_description || 'No title'}}`;
                            const sysId = item.sys_id || '';
                            const url = `{}/incident.do?sys_id=${{sysId}}`;
                            const description = item.description || '';
                            const resolution = item.close_notes || '';
                            const content = `Description: ${{description}}\\nResolution: ${{resolution}}`;
                            const excerpt = content.substring(0, 200);

                            results.push({{
                                title,
                                url,
                                excerpt,
                                content,
                                source: 'ServiceNow'
                            }});
                        }}
                    }}
                }}

                return {{ results }};
            }} catch (error) {{
                return {{ error: error.message }};
            }}
        }})();
        "#,
        instance_url.trim_end_matches('/'),
        urlencoding::encode(query),
        urlencoding::encode(query),
        instance_url.trim_end_matches('/'),
        instance_url.trim_end_matches('/'),
        urlencoding::encode(query),
        urlencoding::encode(query),
        instance_url.trim_end_matches('/')
    );

    let result: serde_json::Value = webview_window
        .eval(&search_script)
        .map_err(|e| format!("Failed to execute search: {}", e))?;

    if let Some(error) = result.get("error") {
        return Err(format!("Search error: {}", error));
    }

    if let Some(results_array) = result.get("results").and_then(|v| v.as_array()) {
        let mut results = Vec::new();
        for item in results_array {
            if let Ok(search_result) = serde_json::from_value::<SearchResult>(item.clone()) {
                results.push(search_result);
            }
        }
        Ok(results)
    } else {
        Ok(Vec::new())
    }
}

/// Search Azure DevOps from within the authenticated webview
async fn search_azuredevops_from_webview<R: tauri::Runtime>(
    webview_window: &WebviewWindow<R>,
    org_url: &str,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    // Azure DevOps search requires project parameter, which we don't have here
    // This would need to be passed in from the config
    // For now, return empty results
    tracing::warn!("Azure DevOps webview search not yet implemented");
    Ok(Vec::new())
}
