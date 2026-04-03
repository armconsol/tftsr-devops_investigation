pub mod auth;
pub mod azuredevops;
pub mod callback_server;
pub mod confluence;
pub mod servicenow;

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
