# Security Audit Report

**Application**: Troubleshooting and RCA Assistant (TRCAA)
**Audit Date**: 2026-04-06
**Scope**: All git-tracked source files (159 files)
**Context**: Pre-open-source release under MIT license

---

## Executive Summary

The codebase is generally well-structured with several positive security practices already in place: parameterized SQL queries, AES-256-GCM credential encryption, PKCE for OAuth flows, PII detection and redaction before AI transmission, hash-chained audit logs, and a restrictive CSP. However, the audit identified **3 CRITICAL**, **5 HIGH**, **5 MEDIUM**, and **5 LOW** findings that must be addressed before public release.

---

## CRITICAL Findings

### C1. Corporate-Internal Documents Shipped in Repository

**Files**:
- `GenAI API User Guide.md` (entire file)
- `HANDOFF-MSI-GENAI.md` (entire file)

**Issue**: These files contain proprietary Motorola Solutions / MSI internal documentation. `GenAI API User Guide.md` is authored by named MSI employees (Dipjyoti Bisharad, Jahnavi Alike, Sunil Vurandur, Anjali Kamath, Vibin Jacob, Girish Manivel) and documents internal API contracts at `genai-service.stage.commandcentral.com` and `genai-service.commandcentral.com`. `HANDOFF-MSI-GENAI.md` explicitly references "MSI GenAI API" integration details including internal endpoint URLs, header formats, and payload contracts.

Publishing these files under MIT license likely violates corporate IP agreements and exposes internal infrastructure details.

**Recommended Fix**: Remove both files from the repository entirely and scrub from git history using `git filter-repo` before making the repo public.

---

### C2. Internal Infrastructure URLs Hardcoded in CSP and Source

**File**: `src-tauri/tauri.conf.json`, line 13
**Also**: `src-tauri/src/ai/openai.rs`, line 219

**Issue**: The CSP `connect-src` directive includes corporate-internal endpoints:
```
https://genai-service.stage.commandcentral.com
https://genai-service.commandcentral.com
```

Additionally, `openai.rs` line 219 sends `X-msi-genai-client: troubleshooting-rca-assistant` as a hardcoded header in the custom REST path, tying the application to an internal MSI service.

These expose internal service infrastructure to anyone reading the source and indicate the app was designed to interact with corporate systems.

**Recommended Fix**:
- Remove the two `commandcentral.com` entries from the CSP.
- Remove or make the `X-msi-genai-client` header configurable rather than hardcoded.
- Audit the CSP to ensure only generic/public endpoints remain (OpenAI, Anthropic, Mistral, Google, Ollama, Atlassian, Microsoft are fine).

---

### C3. Private Gogs Server IP Exposed in All CI Workflows

**Files**:
- `.gitea/workflows/test.yml` (lines 17, 44, 72, 99, 126)
- `.gitea/workflows/auto-tag.yml` (lines 31, 52, 79, 95, 97, 141, 162, 227, 252, 313, 338, 401, 464)
- `.gitea/workflows/build-images.yml` (lines 4, 10, 11, 16-18, 33, 46, 69, 92)

**Issue**: All CI workflow files reference `172.0.0.29:3000` (a private Gogs instance) and `sarman` username. While the IP is RFC1918 private address space, it reveals internal infrastructure topology and the developer's username across dozens of lines. The `build-images.yml` also exposes `REGISTRY_USER: sarman` and container registry details.

**Recommended Fix**: Before open-sourcing, replace all workflow files with GitHub Actions equivalents, or at minimum replace the hardcoded private IP and username with parameterized variables or remove the `.gitea/` directory entirely if moving to GitHub.

---

## HIGH Findings

### H1. Hardcoded Development Encryption Key in Auth Module

**File**: `src-tauri/src/integrations/auth.rs`, line 179

```rust
return Ok("dev-key-change-me-in-production-32b".to_string());
```

**Issue**: In debug builds, the credential encryption key is a well-known hardcoded string. Anyone reading the source can decrypt any credentials stored by a debug build. Since this is about to be open source, attackers know the exact key to use against any debug-mode installation.

**Also at**: `src-tauri/src/db/connection.rs`, line 39: `"dev-key-change-in-prod"`

While this is gated behind `cfg!(debug_assertions)`, open-sourcing the code means the development key is permanently public knowledge. If any user runs a debug build or if the release profile check is ever misconfigured, all stored credentials are trivially decryptable.

