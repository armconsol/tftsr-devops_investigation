// Copyright (c) 2025 Shaun Arman
// MIT License - see LICENSE file for details

use crate::pii::patterns::get_patterns;
use crate::pii::{PiiSpan, PiiType};
use regex::Regex;

pub struct PiiDetector {
    patterns: Vec<(PiiType, Regex)>,
}

impl PiiDetector {
    pub fn new() -> Self {
        PiiDetector {
            patterns: get_patterns(),
        }
    }

    pub fn detect(&self, text: &str) -> Vec<PiiSpan> {
        let mut spans: Vec<PiiSpan> = Vec::new();

        for (pii_type, regex) in &self.patterns {
            for mat in regex.find_iter(text) {
                spans.push(PiiSpan::new(
                    pii_type.clone(),
                    mat.start(),
                    mat.end(),
                    mat.as_str().to_string(),
                ));
            }
        }

        // Sort by start position
        spans.sort_by_key(|s| s.start);

        // Remove overlapping spans (keep longer one)
        let mut filtered: Vec<PiiSpan> = Vec::new();
        for span in spans {
            if let Some(last) = filtered.last() {
                if span.start < last.end {
                    // Overlap: keep the longer one
                    if span.end - span.start > last.end - last.start {
                        filtered.pop();
                        filtered.push(span);
                    }
                    continue;
                }
            }
            filtered.push(span);
        }

        filtered
    }
}

impl Default for PiiDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ipv4() {
        let detector = PiiDetector::new();
        let text = "Connected to 192.168.1.100 from 10.0.0.1";
        let spans = detector.detect(text);
        let ipv4_spans: Vec<_> = spans.iter().filter(|s| s.pii_type == "IPv4").collect();
        assert!(!ipv4_spans.is_empty());
        assert!(ipv4_spans.iter().any(|s| s.original == "192.168.1.100"));
    }

    #[test]
    fn test_detect_email() {
        let detector = PiiDetector::new();
        let text = "User admin@example.com logged in";
        let spans = detector.detect(text);
        assert!(spans.iter().any(|s| s.pii_type == "Email"));
    }

    #[test]
    fn test_detect_bearer_token() {
        let detector = PiiDetector::new();
        let text = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test";
        let spans = detector.detect(text);
        assert!(spans.iter().any(|s| s.pii_type == "Bearer"));
    }

    #[test]
    fn test_detect_password_keyword() {
        let detector = PiiDetector::new();
        // Full keyword forms
        assert!(detector
            .detect("password: hunter2")
            .iter()
            .any(|s| s.pii_type == "Password"));
        assert!(detector
            .detect("passwd=hunter2")
            .iter()
            .any(|s| s.pii_type == "Password"));
        assert!(detector
            .detect("pwd: hunter2")
            .iter()
            .any(|s| s.pii_type == "Password"));
    }

    #[test]
    fn test_detect_pass_abbreviation() {
        let detector = PiiDetector::new();
        // Abbreviated form used in credential files (was the failing case)
        let text = "user: alpha\npass: abc123!!";
        let spans = detector.detect(text);
        assert!(
            spans.iter().any(|s| s.pii_type == "Password"),
            "Expected Password span for 'pass: abc123!!' — got: {spans:?}"
        );
    }

    #[test]
    fn test_detect_secret_keyword() {
        let detector = PiiDetector::new();
        assert!(detector
            .detect("secret: mysecretvalue")
            .iter()
            .any(|s| s.pii_type == "Password"));
        assert!(detector
            .detect("passphrase: correct horse battery staple")
            .iter()
            .any(|s| s.pii_type == "Password"));
    }

    #[test]
    fn test_detect_password_natural_language() {
        let detector = PiiDetector::new();
        // Direct juxtaposition: "password <value>" (was the second failing case)
        let spans = detector.detect("Is the password password123 good");
        assert!(
            spans.iter().any(|s| s.pii_type == "Password"),
            "Expected Password span for natural-language 'password password123' — got: {spans:?}"
        );
        // "password is X"
        assert!(detector
            .detect("my password is hunter2")
            .iter()
            .any(|s| s.pii_type == "Password"));
        // Value must have digit or special — plain words should not trigger
        assert!(
            !detector
                .detect("password strength")
                .iter()
                .any(|s| s.pii_type == "Password"),
            "False positive: 'password strength' should not match"
        );
        assert!(
            !detector
                .detect("password policy")
                .iter()
                .any(|s| s.pii_type == "Password"),
            "False positive: 'password policy' should not match"
        );
    }

    #[test]
    fn test_password_no_false_positive_bypass() {
        let detector = PiiDetector::new();
        // "bypass" contains "pass" as a substring — must NOT match
        let spans = detector.detect("bypass: enabled");
        assert!(
            !spans.iter().any(|s| s.pii_type == "Password"),
            "False positive: 'bypass:' should not match Password pattern"
        );
    }

    #[test]
    fn test_no_overlap() {
        let detector = PiiDetector::new();
        let text = "IP: 192.168.1.1 user: test@test.com";
        let spans = detector.detect(text);
        // Verify no two spans overlap
        for i in 0..spans.len() {
            for j in (i + 1)..spans.len() {
                assert!(spans[i].end <= spans[j].start, "Spans overlap!");
            }
        }
    }
}
