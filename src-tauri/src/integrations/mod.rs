pub mod auth;
pub mod azuredevops;
pub mod azuredevops_search;
pub mod callback_server;
pub mod confluence;
pub mod confluence_search;
pub mod servicenow;
pub mod servicenow_search;
pub mod webview_auth;
pub mod webview_fetch;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    pub url: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketResult {
    pub id: String,
    pub ticket_number: String,
    pub url: String,
}

/// Authentication method for integration services
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum AuthMethod {
    #[serde(rename = "oauth2")]
    OAuth2 {
        access_token: String,
        expires_at: Option<i64>,
    },
    #[serde(rename = "cookies")]
    Cookies { cookies: Vec<webview_auth::Cookie> },
    #[serde(rename = "token")]
    Token {
        token: String,
        token_type: String, // "Bearer", "Basic", etc.
    },
}