**Recommended Fix**:
- Remove the hardcoded dev key entirely.
- In debug mode, auto-generate and persist a random key the same way the release path does (lines 44-57 of `connection.rs` already implement this pattern).
- Document in a `SECURITY.md` file that credentials are encrypted at rest and the key management approach.

---

### H2. Encryption Key Derivation Uses Raw SHA-256 Instead of a KDF

**File**: `src-tauri/src/integrations/auth.rs`, lines 185-191

```rust
fn derive_aes_key() -> Result<[u8; 32], String> {
    let key_material = get_encryption_key_material()?;
    let digest = Sha256::digest(key_material.as_bytes());
    ...
}
```

**Issue**: The AES-256-GCM key is derived from the raw material by a single SHA-256 hash. There is no salt and no iteration count. This means if the key material has low entropy (as the dev key does), the derived key is trivially brute-forceable. In contrast, the database encryption properly uses PBKDF2-HMAC-SHA512 with 256,000 iterations (line 69 of `connection.rs`).

**Recommended Fix**: Use a proper KDF (PBKDF2, Argon2, or HKDF) with a persisted random salt and sufficient iteration count for deriving the AES key. The `db/connection.rs` module already demonstrates the correct approach.

---

### H3. Release Build Fails Open if TRCAA_ENCRYPTION_KEY is Unset

**File**: `src-tauri/src/integrations/auth.rs`, line 182

```rust
Err("TRCAA_ENCRYPTION_KEY must be set in release builds".to_string())
```

**Issue**: In release mode, if the `TRCAA_ENCRYPTION_KEY` (or legacy `TRCAA_ENCRYPTION_KEY`) environment variable is not set, any attempt to store or retrieve credentials will fail with an error. Unlike the database key management (which auto-generates and persists a key), credential encryption requires manual environment variable configuration. For a desktop app distributed to end users, this is an unworkable UX: users will never set this variable, meaning credential storage will be broken out of the box in release builds.

**Recommended Fix**: Mirror the database key management pattern: auto-generate a random key on first use, persist it to a file in the app data directory with 0600 permissions (as already done for `.dbkey`), and read it back on subsequent launches.

---

### H4. API Keys Transmitted to Frontend via IPC and Stored in Memory

**File**: `src/stores/settingsStore.ts`, lines 56-63
**Also**: `src-tauri/src/state.rs`, line 12 (`api_key` field in `ProviderConfig`)

**Issue**: The `ProviderConfig` struct includes `api_key: String` which is serialized over Tauri's IPC bridge from Rust to TypeScript and back. The settings store correctly strips API keys before persisting to `localStorage` (line 60: `api_key: ""`), which is good. However, the full API key lives in the Zustand store in browser memory for the duration of the session. If the webview's JavaScript context is compromised (e.g., via a future XSS or a malicious Tauri plugin), the API key is accessible.

**Recommended Fix**: Store API keys exclusively in the Rust backend (encrypted in the database). The frontend should only send a provider identifier; the backend should look up the key internally before making API calls. This eliminates API keys from the IPC surface entirely.

---

### H5. Filesystem Capabilities Are Overly Broad

**File**: `src-tauri/capabilities/default.json`, lines 16-24

```json
"fs:allow-read",
"fs:allow-write",
"fs:allow-mkdir",
```

**Issue**: The capabilities include `fs:allow-read` and `fs:allow-write` without scope constraints (in addition to the properly scoped `fs:scope-app-recursive` and `fs:scope-temp-recursive`). The unscoped `fs:allow-read`/`fs:allow-write` permissions may override the scope restrictions, potentially allowing the frontend JavaScript to read or write arbitrary files on the filesystem depending on Tauri 2.x ACL resolution order.

**Recommended Fix**: Remove the unscoped `fs:allow-read`, `fs:allow-write`, and `fs:allow-mkdir` permissions. Keep only the scoped variants (`fs:allow-app-read-recursive`, `fs:allow-app-write-recursive`, `fs:allow-temp-read-recursive`, `fs:allow-temp-write-recursive`) plus the `fs:scope-*` directives. File dialog operations (`dialog:allow-open`, `dialog:allow-save`) already handle user-initiated file access.

---

## MEDIUM Findings

### M1. Export Document Accepts Arbitrary Output Directory Without Validation

**File**: `src-tauri/src/commands/docs.rs`, lines 154-162

