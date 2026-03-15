use serde::{Deserialize, Serialize};

use super::{ConnectionResult, TicketResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureDevOpsConfig {
    pub organization_url: String,
    pub project: String,
    pub pat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: i64,
    pub title: String,
    pub work_item_type: String,
    pub state: String,
    pub url: String,
}

pub async fn test_connection(_config: &AzureDevOpsConfig) -> Result<ConnectionResult, String> {
    Err(
        "Azure DevOps integration available in v0.2. Please update to the latest version."
            .to_string(),
    )
}

pub async fn create_work_item(
    _config: &AzureDevOpsConfig,
    _title: &str,
    _description: &str,
    _work_item_type: &str,
    _severity: &str,
) -> Result<TicketResult, String> {
    Err(
        "Azure DevOps integration available in v0.2. Please update to the latest version."
            .to_string(),
    )
}

pub async fn get_work_item(
    _config: &AzureDevOpsConfig,
    _work_item_id: i64,
) -> Result<WorkItem, String> {
    Err(
        "Azure DevOps integration available in v0.2. Please update to the latest version."
            .to_string(),
    )
}

pub async fn update_work_item(
    _config: &AzureDevOpsConfig,
    _work_item_id: i64,
    _updates: serde_json::Value,
) -> Result<TicketResult, String> {
    Err(
        "Azure DevOps integration available in v0.2. Please update to the latest version."
            .to_string(),
    )
}
