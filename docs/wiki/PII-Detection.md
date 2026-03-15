# PII Detection

## Overview

Before any text is sent to an AI provider, TFTSR scans it for personally identifiable information (PII). Users must review and approve each detected span before the redacted text is transmitted.

## Detection Flow

```
1. Upload log file
      ↓
2. detect_pii(log_file_id)
   → Scans content with 13 regex patterns
   → Resolves overlapping matches (longest wins)
   → Returns Vec<PiiSpan> with byte offsets + replacements
      ↓
3. User reviews spans in PiiDiffViewer (before/after diff)
   → Approves or rejects each span
      ↓
4. apply_redactions(log_file_id, approved_span_ids)
   → Rewrites text with replacements (iterates spans in REVERSE order to preserve offsets)
   → Records SHA-256 hash of redacted text in audit_log
      ↓
5. Redacted text safe to send to AI
```

## Detection Patterns (13 Types)

| Type | Replacement | Pattern notes |
|------|-------------|---------------|
| `UrlWithCredentials` | `[URL]` | `scheme://user:pass@host` |
| `BearerToken` | `[Bearer]` | Case-insensitive `bearer` keyword + token chars |
| `ApiKey` | `[ApiKey]` | `api_key=`, `apikey=`, `access_token=` + 16+ char value |
| `Password` | `[Password]` | `password=`, `passwd=`, `pwd=` + non-whitespace value |
| `Ssn` | `[SSN]` | `\b\d{3}-\d{2}-\d{4}\b` |
| `CreditCard` | `[CreditCard]` | Visa/MC/Amex Luhn-format numbers |
| `Email` | `[Email]` | RFC-compliant email addresses |
| `MacAddress` | `[MAC]` | `XX:XX:XX:XX:XX:XX` and `XX-XX-XX-XX-XX-XX` |
| `Ipv6` | `[IPv6]` | Full and compressed IPv6 addresses |
| `Ipv4` | `[IPv4]` | Standard dotted-quad notation |
| `PhoneNumber` | `[Phone]` | US and international phone formats |
| `Hostname` | _(patterns.rs)_ | Configurable hostname patterns |
| `UrlCredentials` | _(covered by UrlWithCredentials)_ | |

## Overlap Resolution

When two patterns match overlapping text, the **longer match wins**:

```rust
let mut filtered: Vec<PiiSpan> = Vec::new();
for span in sorted_by_start {
    if let Some(last) = filtered.last() {
        if span.start < last.end {
            // Overlap: keep the longer span
            if span.end - span.start > last.end - last.start {
                filtered.pop();
                filtered.push(span);
            }
            continue;
        }
    }
    filtered.push(span);
}
```

## PiiSpan Struct

```rust
pub struct PiiSpan {
    pub id: String,          // UUID v7
    pub pii_type: PiiType,
    pub start: usize,        // byte offset in original text
    pub end: usize,
    pub original_value: String,
    pub replacement: String, // e.g., "[IPv4]"
}
```

## Redaction Algorithm

Spans are applied in **reverse order** to preserve byte offsets:

```rust
let mut redacted = original.to_string();
for span in approved_spans.iter().rev() {  // reverse!
    redacted.replace_range(span.start..span.end, &span.replacement);
}
```

## Audit Logging

Every redaction and every AI send is logged:

```rust
write_audit_event(
    &conn,
    "ai_send",                        // action
    "issue",                          // entity_type
    &issue_id,                        // entity_id
    &json!({
        "log_file_ids": [...],
        "redacted_hash": sha256_hex,  // SHA-256 of redacted text
        "provider": provider_name,
    }).to_string(),
)?;
```

## Security Guarantees

- PII detection runs **locally** — original text never leaves the machine
- Only the redacted text is sent to AI providers
- The SHA-256 hash in the audit log allows integrity verification
- If redaction is skipped (no PII detected), the audit log still records the send
