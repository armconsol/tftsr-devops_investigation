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
    pub number: String,
    pub short_description: String,
    pub description: String,
    pub urgency: String,
    pub impact: String,
    pub state: String,
}

pub async fn test_connection(_config: &ServiceNowConfig) -> Result<ConnectionResult, String> {
    Err(
        "ServiceNow integration available in v0.2. Please update to the latest version."
            .to_string(),
    )
}

pub async fn create_incident(
    _config: &ServiceNowConfig,
    _short_description: &str,
    _description: &str,
    _urgency: &str,
    _impact: &str,
) -> Result<TicketResult, String> {
    Err(
        "ServiceNow integration available in v0.2. Please update to the latest version."
            .to_string(),
    )
}

pub async fn get_incident(
    _config: &ServiceNowConfig,
    _incident_number: &str,
) -> Result<Incident, String> {
    Err(
        "ServiceNow integration available in v0.2. Please update to the latest version."
            .to_string(),
    )
}

pub async fn update_incident(
    _config: &ServiceNowConfig,
    _incident_number: &str,
    _updates: serde_json::Value,
) -> Result<TicketResult, String> {
    Err(
        "ServiceNow integration available in v0.2. Please update to the latest version."
            .to_string(),
    )
}
