# Security Model

## Threat Model Summary

TFTSR handles sensitive IT incident data including log files that may contain credentials, PII, and internal infrastructure details. The security model addresses:

1. **Data at rest** — Database encryption
2. **Data in transit** — PII redaction before AI send, TLS for all outbound requests
3. **Secret storage** — API keys in Stronghold vault
4. **Audit trail** — Complete log of every external data transmission
5. **Least privilege** — Minimal Tauri capabilities

---

## Database Encryption (SQLCipher AES-256)

Production builds use SQLCipher:
- **Cipher:** AES-256-CBC
- **KDF:** PBKDF2-HMAC-SHA512, 256,000 iterations
- **HMAC:** HMAC-SHA512
- **Page size:** 4096 bytes
- **Key source:** `TFTSR_DB_KEY` environment variable

Debug builds use plain SQLite (no encryption) for developer convenience.

> ⚠️ **Never** use the default key (`dev-key-change-in-prod`) in a production environment.

---

## API Key Storage (Stronghold)

AI provider API keys are stored in `tauri-plugin-stronghold` — an encrypted vault backed by the [IOTA Stronghold](https://github.com/iotaledger/stronghold.rs) library.

The vault is initialized with a password-derived key using Argon2. API keys are never written to disk in plaintext or to the SQLite database.

---

## PII Redaction

**Mandatory path:** No text can be sent to an AI provider without going through the PII detection and user-approval flow.

```
log file → detect_pii() → user approves spans → apply_redactions() → AI provider
```

- Original text **never leaves the machine**
- Only the redacted version is transmitted
- The SHA-256 hash of the redacted text is recorded in the audit log for integrity verification
- See [PII Detection](PII-Detection) for the full list of detected patterns

---

## Audit Log

Every external data transmission is recorded:

```rust
write_audit_event(
    &conn,
    action,       // "ai_send", "publish_to_confluence", etc.
    entity_type,  // "issue", "document"
    entity_id,    // UUID of the related record
    details,      // JSON: provider, model, hashes, log_file_ids
)?;
```

The audit log is stored in the encrypted SQLite database. It cannot be deleted through the UI.

**Audit entry fields:**
- `action` — what was done
- `entity_type` — type of record involved
- `entity_id` — UUID of that record
- `user_id` — always `"local"` (single-user app)
- `details` — JSON blob with hashes and metadata
- `timestamp` — UTC datetime

---

## Tauri Capabilities (Least Privilege)

Defined in `src-tauri/capabilities/default.json`:

| Plugin | Permissions granted |
|--------|-------------------|
| `dialog` | `allow-open`, `allow-save` |
| `fs` | `read-text`, `write-text`, `read`, `write`, `mkdir` — scoped to app dir and temp |
| `shell` | `allow-execute` — for running system commands |
| `http` | default — connect only to approved origins |

---

## Content Security Policy

```
default-src 'self';
style-src 'self' 'unsafe-inline';
img-src 'self' data: asset: https:;
connect-src 'self'
  http://localhost:11434
  https://api.openai.com
  https://api.anthropic.com
  https://api.mistral.ai
  https://generativelanguage.googleapis.com;
```

HTTP is blocked by default. Only whitelisted HTTPS endpoints (and localhost for Ollama) are reachable.

---

## TLS

All outbound HTTP requests use `reqwest` with default TLS settings (TLS 1.2+ required). Certificate verification is enabled. No custom trust anchors are added.

---

## Security Checklist for New Features

- [ ] Does it send data externally? → Add audit log entry
- [ ] Does it handle user-provided text? → Run PII detection first
- [ ] Does it store secrets? → Use Stronghold, not the SQLite DB
- [ ] Does it need filesystem access? → Scope the fs capability
- [ ] Does it need a new HTTP endpoint? → Add to CSP `connect-src`
