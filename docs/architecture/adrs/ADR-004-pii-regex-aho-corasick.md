# ADR-004: Regex + Aho-Corasick for PII Detection

**Status**: Accepted
**Date**: 2025-Q3
**Deciders**: sarman

---

## Context

Log files submitted for AI analysis may contain sensitive data: IP addresses, emails, bearer tokens, passwords, SSNs, credit card numbers, MAC addresses, phone numbers, and API keys. This data must be detected and redacted before any content leaves the machine via an AI API call.

Requirements:
- Fast scanning of files up to 50MB
- Multiple pattern types with different regex complexity
- Non-overlapping spans (longest match wins on overlap)
- User-controlled toggle per pattern type
- Byte-offset tracking for accurate replacement

---

## Decision

Use **Rust `regex` crate** for per-pattern matching combined with **`aho-corasick`** for multi-pattern string searching. Detection runs entirely in the Rust backend on the raw log content.

---

## Rationale

**Alternatives considered:**

| Option | Pros | Cons |
|--------|------|------|
| **regex + aho-corasick** (chosen) | Fast, Rust-native, no external deps, byte-offset accurate | Regex patterns need careful tuning; false positives possible |
| ML-based NER (spaCy, Presidio) | Higher recall for contextual PII | Requires Python runtime, large model files, not offline-friendly |
| Simple string matching | Extremely fast | Too many false negatives on varied formats |
| WASM-based detection | Runs in browser | Slower; log content in JS memory before Rust sees it |

**Implementation approach:**

1. **12 regex patterns** compiled once at startup via `lazy_static!`
2. Each pattern returns `(start, end, replacement)` tuples
3. All spans from all patterns collected into a flat `Vec<PiiSpan>`
4. Spans sorted by `start` offset
5. **Overlap resolution**: iterate through sorted spans, skip any span whose start is before the current end (greedy, longest match)
6. Spans stored in DB with UUID — referenced by `approved` flag when user confirms redaction
7. Redaction applies spans in **reverse order** to preserve byte offsets

**Why aho-corasick for some patterns:**
Literal string searches (e.g., `password=`, `api_key=`, `bearer `) are faster with Aho-Corasick multi-pattern matching than running individual regexes. The regex then validates the captured value portion.

---

## Patterns

| Pattern ID | Type | Example Match |
|------------|------|---------------|
| `url_credentials` | URL with embedded credentials | `https://user:pass@host` |
| `bearer_token` | Authorization headers | `Bearer eyJhbGc...` |
| `api_key` | API key assignments | `api_key=sk-abc123...` |
| `password` | Password assignments | `password=secret123` |
| `ssn` | Social Security Numbers | `123-45-6789` |
| `credit_card` | Credit card numbers | `4111 1111 1111 1111` |
| `email` | Email addresses | `user@example.com` |
| `mac_address` | MAC addresses | `AA:BB:CC:DD:EE:FF` |
| `ipv6` | IPv6 addresses | `2001:db8::1` |
| `ipv4` | IPv4 addresses | `192.168.1.1` |
| `phone` | Phone numbers | `+1 (555) 123-4567` |
| `hostname` | FQDNs | `db-prod.internal.example.com` |

---

## Consequences

**Positive:**
- No runtime dependencies — detection works fully offline
- 50MB file scanned in <500ms on modern hardware
- Patterns independently togglable via `pii_enabled_patterns` in settings
- Byte-accurate offsets enable precise redaction without re-parsing

**Negative:**
- Regex-based detection has false positives (e.g., version strings matching IPv4 patterns)
- User must review and approve — not fully automatic (mitigated by UX design)
- Pattern maintenance required as new credential formats emerge
- No contextual understanding (a password in a comment vs an active credential look identical)

**User safeguard:**
All redactions require user approval via `PiiDiffViewer` before the redacted log is written. The original is never sent to AI.
