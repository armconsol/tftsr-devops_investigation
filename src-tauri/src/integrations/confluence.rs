use serde::{Deserialize, Serialize};

use super::{ConnectionResult, PublishResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfluenceConfig {
    pub base_url: String,
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub title: String,
    pub space_key: String,
    pub url: String,
}

/// Test connection to Confluence by fetching current user info
pub async fn test_connection(config: &ConfluenceConfig) -> Result<ConnectionResult, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/rest/api/user/current", config.base_url.trim_end_matches('/'));

    let resp = client
        .get(&url)
        .bearer_auth(&config.access_token)
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    if resp.status().is_success() {
        Ok(ConnectionResult {
            success: true,
            message: "Successfully connected to Confluence".to_string(),
        })
    } else {
        Ok(ConnectionResult {
            success: false,
            message: format!("Connection failed with status: {}", resp.status()),
        })
    }
}

/// List all spaces accessible with the current token
pub async fn list_spaces(config: &ConfluenceConfig) -> Result<Vec<Space>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/rest/api/space", config.base_url.trim_end_matches('/'));

    let resp = client
        .get(&url)
        .bearer_auth(&config.access_token)
        .query(&[("limit", "100")])
        .send()
        .await
        .map_err(|e| format!("Failed to list spaces: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Failed to list spaces: {} - {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let spaces = body["results"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|s| {
            Some(Space {
                key: s["key"].as_str()?.to_string(),
                name: s["name"].as_str()?.to_string(),
            })
        })
        .collect();

    Ok(spaces)
}

/// Search for pages by title or content
pub async fn search_pages(
    config: &ConfluenceConfig,
    query: &str,
    space_key: Option<&str>,
) -> Result<Vec<Page>, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/rest/api/content/search",
        config.base_url.trim_end_matches('/')
    );

    let mut cql = format!("text ~ \"{}\"", query);
    if let Some(space) = space_key {
        cql = format!("{} AND space = {}", cql, space);
    }

    let resp = client
        .get(&url)
        .bearer_auth(&config.access_token)
        .query(&[("cql", &cql), ("limit", &"50".to_string())])
        .send()
        .await
        .map_err(|e| format!("Search failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Search failed: {} - {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let pages = body["results"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|p| {
            let base_url = config.base_url.trim_end_matches('/');
            let page_id = p["id"].as_str()?;
            Some(Page {
                id: page_id.to_string(),
                title: p["title"].as_str()?.to_string(),
                space_key: p["space"]["key"].as_str()?.to_string(),
                url: format!("{}/pages/viewpage.action?pageId={}", base_url, page_id),
            })
        })
        .collect();

    Ok(pages)
}

/// Publish a new page to Confluence
pub async fn publish_page(
    config: &ConfluenceConfig,
    space_key: &str,
    title: &str,
    content_html: &str,
    parent_page_id: Option<&str>,
) -> Result<PublishResult, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/rest/api/content", config.base_url.trim_end_matches('/'));

    let mut body = serde_json::json!({
        "type": "page",
        "title": title,
        "space": { "key": space_key },
        "body": {
            "storage": {
                "value": content_html,
                "representation": "storage"
            }
        }
    });

    if let Some(parent_id) = parent_page_id {
        body["ancestors"] = serde_json::json!([{ "id": parent_id }]);
    }

    let resp = client
        .post(&url)
        .bearer_auth(&config.access_token)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to publish page: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Failed to publish page: {} - {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let page_id = result["id"].as_str().unwrap_or("");
    let page_url = format!(
        "{}/pages/viewpage.action?pageId={}",
        config.base_url.trim_end_matches('/'),
        page_id
    );

    Ok(PublishResult {
        id: page_id.to_string(),
        url: page_url,
    })
}

