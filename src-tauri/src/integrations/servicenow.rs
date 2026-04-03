use serde::{Deserialize, Serialize};

use super::{ConnectionResult, TicketResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNowConfig {
    pub instance_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub sys_id: String,
    pub number: String,
    pub short_description: String,
    pub description: String,
    pub urgency: String,
    pub impact: String,
    pub state: String,
}

/// Test connection to ServiceNow by querying a single incident
pub async fn test_connection(config: &ServiceNowConfig) -> Result<ConnectionResult, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/api/now/table/incident",
        config.instance_url.trim_end_matches('/')
    );

    let resp = client
        .get(&url)
        .basic_auth(&config.username, Some(&config.password))
        .query(&[("sysparm_limit", "1")])
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    if resp.status().is_success() {
        Ok(ConnectionResult {
            success: true,
            message: "Successfully connected to ServiceNow".to_string(),
        })
    } else {
        Ok(ConnectionResult {
            success: false,
            message: format!("Connection failed with status: {}", resp.status()),
        })
    }
}

/// Search for incidents by description or number
pub async fn search_incidents(
    config: &ServiceNowConfig,
    query: &str,
) -> Result<Vec<Incident>, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/api/now/table/incident",
        config.instance_url.trim_end_matches('/')
    );

    let sysparm_query = format!("short_descriptionLIKE{}", query);

    let resp = client
        .get(&url)
        .basic_auth(&config.username, Some(&config.password))
        .query(&[("sysparm_query", &sysparm_query), ("sysparm_limit", &"10".to_string())])
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

    let incidents = body["result"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|i| {
            Some(Incident {
                sys_id: i["sys_id"].as_str()?.to_string(),
                number: i["number"].as_str()?.to_string(),
                short_description: i["short_description"].as_str()?.to_string(),
                description: i["description"].as_str().unwrap_or("").to_string(),
                urgency: i["urgency"].as_str().unwrap_or("3").to_string(),
                impact: i["impact"].as_str().unwrap_or("3").to_string(),
                state: i["state"].as_str().unwrap_or("1").to_string(),
            })
        })
        .collect();

    Ok(incidents)
}

/// Create a new incident in ServiceNow
pub async fn create_incident(
    config: &ServiceNowConfig,
    short_description: &str,
    description: &str,
    urgency: &str,
    impact: &str,
) -> Result<TicketResult, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/api/now/table/incident",
        config.instance_url.trim_end_matches('/')
    );

    let body = serde_json::json!({
        "short_description": short_description,
        "description": description,
        "urgency": urgency,
        "impact": impact,
    });

    let resp = client
        .post(&url)
        .basic_auth(&config.username, Some(&config.password))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to create incident: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Failed to create incident: {} - {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let incident_number = result["result"]["number"].as_str().unwrap_or("");
    let sys_id = result["result"]["sys_id"].as_str().unwrap_or("");
    let incident_url = format!(
        "{}/nav_to.do?uri=incident.do?sys_id={}",
        config.instance_url.trim_end_matches('/'),
        sys_id
    );

    Ok(TicketResult {
        id: sys_id.to_string(),
        ticket_number: incident_number.to_string(),
        url: incident_url,
    })
}

/// Get an incident by sys_id or number
pub async fn get_incident(
    config: &ServiceNowConfig,
    incident_id: &str,
) -> Result<Incident, String> {
    let client = reqwest::Client::new();

    // Determine if incident_id is a sys_id or incident number
    let (url, use_query) = if incident_id.starts_with("INC") {
        // It's an incident number, use query parameter
        (
            format!(
                "{}/api/now/table/incident",
                config.instance_url.trim_end_matches('/')
            ),
            true,
        )
    } else {
        // It's a sys_id, use direct path
        (
            format!(
                "{}/api/now/table/incident/{}",
                config.instance_url.trim_end_matches('/'),
                incident_id
            ),
            false,
        )
    };

    let mut request = client
        .get(&url)
        .basic_auth(&config.username, Some(&config.password));

    if use_query {
        request = request.query(&[("sysparm_query", &format!("number={}", incident_id))]);
    }

    let resp = request
        .send()
        .await
        .map_err(|e| format!("Failed to get incident: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Failed to get incident: {} - {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let incident_data = if use_query {
        // Query response has "result" array
        body["result"]
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| "Incident not found".to_string())?
    } else {
        // Direct sys_id response has "result" object
        &body["result"]
    };

    Ok(Incident {
        sys_id: incident_data["sys_id"]
            .as_str()
            .ok_or_else(|| "Missing sys_id".to_string())?
            .to_string(),
        number: incident_data["number"]
            .as_str()
            .ok_or_else(|| "Missing number".to_string())?
            .to_string(),
        short_description: incident_data["short_description"]
            .as_str()
            .ok_or_else(|| "Missing short_description".to_string())?
            .to_string(),
        description: incident_data["description"].as_str().unwrap_or("").to_string(),
        urgency: incident_data["urgency"].as_str().unwrap_or("3").to_string(),
        impact: incident_data["impact"].as_str().unwrap_or("3").to_string(),
        state: incident_data["state"].as_str().unwrap_or("1").to_string(),
    })
}

