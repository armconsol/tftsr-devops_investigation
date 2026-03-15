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