/// Update an existing page in Confluence
pub async fn update_page(
    config: &ConfluenceConfig,
    page_id: &str,
    title: &str,
    content_html: &str,
    version: i32,
) -> Result<PublishResult, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/rest/api/content/{}",
        config.base_url.trim_end_matches('/'),
        page_id
    );

    let body = serde_json::json!({
        "id": page_id,
        "type": "page",
        "title": title,
        "version": { "number": version + 1 },
        "body": {
            "storage": {
                "value": content_html,
                "representation": "storage"
            }
        }
    });

    let resp = client
        .put(&url)
        .bearer_auth(&config.access_token)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to update page: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Failed to update page: {} - {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let updated_page_id = result["id"].as_str().unwrap_or(page_id);
    let page_url = format!(
        "{}/pages/viewpage.action?pageId={}",
        config.base_url.trim_end_matches('/'),
        updated_page_id
    );

    Ok(PublishResult {
        id: updated_page_id.to_string(),
        url: page_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/api/user/current")
            .match_header("authorization", "Bearer test_token")
            .with_status(200)
            .with_body(r#"{"username":"test_user"}"#)
            .create_async()
            .await;

        let config = ConfluenceConfig {
            base_url: server.url(),
            access_token: "test_token".to_string(),
        };

        let result = test_connection(&config).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let conn = result.unwrap();
        assert!(conn.success);
        assert!(conn.message.contains("Successfully connected"));
    }

    #[tokio::test]
    async fn test_connection_failure() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/api/user/current")
            .with_status(401)
            .create_async()
            .await;

        let config = ConfluenceConfig {
            base_url: server.url(),
            access_token: "invalid_token".to_string(),
        };

        let result = test_connection(&config).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let conn = result.unwrap();
        assert!(!conn.success);
    }

    #[tokio::test]
    async fn test_list_spaces() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/api/space")
            .match_header("authorization", "Bearer test_token")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("limit".into(), "100".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
                "results": [
                    {"key": "DEV", "name": "Development"},
                    {"key": "OPS", "name": "Operations"}
                ]
            }"#,
            )
            .create_async()
            .await;

        let config = ConfluenceConfig {
            base_url: server.url(),
            access_token: "test_token".to_string(),
        };

        let result = list_spaces(&config).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let spaces = result.unwrap();
        assert_eq!(spaces.len(), 2);
        assert_eq!(spaces[0].key, "DEV");
        assert_eq!(spaces[1].name, "Operations");
    }

    #[tokio::test]
    async fn test_search_pages() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/api/content/search")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("cql".into(), "text ~ \"kubernetes\"".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
                "results": [
                    {
                        "id": "123",
                        "title": "Kubernetes Guide",
                        "space": {"key": "DEV"}
                    }
                ]
            }"#,
            )
            .create_async()
            .await;

        let config = ConfluenceConfig {
            base_url: server.url(),
            access_token: "test_token".to_string(),
        };

        let result = search_pages(&config, "kubernetes", None).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let pages = result.unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].title, "Kubernetes Guide");
        assert_eq!(pages[0].space_key, "DEV");
    }

    #[tokio::test]
    async fn test_publish_page() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/rest/api/content")
            .match_header("authorization", "Bearer test_token")
            .with_status(200)
            .with_body(r#"{"id":"456","title":"New Page"}"#)
            .create_async()
            .await;

        let config = ConfluenceConfig {
            base_url: server.url(),
            access_token: "test_token".to_string(),
        };

        let result = publish_page(&config, "DEV", "New Page", "<p>Content</p>", None).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let publish = result.unwrap();
        assert_eq!(publish.id, "456");
        assert!(publish.url.contains("pageId=456"));
    }

    #[tokio::test]
    async fn test_update_page() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PUT", "/rest/api/content/789")
            .match_header("authorization", "Bearer test_token")
            .with_status(200)
            .with_body(r#"{"id":"789","title":"Updated Page"}"#)
            .create_async()
            .await;

        let config = ConfluenceConfig {
            base_url: server.url(),
            access_token: "test_token".to_string(),
        };

        let result = update_page(&config, "789", "Updated Page", "<p>New content</p>", 1).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let publish = result.unwrap();
        assert_eq!(publish.id, "789");
    }
}