/// Update an existing incident
pub async fn update_incident(
    config: &ServiceNowConfig,
    sys_id: &str,
    updates: serde_json::Value,
) -> Result<TicketResult, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/api/now/table/incident/{}",
        config.instance_url.trim_end_matches('/'),
        sys_id
    );

    let resp = client
        .patch(&url)
        .basic_auth(&config.username, Some(&config.password))
        .header("Content-Type", "application/json")
        .json(&updates)
        .send()
        .await
        .map_err(|e| format!("Failed to update incident: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Failed to update incident: {} - {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let incident_number = result["result"]["number"].as_str().unwrap_or("");
    let updated_sys_id = result["result"]["sys_id"].as_str().unwrap_or(sys_id);
    let incident_url = format!(
        "{}/nav_to.do?uri=incident.do?sys_id={}",
        config.instance_url.trim_end_matches('/'),
        updated_sys_id
    );

    Ok(TicketResult {
        id: updated_sys_id.to_string(),
        ticket_number: incident_number.to_string(),
        url: incident_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/now/table/incident")
            .match_header("authorization", mockito::Matcher::Regex("Basic .+".into()))
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("sysparm_limit".into(), "1".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"result":[]}"#)
            .create_async()
            .await;

        let config = ServiceNowConfig {
            instance_url: server.url(),
            username: "admin".to_string(),
            password: "password".to_string(),
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
            .mock("GET", "/api/now/table/incident")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("sysparm_limit".into(), "1".into()),
            ]))
            .with_status(401)
            .create_async()
            .await;

        let config = ServiceNowConfig {
            instance_url: server.url(),
            username: "admin".to_string(),
            password: "wrong_password".to_string(),
        };

        let result = test_connection(&config).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let conn = result.unwrap();
        assert!(!conn.success);
    }

    #[tokio::test]
    async fn test_search_incidents() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/now/table/incident")
            .match_header("authorization", mockito::Matcher::Regex("Basic .+".into()))
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("sysparm_query".into(), "short_descriptionLIKElogin".into()),
                mockito::Matcher::UrlEncoded("sysparm_limit".into(), "10".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
                "result": [
                    {
                        "sys_id": "abc123",
                        "number": "INC0010001",
                        "short_description": "Login issue",
                        "description": "Users cannot login",
                        "urgency": "2",
                        "impact": "2",
                        "state": "2"
                    }
                ]
            }"#,
            )
            .create_async()
            .await;

        let config = ServiceNowConfig {
            instance_url: server.url(),
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        let result = search_incidents(&config, "login").await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let incidents = result.unwrap();
        assert_eq!(incidents.len(), 1);
        assert_eq!(incidents[0].number, "INC0010001");
        assert_eq!(incidents[0].short_description, "Login issue");
    }

    #[tokio::test]
    async fn test_create_incident() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/now/table/incident")
            .match_header("authorization", mockito::Matcher::Regex("Basic .+".into()))
            .match_header("content-type", "application/json")
            .with_status(201)
            .with_body(
                r#"{
                "result": {
                    "sys_id": "def456",
                    "number": "INC0010002"
                }
            }"#,
            )
            .create_async()
            .await;

        let config = ServiceNowConfig {
            instance_url: server.url(),
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        let result = create_incident(&config, "Test issue", "Description", "3", "3").await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let ticket = result.unwrap();
        assert_eq!(ticket.ticket_number, "INC0010002");
        assert_eq!(ticket.id, "def456");
        assert!(ticket.url.contains("def456"));
    }

    #[tokio::test]
    async fn test_get_incident_by_sys_id() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/now/table/incident/abc123")
            .match_header("authorization", mockito::Matcher::Regex("Basic .+".into()))
            .with_status(200)
            .with_body(
                r#"{
                "result": {
                    "sys_id": "abc123",
                    "number": "INC0010001",
                    "short_description": "Login issue",
                    "description": "Users cannot login",
                    "urgency": "2",
                    "impact": "2",
                    "state": "2"
                }
            }"#,
            )
            .create_async()
            .await;

        let config = ServiceNowConfig {
            instance_url: server.url(),
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        let result = get_incident(&config, "abc123").await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let incident = result.unwrap();
        assert_eq!(incident.sys_id, "abc123");
        assert_eq!(incident.number, "INC0010001");
    }

    #[tokio::test]
    async fn test_get_incident_by_number() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/now/table/incident")
            .match_header("authorization", mockito::Matcher::Regex("Basic .+".into()))
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("sysparm_query".into(), "number=INC0010001".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
                "result": [{
                    "sys_id": "abc123",
                    "number": "INC0010001",
                    "short_description": "Login issue",
                    "description": "Users cannot login",
                    "urgency": "2",
                    "impact": "2",
                    "state": "2"
                }]
            }"#,
            )
            .create_async()
            .await;

        let config = ServiceNowConfig {
            instance_url: server.url(),
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        let result = get_incident(&config, "INC0010001").await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let incident = result.unwrap();
        assert_eq!(incident.sys_id, "abc123");
        assert_eq!(incident.number, "INC0010001");
    }

    #[tokio::test]
    async fn test_update_incident() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PATCH", "/api/now/table/incident/abc123")
            .match_header("authorization", mockito::Matcher::Regex("Basic .+".into()))
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_body(
                r#"{
                "result": {
                    "sys_id": "abc123",
                    "number": "INC0010001"
                }
            }"#,
            )
            .create_async()
            .await;

        let config = ServiceNowConfig {
            instance_url: server.url(),
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        let updates = serde_json::json!({
            "state": "6",
            "close_notes": "Issue resolved"
        });

        let result = update_incident(&config, "abc123", updates).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let ticket = result.unwrap();
        assert_eq!(ticket.id, "abc123");
        assert_eq!(ticket.ticket_number, "INC0010001");
    }
}
