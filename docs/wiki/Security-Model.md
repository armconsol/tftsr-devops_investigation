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
- **Page size:** 16384 bytes
- **Key source:** `TFTSR_DB_KEY` environment variable

Debug builds use plain SQLite (no encryption) for developer convenience.

Release builds now fail startup if `TFTSR_DB_KEY` is missing or empty.

---

## Credential Encryption

Integration tokens are encrypted with AES-256-GCM before persistence:
- **Key source:** `TFTSR_ENCRYPTION_KEY` (required in release builds)
- **Key derivation:** SHA-256 hash of key material to a fixed 32-byte AES key
- **Nonce:** Cryptographically secure random nonce per encryption

Release builds fail secure operations if `TFTSR_ENCRYPTION_KEY` is unset or empty.

The Stronghold plugin remains enabled and now uses a per-installation salt derived from the app data directory path hash instead of a fixed static salt.

---

## PII Redaction

**Mandatory path:** No text can be sent to an AI provider without going through the PII detection and user-approval flow.

```
log file → detect_pii() → user approves spans → apply_redactions() → AI provider
```

- Original text **never leaves the machine**
- Only the redacted version is transmitted
- The SHA-256 hash of the redacted text is recorded in the audit log for integrity verification
- `pii_spans.original_value` is cleared after redaction to avoid retaining raw detected secrets in storage
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

### Tamper Evidence

`audit_log` entries now include:
- `prev_hash` — hash of the previous audit entry
- `entry_hash` — SHA-256 hash of current entry payload + `prev_hash`

This creates a hash chain and makes post-hoc modification detectable.

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
| `shell` | `allow-open` only |
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

All outbound HTTP requests use `reqwest` with certificate verification enabled and a request timeout configured for provider calls.

CI/CD currently uses internal `http://` endpoints for self-hosted Gitea release automation on a trusted LAN. Recommended hardening: migrate runners and API calls to HTTPS with internal certificates.

---

## MCP Server Security

MCP server support introduces external tool execution capabilities. The following controls mitigate the associated risks.

### Auth Value Storage

- Auth tokens (API keys, bearer tokens, OAuth2 access tokens) are encrypted with **AES-256-GCM** before persistence in `mcp_servers.auth_value`.
- Encryption uses the same key derivation as integration credentials (`TFTSR_ENCRYPTION_KEY` → SHA-256 → 32-byte AES key).
- Random 96-bit nonce per encryption operation.
- Format: `base64(nonce || ciphertext || tag)`.

### Server-Side Response Scrubbing

- `list_mcp_servers` and all CRUD commands set `auth_value = None` before returning to the frontend.
- The encrypted ciphertext never reaches the WebView layer.
- Decryption only occurs internally when establishing a connection (discovery) or executing a tool call.

### Audit Trail

- `write_audit_event` is called **before** every MCP tool execution with:
  - `action`: `"mcp_tool_call"`
  - `entity_type`: `"mcp_tool"`
  - `entity_id`: the tool key being invoked
  - `details`: JSON containing server ID, tool name, and argument hash
- This provides a complete, tamper-evident record of all external tool invocations.

### PII Scan on Arguments

- Before dispatching a tool call, the arguments JSON is scanned through the PII detection pipeline.
- If PII is detected, a **non-blocking warning** is surfaced to the user.
- This prevents inadvertent leakage of credentials, email addresses, or IP addresses to external MCP servers.

### Stdio Transport Path Validation

- `build_stdio_transport()` rejects any `command` that is not an absolute path.
- This prevents:
  - Path traversal attacks (e.g., `../../malicious`)
  - Reliance on `$PATH` resolution which could be manipulated
  - Unintended execution of relative-path binaries

### Network Boundaries

- HTTP transport uses `reqwest` with TLS certificate verification for HTTPS endpoints.
- stdio transport communicates only with locally spawned processes (no network exposure).
- MCP server URLs should be added to the Content Security Policy `connect-src` if fetched from the WebView layer.

### Cascade Deletes

- Removing an MCP server cascades to delete all associated `mcp_tools` and `mcp_resources` records.
- The live connection is also removed from the in-memory connection pool.
- No orphaned tool definitions can persist after server removal.

---

## Security Checklist for New Features

- [ ] Does it send data externally? → Add audit log entry
- [ ] Does it handle user-provided text? → Run PII detection first
- [ ] Does it store secrets? → Use Stronghold, not the SQLite DB
- [ ] Does it need filesystem access? → Scope the fs capability
- [ ] Does it need a new HTTP endpoint? → Add to CSP `connect-src`
- [ ] Does it add a new provider endpoint? → Avoid query-param secrets, use auth headers