```rust
let base_dir = if output_dir.is_empty() || output_dir == "." {
    dirs::download_dir().unwrap_or_else(|| { ... })
} else {
    PathBuf::from(&output_dir)
};
```

**Issue**: The `export_document` command accepts an `output_dir` string from the frontend and writes files to it without canonicalization or path validation. While the frontend likely provides a dialog-selected path, a compromised frontend could write files to arbitrary directories (e.g., `../../etc/cron.d/` on Linux). There is no check that `output_dir` is within an expected scope.

**Recommended Fix**: Canonicalize the path and validate it against an allowlist of directories (Downloads, app data, or user-selected via dialog). Reject paths containing `..` or pointing to system directories.

---

### M2. OAuth Callback Server Listens on Fixed Port Without CSRF Protection

**File**: `src-tauri/src/integrations/callback_server.rs`, lines 14-33

**Issue**: The OAuth callback server binds to `127.0.0.1:8765`. While binding to localhost is correct, the server accepts any HTTP GET to `/callback?code=...&state=...` without verifying the origin of the request. A malicious local process or a webpage with access to `localhost` could forge a callback request. The `state` parameter provides some CSRF protection, but it is stored in a global `HashMap` without TTL, meaning stale state values persist indefinitely.

**Recommended Fix**:
- Add a TTL (e.g., 10 minutes) to OAuth state entries to prevent stale state accumulation.
- Consider using a random high port instead of the fixed 8765 to reduce predictability.

---

### M3. Audit Log Hash Chain is Appendable but Not Verifiable

**File**: `src-tauri/src/audit/log.rs`, lines 4-16

**Issue**: The audit log implements a hash chain (each entry includes the hash of the previous entry), which is good for tamper detection. However, there is no command or function to verify the integrity of the chain. An attacker with database access could modify entries and recompute all subsequent hashes. Without an external anchor (e.g., periodic hash checkpoint to an external store), the chain only proves ordering, not immutability.

**Recommended Fix**: Add a `verify_audit_chain()` function and consider periodically exporting chain checkpoints to a file outside the database. Document the threat model in `SECURITY.md`.

---

### M4. Non-Windows Key File Permissions Not Enforced

**File**: `src-tauri/src/db/connection.rs`, lines 25-28

```rust
#[cfg(not(unix))]
fn write_key_file(path: &Path, key: &str) -> anyhow::Result<()> {
    std::fs::write(path, key)?;
    Ok(())
}
```

**Issue**: On non-Unix platforms (Windows), the database key file is written with default permissions, potentially making it world-readable. The Unix path correctly uses mode `0o600`.

**Recommended Fix**: On Windows, use platform-specific ACL APIs to restrict the key file to the current user, or at minimum document this limitation.

---

### M5. `unsafe-inline` in Style CSP Directive

**File**: `src-tauri/tauri.conf.json`, line 13

```
style-src 'self' 'unsafe-inline'
```

**Issue**: The CSP allows `unsafe-inline` for styles. While this is common in React/Tailwind applications and the attack surface is lower than `unsafe-inline` for scripts, it still permits style-based data exfiltration attacks (e.g., CSS injection to leak attribute values).

**Recommended Fix**: If feasible, use nonce-based or hash-based style CSP. If not feasible due to Tailwind's runtime style injection, document this as an accepted risk.

---

## LOW Findings

### L1. `http:default` Capability Grants Broad Network Access

**File**: `src-tauri/capabilities/default.json`, line 28

**Issue**: The `http:default` permission allows the frontend to make arbitrary HTTP requests. Combined with the broad CSP `connect-src`, this gives the webview significant network access. For a desktop app this is often necessary, but it should be documented and reviewed.

**Recommended Fix**: Consider restricting `http` permissions to specific URL patterns matching only the known AI provider APIs and integration endpoints.

---

### L2. IntelliJ IDEA Config Files Tracked in Git

**Files**:
- `.idea/.gitignore`
- `.idea/copilot.data.migration.ask2agent.xml`
- `.idea/misc.xml`
- `.idea/modules.xml`
- `.idea/trcaa-devops_investigation.iml`
- `.idea/vcs.xml`

**Issue**: IDE configuration files are tracked. These may leak editor preferences and do not belong in an open-source repository.

**Recommended Fix**: Add `.idea/` to `.gitignore` and remove from tracking with `git rm -r --cached .idea/`.

---

### L3. Placeholder OAuth Client IDs in Source

**File**: `src-tauri/src/commands/integrations.rs`, lines 181, 187

