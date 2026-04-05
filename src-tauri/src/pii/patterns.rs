use crate::pii::PiiType;
use regex::Regex;

/// Returns a vector of (PiiType, compiled Regex) pairs for all supported PII patterns.
pub fn get_patterns() -> Vec<(PiiType, Regex)> {
    vec![
        // URL with credentials (check before email to avoid partial matches)
        (
            PiiType::UrlWithCreds,
            Regex::new(r"[a-z][a-z0-9+\-.]*://[^:@/\s]+:[^@/\s]+@").unwrap(),
        ),
        // Bearer token
        (
            PiiType::BearerToken,
            Regex::new(r"(?i)bearer\s+[A-Za-z0-9\-._~+/]+=*").unwrap(),
        ),
        // API key
        (
            PiiType::ApiKey,
            Regex::new(
                r"(?i)(?:api[_\-]?key|apikey|access[_\-]?token)\s*[=:]\s*[A-Za-z0-9\-._~+/]{16,}",
            )
            .unwrap(),
        ),
        // Password
        (
            PiiType::Password,
            Regex::new(r"(?i)(?:password|passwd|pwd)\s*[=:]\s*\S+").unwrap(),
        ),
        // SSN (check before phone to avoid partial matches)
        (
            PiiType::Ssn,
            Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(),
        ),
        // Credit card
        (
            PiiType::CreditCard,
            Regex::new(
                r"\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|6(?:011|5[0-9]{2})[0-9]{12}|3(?:0[0-5]|[68][0-9])[0-9]{11}|35(?:2[89]|[3-8][0-9])[0-9]{12})\b",
            )
            .unwrap(),
        ),
        // Email
        (
            PiiType::Email,
            Regex::new(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b").unwrap(),
        ),
        // MAC address
        (
            PiiType::MacAddress,
            Regex::new(r"\b(?:[0-9A-Fa-f]{2}[:\-]){5}[0-9A-Fa-f]{2}\b").unwrap(),
        ),
        // IPv6 (check before IPv4 since IPv6 can contain IPv4-like segments)
        (
            PiiType::Ipv6,
            Regex::new(
                r"(?i)\b(?:[0-9a-f]{1,4}:){7}[0-9a-f]{1,4}\b|(?i)\b(?:[0-9a-f]{1,4}:){1,7}:\b|(?i)\b(?:[0-9a-f]{1,4}:){1,6}:[0-9a-f]{1,4}\b|(?i)\b::(?:[0-9a-f]{1,4}:){0,5}[0-9a-f]{1,4}\b|(?i)\b[0-9a-f]{1,4}::(?:[0-9a-f]{1,4}:){0,4}[0-9a-f]{1,4}\b",
            )
            .unwrap(),
        ),
        // IPv4
        (
            PiiType::Ipv4,
            Regex::new(
                r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b",
            )
            .unwrap(),
        ),
        // Phone number
        (
            PiiType::PhoneNumber,
            Regex::new(r"\b(?:\+?1[-.\s]?)?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}\b")
                .unwrap(),
        ),
        // Hostname / FQDN
        (
            PiiType::Hostname,
            Regex::new(
                r"\b(?:[A-Za-z0-9](?:[A-Za-z0-9\-]{0,61}[A-Za-z0-9])?\.)+[A-Za-z]{2,63}\b",
            )
            .unwrap(),
        ),
    ]
}
