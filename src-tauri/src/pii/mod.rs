// Copyright (c) 2025 Shaun Arman
// MIT License - see LICENSE file for details

pub mod detector;
pub mod patterns;
pub mod redactor;

pub use detector::*;
pub use patterns::*;
pub use redactor::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PiiType {
    Ipv4,
    Ipv6,
    Email,
    PhoneNumber,
    Ssn,
    CreditCard,
    MacAddress,
    Hostname,
    ApiKey,
    BearerToken,
    Password,
    UrlWithCreds,
}

impl std::fmt::Display for PiiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PiiType::Ipv4 => write!(f, "IPv4"),
            PiiType::Ipv6 => write!(f, "IPv6"),
            PiiType::Email => write!(f, "Email"),
            PiiType::PhoneNumber => write!(f, "Phone"),
            PiiType::Ssn => write!(f, "SSN"),
            PiiType::CreditCard => write!(f, "CreditCard"),
            PiiType::MacAddress => write!(f, "MAC"),
            PiiType::Hostname => write!(f, "Hostname"),
            PiiType::ApiKey => write!(f, "ApiKey"),
            PiiType::BearerToken => write!(f, "Bearer"),
            PiiType::Password => write!(f, "Password"),
            PiiType::UrlWithCreds => write!(f, "URL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiSpan {
    pub id: String,
    pub pii_type: String,
    pub start: usize,
    pub end: usize,
    pub original: String,
    pub replacement: String,
}

impl PiiSpan {
    pub fn new(pii_type: PiiType, start: usize, end: usize, original: String) -> Self {
        let replacement = format!("[{pii_type}]");
        PiiSpan {
            id: Uuid::now_v7().to_string(),
            pii_type: pii_type.to_string(),
            start,
            end,
            original,
            replacement,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiDetectionResult {
    pub log_file_id: String,
    pub spans: Vec<PiiSpan>,
    pub original_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactedLogFile {
    pub log_file_id: String,
    pub redacted_text: String,
    pub data_hash: String,
}
