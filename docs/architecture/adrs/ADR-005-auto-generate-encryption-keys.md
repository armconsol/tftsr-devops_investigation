# ADR-005: Auto-generate Encryption Keys at Runtime

**Status**: Accepted
**Date**: 2026-04
**Deciders**: sarman

---

## Context

The application uses two encryption keys:
1. **Database key** (`TRCAA_DB_KEY` (or legacy `TRCAA_DB_KEY`)): SQLCipher AES-256 key for the full database
2. **Credential key** (`TRCAA_ENCRYPTION_KEY` (or legacy `TRCAA_ENCRYPTION_KEY`)): AES-256-GCM key for token/API key encryption

The original design required both to be set as environment variables in release builds. This caused:
- **Critical failure on Mac**: Fresh installs would crash at startup with "file is not a database" error
- **Silent failure on save**: Saving AI providers would fail with "TRCAA_ENCRYPTION_KEY must be set in release builds"
- **Developer friction**: Switching from `cargo tauri dev` (debug, plain SQLite) to a release build would crash because the existing plain database couldn't be opened as encrypted

---

## Decision

Auto-generate cryptographically secure 256-bit keys at first launch and persist them to the app data directory with restricted file permissions.

---

## Key Storage

| Key | File | Permissions | Location |
|-----|------|-------------|----------|
| Database | `.dbkey` | `0600` (owner r/w only) | `$TRCAA_DATA_DIR/` |
| Credentials | `.enckey` | `0600` (owner r/w only) | `$TRCAA_DATA_DIR/` |

**Platform data directories:**
- macOS: `~/Library/Application Support/trcaa/`
- Linux: `~/.local/share/trcaa/`
- Windows: `%APPDATA%\trcaa\`

---

## Key Resolution Order

For both keys:
1. Check environment variable (`TRCAA_DB_KEY` (or legacy `TRCAA_DB_KEY`) / `TRCAA_ENCRYPTION_KEY` (or legacy `TRCAA_ENCRYPTION_KEY`)) — use if set and non-empty
2. If debug build — use hardcoded dev key (never touches filesystem)
3. If `.dbkey` / `.enckey` exists and is non-empty — load from file
4. Otherwise — generate 32 random bytes via `OsRng`, hex-encode to 64-char string, write to file with `mode 0600`

---

## Plain-to-Encrypted Migration

When a release build encounters an existing plain SQLite database (written by a debug build), rather than crashing:

```
1. Detect plain SQLite via 16-byte header check ("SQLite format 3\0")
2. Copy database to .db.plain-backup
3. Open plain database
4. ATTACH encrypted database at temp path with new key
5. SELECT sqlcipher_export('encrypted')   -- copies all tables, indexes, triggers
6. DETACH encrypted
7. rename(temp_encrypted, original_path)
8. Open encrypted database with key
```

---

## Alternatives Considered

| Option | Pros | Cons |
|--------|------|------|
| **Auto-generate keys** (chosen) | Works out-of-the-box, no user config | Key file loss = data loss (acceptable: key + DB on same machine) |
| Require env vars (original) | Explicit — users know their key | Crashes on fresh install, poor UX |
| Derive from machine ID | No file to lose | Machine ID changes break DB on hardware changes |
| OS keychain | Most secure | Complex cross-platform implementation; adds dependency |
| Prompt user for password | User controls key | Poor UX for a tool; password complexity issues |

**Why not OS keychain:**
The `tauri-plugin-stronghold` already provides a keychain-like abstraction for credentials, but integrating SQLCipher key retrieval into Stronghold would create a chicken-and-egg problem: Stronghold itself needs to be initialized before the database that stores Stronghold's key material.

---

## Consequences

**Positive:**
- Zero-configuration installation — app works on first launch
- Developers can freely switch between debug and release builds
- Environment variable override still available for automated/enterprise deployments
- Key files are protected by Unix file permissions (`0600`)

**Negative:**
- If `.dbkey` or `.enckey` are deleted, the database and all stored credentials become permanently inaccessible
- Key files are not themselves encrypted — OS-level protection depends on filesystem permissions
- Not suitable for multi-user scenarios where different users need isolated key material (single-user desktop app — acceptable)

**Mitigation for key loss:**
Document clearly that backing up `$TRCAA_DATA_DIR` (including hidden files) preserves both key files and database. Loss of keys without losing the database = data loss.