```rust
"confluence-client-id-placeholder"
"ado-client-id-placeholder"
```

**Issue**: These placeholder strings are used as fallbacks when environment variables are not set. While they are obviously not real credentials, they could confuse users or be mistaken for actual client IDs in bug reports.

**Recommended Fix**: Make the OAuth flow fail explicitly with a clear error message when the client ID environment variable is not set, rather than falling back to a placeholder.

---

### L4. Username `sarman` Embedded in CI Workflows and Makefile

**Files**: `.gitea/workflows/*.yml`, `Makefile` line 2

**Issue**: The developer's username appears throughout CI configuration. While not a security vulnerability per se, it is a privacy concern for open-source release.

**Recommended Fix**: Parameterize the username in CI workflows. Update the Makefile to use a generic repository reference.

---

### L5. `shell:allow-open` Capability Enabled

**File**: `src-tauri/capabilities/default.json`, line 27

**Issue**: The `shell:allow-open` permission allows the frontend to open URLs in the system browser. This is used for OAuth flows and external links. While convenient, a compromised frontend could open arbitrary URLs.

**Recommended Fix**: This is acceptable for the app's functionality but should be documented. Consider restricting to specific URL patterns if Tauri 2.x supports it.

---

## Positive Security Observations

The following practices are already well-implemented:

1. **Parameterized SQL queries**: All database operations use `rusqlite::params![]` with positional parameters. No string interpolation in SQL. The dynamic query builder in `list_issues` and `get_audit_log` correctly uses indexed parameter placeholders.

2. **SQLCipher encryption at rest**: Release builds encrypt the database using AES-256-CBC via SQLCipher with PBKDF2-HMAC-SHA512 (256k iterations).

3. **PII detection and mandatory redaction**: Log files must pass PII detection and redaction before being sent to AI providers (`redacted_path_for()` enforces this check).

4. **PKCE for OAuth**: The OAuth implementation uses PKCE (S256) with cryptographically random verifiers.

5. **Hash-chained audit log**: Every security-relevant action is logged with a SHA-256 hash chain.

6. **Path traversal prevention**: `upload_log_file` uses `std::fs::canonicalize()` and validates the result is a regular file with size limits.

7. **No `dangerouslySetInnerHTML` or `eval()`**: The frontend renders AI responses as plain text via `{msg.content}` in JSX, preventing XSS from AI model output.

8. **API key scrubbing from localStorage**: The settings store explicitly strips `api_key` before persisting (line 60 of `settingsStore.ts`).

9. **No shell command injection**: All `std::process::Command` calls use hardcoded binary names with literal arguments. No user input is passed to shell commands.

10. **No secrets in git history**: `.gitignore` properly excludes `.env`, `.secrets`, `secrets.yml`, and related files. No private keys or certificates are tracked.

11. **Mutex guards not held across await points**: The codebase correctly drops `MutexGuard` before `.await` by scoping locks inside `{ }` blocks.

---

## Recommendations Summary (Priority Order)

| Priority | Action | Effort |
|----------|--------|--------|
| **P0** | Remove `GenAI API User Guide.md` and `HANDOFF-MSI-GENAI.md` from repo and git history | Small |
| **P0** | Remove `commandcentral.com` URLs from CSP and hardcoded MSI headers from `openai.rs` | Small |
| **P0** | Replace or parameterize private IP (`172.0.0.29`) and username in all `.gitea/` workflows | Medium |
| **P1** | Replace hardcoded dev encryption keys with auto-generated per-install keys | Small |
| **P1** | Use proper KDF (PBKDF2/HKDF) for AES key derivation in `auth.rs` | Small |
| **P1** | Auto-generate encryption key for credential storage (mirror `connection.rs` pattern) | Small |
| **P1** | Remove unscoped `fs:allow-read`/`fs:allow-write` from capabilities | Small |
| **P2** | Move API key storage to backend-only (remove from IPC surface) | Medium |
| **P2** | Add path validation to `export_document` output directory | Small |
| **P2** | Add TTL to OAuth state entries | Small |
| **P2** | Add audit chain verification function | Small |
| **P3** | Remove `.idea/` from git tracking | Trivial |
| **P3** | Replace placeholder OAuth client IDs with explicit errors | Trivial |
| **P3** | Parameterize username in CI/Makefile | Small |

---

*Report generated by security audit of git-tracked source files at commit HEAD on feature/ai-tool-calling-integration-search branch.*
