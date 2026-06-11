use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Proxmox VE/PBS API client
/// Implements authentication and request handling for Proxmox APIs
pub struct ProxmoxClient {
    base_url: String,
    port: u16,
    username: String,
    api_token: Option<String>,
    client: Client,
}

/// Authentication response from Proxmox
#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub ticket: String,
    pub username: String,
    pub expire: u64,
    pub cap: String,
}

/// API token for authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiToken {
    pub token_id: String,
    pub name: String,
    pub expire: u64,
    pub permissions: Vec<String>,
}

impl ProxmoxClient {
    /// Create a new Proxmox client
    pub fn new(base_url: &str, port: u16, username: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            port,
            username: username.to_string(),
            api_token: None,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Authenticate with root username and password
    /// Returns the API ticket for subsequent requests
    pub async fn authenticate(&self, password: &str) -> Result<String> {
        let url = format!("{}/api2/json/access/ticket", self.base_url);

        let params = vec![
            ("username", self.username.as_str()),
            ("password", password),
        ];

        let response = self
            .client
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(|e| anyhow!("Authentication request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Authentication failed with status {}: {}",
                status,
                text
            ));
        }

        let auth: AuthResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse authentication response: {}", e))?;

        Ok(auth.ticket)
    }

    /// Authenticate with API token
    pub fn authenticate_with_token(&mut self, token: &str) {
        self.api_token = Some(token.to_string());
    }

    /// Get the full API URL for a given path
    fn get_api_url(&self, path: &str) -> String {
        format!(
            "{}/api2/json/{}",
            self.base_url,
            path.trim_start_matches('/')
        )
    }

    /// Build request headers with authentication
    fn build_headers(&self, ticket: Option<&str>) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        if let Some(token) = &self.api_token {
            // API token format: user@realm!tokenid=tokenvalue
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("PVEAPIAuth {}", token)
                    .parse()
                    .expect("Invalid auth header"),
            );
        } else if let Some(ticket) = ticket {
            // Cookie-based authentication
            headers.insert(
                "Cookie",
                format!("PVEAuthCookie={}", ticket)
                    .parse()
                    .expect("Invalid cookie header"),
            );
        }

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded"
                .parse()
                .expect("Invalid content type"),
        );

        headers
    }

    /// GET request to Proxmox API
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        ticket: Option<&str>,
    ) -> Result<T> {
        let url = self.get_api_url(path);
        let headers = self.build_headers(ticket);

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| anyhow!("GET request failed: {}", e))?;

        self.handle_response(response).await
    }

    /// POST request to Proxmox API
    pub async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        ticket: Option<&str>,
    ) -> Result<T> {
        let url = self.get_api_url(path);
        let headers = self.build_headers(ticket);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .map_err(|e| anyhow!("POST request failed: {}", e))?;

        self.handle_response(response).await
    }

    /// PUT request to Proxmox API
    pub async fn put<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        ticket: Option<&str>,
    ) -> Result<T> {
        let url = self.get_api_url(path);
        let headers = self.build_headers(ticket);

        let response = self
            .client
            .put(&url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .map_err(|e| anyhow!("PUT request failed: {}", e))?;

        self.handle_response(response).await
    }

    /// DELETE request to Proxmox API
    pub async fn delete<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        ticket: Option<&str>,
    ) -> Result<T> {
        let url = self.get_api_url(path);
        let headers = self.build_headers(ticket);

        let response = self
            .client
            .delete(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| anyhow!("DELETE request failed: {}", e))?;

        self.handle_response(response).await
    }

    /// Handle API response
    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "API request failed with status {}: {}",
                status,
                text
            ));
        }

        let data: HashMap<String, serde_json::Value> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse API response: {}", e))?;

        // Proxmox API wraps data in "data" field
        data.get("data")
            .ok_or_else(|| anyhow!("Response missing 'data' field"))
            .and_then(|d| {
                serde_json::from_value(d.clone())
                    .map_err(|e| anyhow!("Failed to deserialize data: {}", e))
            })
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the username
    pub fn username(&self) -> &str {
        &self.username
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxmox_client_new() {
        let client = ProxmoxClient::new("https://pve.example.com", 8006, "root@pam");
        assert_eq!(client.base_url(), "https://pve.example.com");
        assert_eq!(client.port(), 8006);
        assert_eq!(client.username(), "root@pam");
    }

    #[test]
    fn test_proxmox_client_with_trailing_slash() {
        let client = ProxmoxClient::new("https://pve.example.com/", 8006, "root@pam");
        assert_eq!(client.base_url(), "https://pve.example.com");
    }

    #[test]
    fn test_get_api_url() {
        let client = ProxmoxClient::new("https://pve.example.com", 8006, "root@pam");
        assert_eq!(
            client.get_api_url("cluster/resources"),
            "https://pve.example.com/api2/json/cluster/resources"
        );
        assert_eq!(
            client.get_api_url("/cluster/resources"),
            "https://pve.example.com/api2/json/cluster/resources"
        );
    }
}
