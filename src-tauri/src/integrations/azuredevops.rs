use serde::{Deserialize, Serialize};

use super::{ConnectionResult, TicketResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureDevOpsConfig {
    pub organization_url: String,
    pub project: String,
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: i64,
    pub title: String,
    pub work_item_type: String,
    pub state: String,
    pub description: String,
}

fn escape_wiql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

/// Test connection to Azure DevOps by querying project info
pub async fn test_connection(config: &AzureDevOpsConfig) -> Result<ConnectionResult, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/_apis/projects/{}?api-version=7.0",
        config.organization_url.trim_end_matches('/'),
        config.project
    );

    let resp = client
        .get(&url)
        .bearer_auth(&config.access_token)
        .send()
        .await
        .map_err(|e| format!("Connection failed: {e}"))?;

    if resp.status().is_success() {
        Ok(ConnectionResult {
            success: true,
            message: "Successfully connected to Azure DevOps".to_string(),
        })
    } else {
        let status = resp.status();
        Ok(ConnectionResult {
            success: false,
            message: format!("Connection failed with status: {status}"),
        })
    }
}

/// Search for work items using WIQL query
pub async fn search_work_items(
    config: &AzureDevOpsConfig,
    query: &str,
) -> Result<Vec<WorkItem>, String> {
    let client = reqwest::Client::new();
    let wiql_url = format!(
        "{}/{}/_apis/wit/wiql?api-version=7.0",
        config.organization_url.trim_end_matches('/'),
        config.project
    );

    // Build WIQL query
    let escaped_query = escape_wiql_literal(query);
    let wiql = format!(
        "SELECT [System.Id], [System.Title], [System.WorkItemType], [System.State] FROM WorkItems WHERE [System.Title] CONTAINS '{escaped_query}' ORDER BY [System.CreatedDate] DESC"
    );

    let body = serde_json::json!({ "query": wiql });

    let resp = client
        .post(&wiql_url)
        .bearer_auth(&config.access_token)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("WIQL query failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "WIQL query failed: {} - {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let wiql_result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse WIQL response: {e}"))?;

    let work_item_refs = wiql_result["workItems"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|w| w["id"].as_i64())
        .collect::<Vec<_>>();

    if work_item_refs.is_empty() {
        return Ok(vec![]);
    }

    // Fetch full work item details
    let ids = work_item_refs
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let detail_url = format!(
        "{}/{}/_apis/wit/workitems?ids={ids}&api-version=7.0",
        config.organization_url.trim_end_matches('/'),
        config.project
    );

    let detail_resp = client
        .get(&detail_url)
        .bearer_auth(&config.access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch work item details: {e}"))?;

    if !detail_resp.status().is_success() {
        return Err(format!(
            "Failed to fetch work item details: {}",
            detail_resp.status()
        ));
    }

    let details: serde_json::Value = detail_resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse work item details: {e}"))?;

    let work_items = details["value"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|w| {
            Some(WorkItem {
                id: w["id"].as_i64()?,
                title: w["fields"]["System.Title"].as_str()?.to_string(),
                work_item_type: w["fields"]["System.WorkItemType"].as_str()?.to_string(),
                state: w["fields"]["System.State"].as_str()?.to_string(),
                description: w["fields"]["System.Description"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            })
        })
        .collect();

    Ok(work_items)
}

/// Create a new work item in Azure DevOps
pub async fn create_work_item(
    config: &AzureDevOpsConfig,
    title: &str,
    description: &str,
    work_item_type: &str,
    severity: &str,
) -> Result<TicketResult, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/{}/_apis/wit/workitems/${work_item_type}?api-version=7.0",
        config.organization_url.trim_end_matches('/'),
        config.project
    );

    let mut operations = vec![
        serde_json::json!({
            "op": "add",
            "path": "/fields/System.Title",
            "value": title
        }),
        serde_json::json!({
            "op": "add",
            "path": "/fields/System.Description",
            "value": description
        }),
    ];

    // Add severity/priority if provided
    if work_item_type == "Bug" && !severity.is_empty() {
        operations.push(serde_json::json!({
            "op": "add",
            "path": "/fields/Microsoft.VSTS.Common.Severity",
            "value": severity
        }));
    }

    let resp = client
        .post(&url)
        .bearer_auth(&config.access_token)
        .header("Content-Type", "application/json-patch+json")
        .json(&operations)
        .send()
        .await
        .map_err(|e| format!("Failed to create work item: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Failed to create work item: {} - {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    let work_item_id = result["id"].as_i64().unwrap_or(0);
    let work_item_url = format!(
        "{}/_workitems/edit/{work_item_id}",
        config.organization_url.trim_end_matches('/')
    );

    Ok(TicketResult {
        id: work_item_id.to_string(),
        ticket_number: format!("#{work_item_id}"),
        url: work_item_url,
    })
}

/// Get a work item by ID
pub async fn get_work_item(
    config: &AzureDevOpsConfig,
    work_item_id: i64,
) -> Result<WorkItem, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/{}/_apis/wit/workitems/{work_item_id}?api-version=7.0",
        config.organization_url.trim_end_matches('/'),
        config.project
    );

    let resp = client
        .get(&url)
        .bearer_auth(&config.access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to get work item: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Failed to get work item: {} - {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    Ok(WorkItem {
        id: result["id"]
            .as_i64()
            .ok_or_else(|| "Missing id".to_string())?,
        title: result["fields"]["System.Title"]
            .as_str()
            .ok_or_else(|| "Missing title".to_string())?
            .to_string(),
        work_item_type: result["fields"]["System.WorkItemType"]
            .as_str()
            .ok_or_else(|| "Missing work item type".to_string())?
            .to_string(),
        state: result["fields"]["System.State"]
            .as_str()
            .ok_or_else(|| "Missing state".to_string())?
            .to_string(),
        description: result["fields"]["System.Description"]
            .as_str()
            .unwrap_or("")
            .to_string(),
    })
}

/// Update an existing work item
pub async fn update_work_item(
    config: &AzureDevOpsConfig,
    work_item_id: i64,
    updates: serde_json::Value,
) -> Result<TicketResult, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/{}/_apis/wit/workitems/{work_item_id}?api-version=7.0",
        config.organization_url.trim_end_matches('/'),
        config.project
    );

    let resp = client
        .patch(&url)
        .bearer_auth(&config.access_token)
        .header("Content-Type", "application/json-patch+json")
        .json(&updates)
        .send()
        .await
        .map_err(|e| format!("Failed to update work item: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Failed to update work item: {} - {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    let updated_work_item_id = result["id"].as_i64().unwrap_or(work_item_id);
    let work_item_url = format!(
        "{}/_workitems/edit/{updated_work_item_id}",
        config.organization_url.trim_end_matches('/')
    );

    Ok(TicketResult {
        id: updated_work_item_id.to_string(),
        ticket_number: format!("#{updated_work_item_id}"),
        url: work_item_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_wiql_literal_escapes_single_quotes() {
        let escaped = escape_wiql_literal("can't deploy");
        assert_eq!(escaped, "can''t deploy");
    }

    #[tokio::test]
    async fn test_connection_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/_apis/projects/TestProject")
            .match_header("authorization", "Bearer test_token")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "api-version".into(),
                "7.0".into(),
            )]))
            .with_status(200)
            .with_body(r#"{"name":"TestProject","id":"abc123"}"#)
            .create_async()
            .await;

        let config = AzureDevOpsConfig {
            organization_url: server.url(),
            project: "TestProject".to_string(),
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
            .mock("GET", "/_apis/projects/TestProject")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "api-version".into(),
                "7.0".into(),
            )]))
            .with_status(401)
            .create_async()
            .await;

        let config = AzureDevOpsConfig {
            organization_url: server.url(),
            project: "TestProject".to_string(),
            access_token: "invalid_token".to_string(),
        };

        let result = test_connection(&config).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let conn = result.unwrap();
        assert!(!conn.success);
    }

    #[tokio::test]
    async fn test_search_work_items() {
        let mut server = mockito::Server::new_async().await;

        let wiql_mock = server
            .mock("POST", "/TestProject/_apis/wit/wiql")
            .match_header("authorization", "Bearer test_token")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "api-version".into(),
                "7.0".into(),
            )]))
            .with_status(200)
            .with_body(r#"{"workItems":[{"id":123}]}"#)
            .create_async()
            .await;

        let detail_mock = server
            .mock("GET", "/TestProject/_apis/wit/workitems")
            .match_header("authorization", "Bearer test_token")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("api-version".into(), "7.0".into()),
                mockito::Matcher::UrlEncoded("ids".into(), "123".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
                "value": [{
                    "id": 123,
                    "fields": {
                        "System.Title": "Bug: Login fails",
                        "System.WorkItemType": "Bug",
                        "System.State": "Active",
                        "System.Description": "Users cannot login"
                    }
                }]
            }"#,
            )
            .create_async()
            .await;

        let config = AzureDevOpsConfig {
            organization_url: server.url(),
            project: "TestProject".to_string(),
            access_token: "test_token".to_string(),
        };

        let result = search_work_items(&config, "login").await;
        wiql_mock.assert_async().await;
        detail_mock.assert_async().await;

        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, 123);
        assert_eq!(items[0].title, "Bug: Login fails");
    }

    #[tokio::test]
    async fn test_create_work_item() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/TestProject/_apis/wit/workitems/$Bug")
            .match_header("authorization", "Bearer test_token")
            .match_header("content-type", "application/json-patch+json")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "api-version".into(),
                "7.0".into(),
            )]))
            .with_status(200)
            .with_body(r#"{"id":456}"#)
            .create_async()
            .await;

        let config = AzureDevOpsConfig {
            organization_url: server.url(),
            project: "TestProject".to_string(),
            access_token: "test_token".to_string(),
        };

        let result = create_work_item(&config, "Test bug", "Description", "Bug", "3").await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let ticket = result.unwrap();
        assert_eq!(ticket.id, "456");
        assert_eq!(ticket.ticket_number, "#456");
        assert!(ticket.url.contains("456"));
    }

    #[tokio::test]
    async fn test_get_work_item() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/TestProject/_apis/wit/workitems/123")
            .match_header("authorization", "Bearer test_token")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "api-version".into(),
                "7.0".into(),
            )]))
            .with_status(200)
            .with_body(
                r#"{
                "id": 123,
                "fields": {
                    "System.Title": "Test item",
                    "System.WorkItemType": "Task",
                    "System.State": "Active",
                    "System.Description": "Test description"
                }
            }"#,
            )
            .create_async()
            .await;

        let config = AzureDevOpsConfig {
            organization_url: server.url(),
            project: "TestProject".to_string(),
            access_token: "test_token".to_string(),
        };

        let result = get_work_item(&config, 123).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let item = result.unwrap();
        assert_eq!(item.id, 123);
        assert_eq!(item.title, "Test item");
    }

    #[tokio::test]
    async fn test_update_work_item() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PATCH", "/TestProject/_apis/wit/workitems/123")
            .match_header("authorization", "Bearer test_token")
            .match_header("content-type", "application/json-patch+json")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "api-version".into(),
                "7.0".into(),
            )]))
            .with_status(200)
            .with_body(r#"{"id":123}"#)
            .create_async()
            .await;

        let config = AzureDevOpsConfig {
            organization_url: server.url(),
            project: "TestProject".to_string(),
            access_token: "test_token".to_string(),
        };

        let updates = serde_json::json!([
            {
                "op": "add",
                "path": "/fields/System.State",
                "value": "Resolved"
            }
        ]);

        let result = update_work_item(&config, 123, updates).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let ticket = result.unwrap();
        assert_eq!(ticket.id, "123");
        assert_eq!(ticket.ticket_number, "#123");
    }
}
