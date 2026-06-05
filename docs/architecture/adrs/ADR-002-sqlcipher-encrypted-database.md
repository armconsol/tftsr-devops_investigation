# ADR-002: SQLCipher for Encrypted Storage

**Status**: Accepted
**Date**: 2025-Q3
**Deciders**: sarman

---

## Context

All incident data (titles, descriptions, log contents, AI conversations, resolution steps, RCA documents) must be stored locally and at rest must be encrypted. The application cannot rely on OS-level full-disk encryption being enabled.

Requirements:
- AES-256 encryption of the full database file
- Key derivation suitable for per-installation keys (not user passwords)
- No plaintext data accessible if the `.db` file is copied off-machine
- Rust-compatible SQLite bindings

---

## Decision

Use **SQLCipher** via `rusqlite` with the `bundled-sqlcipher-vendored-openssl` feature flag.

---

## Rationale

**Alternatives considered:**

| Option | Pros | Cons |
|--------|------|------|
| **SQLCipher** (chosen) | Transparent full-DB encryption, AES-256, PBKDF2 key derivation, vendored so no system dep | Larger binary; not standard SQLite |
| Plain SQLite | Simple, well-known | No encryption — ruled out |
| SQLite + file-level encryption | Flexible | No atomicity; complex implementation |
| LevelDB / RocksDB | Fast, encrypted options exist | No SQL, harder migration |
| `sled` (Rust-native) | Modern, async-friendly | No SQL, immature for complex schemas |

**SQLCipher specifics chosen:**
```
PRAGMA cipher_page_size = 16384;     -- Matches 16KB kernel page (Apple Silicon)
PRAGMA kdf_iter = 256000;            -- 256k PBKDF2 iterations
PRAGMA cipher_hmac_algorithm = HMAC_SHA512;
PRAGMA cipher_kdf_algorithm = PBKDF2_HMAC_SHA512;
```

The `cipher_page_size = 16384` is specifically tuned for Apple Silicon (M-series) which uses 16KB kernel pages — using 4096 (SQLCipher default) causes page boundary issues.

---

## Key Management

Per ADR-005, encryption keys are auto-generated at runtime:
- **Release builds**: Random 256-bit key generated at first launch, stored in `.dbkey` (mode 0600)
- **Debug builds**: Hardcoded dev key (`dev-key-change-in-prod`)
- **Override**: `TRCAA_DB_KEY` (or legacy `TRCAA_DB_KEY`) environment variable

---

## Consequences

**Positive:**
- Full database encryption transparent to all SQL queries
- Vendored OpenSSL means no system library dependency (important for portable AppImage/DMG)
- SHA-512 HMAC provides authenticated encryption (tampering detected)

**Negative:**
- `bundled-sqlcipher-vendored-openssl` significantly increases compile time and binary size
- Cannot use standard SQLite tooling to inspect database files (must use sqlcipher CLI)
- `cipher_page_size` mismatch between debug/release would corrupt databases — mitigated by auto-migration (ADR-005)

**Migration Handling:**
If a plain SQLite database is detected in a release build (e.g., developer switched from debug), `migrate_plain_to_encrypted()` automatically migrates using `ATTACH DATABASE` + `sqlcipher_export`. A `.db.plain-backup` file is created before migration.
