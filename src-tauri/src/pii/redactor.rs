use crate::pii::PiiSpan;
use sha2::{Digest, Sha256};

pub fn apply_redactions(text: &str, spans: &[PiiSpan]) -> String {
    if spans.is_empty() {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;

    for span in spans {
        if span.start >= last_end {
            result.push_str(&text[last_end..span.start]);
            result.push_str(&span.replacement);
            last_end = span.end;
        }
    }
    result.push_str(&text[last_end..]);
    result
}

pub fn hash_content(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pii::{PiiSpan, PiiType};

    #[test]
    fn test_apply_redactions() {
        let text = "Server 192.168.1.1 is down";
        let spans = vec![PiiSpan::new(
            PiiType::Ipv4,
            7,
            18,
            "192.168.1.1".to_string(),
        )];
        let redacted = apply_redactions(text, &spans);
        assert_eq!(redacted, "Server [IPv4] is down");
        assert!(!redacted.contains("192.168.1.1"));
    }

    #[test]
    fn test_empty_spans() {
        let text = "No PII here";
        let redacted = apply_redactions(text, &[]);
        assert_eq!(redacted, text);
    }
}
