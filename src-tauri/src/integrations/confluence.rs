use serde::{Deserialize, Serialize};

use super::{ConnectionResult, PublishResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfluenceConfig {
    pub base_url: String,
    pub username: String,
    pub api_token: String,
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

pub async fn test_connection(_config: &ConfluenceConfig) -> Result<ConnectionResult, String> {
    Err(
        "Confluence integration available in v0.2. Please update to the latest version."
            .to_string(),
    )
}

pub async fn list_spaces(_config: &ConfluenceConfig) -> Result<Vec<Space>, String> {
    Err(
        "Confluence integration available in v0.2. Please update to the latest version."
            .to_string(),
    )
}

pub async fn publish_page(
    _config: &ConfluenceConfig,
    _space_key: &str,
    _title: &str,
    _content_html: &str,
    _parent_page_id: Option<&str>,
) -> Result<PublishResult, String> {
    Err(
        "Confluence integration available in v0.2. Please update to the latest version."
            .to_string(),
    )
}

pub async fn update_page(
    _config: &ConfluenceConfig,
    _page_id: &str,
    _title: &str,
    _content_html: &str,
    _version: i32,
) -> Result<PublishResult, String> {
    Err(
        "Confluence integration available in v0.2. Please update to the latest version."
            .to_string(),
    )
}
