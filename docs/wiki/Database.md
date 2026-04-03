# Database

## Overview

TFTSR uses **SQLite** via `rusqlite` with the `bundled-sqlcipher` feature for AES-256 encryption in production. 11 versioned migrations are tracked in the `_migrations` table.

**DB file location:** `{app_data_dir}/tftsr.db`

---

## Encryption

| Build type | Encryption | Key |
|-----------|-----------|-----|
| Debug (`debug_assertions`) | None (plain SQLite) | — |
| Release | SQLCipher AES-256 | `TFTSR_DB_KEY` env var |

**SQLCipher settings (production):**
- Cipher: AES-256-CBC
- Page size: 4096 bytes
- KDF: PBKDF2-HMAC-SHA512, 256,000 iterations
- HMAC: HMAC-SHA512

```rust
// Simplified init logic
pub fn init_db(data_dir: &Path) -> anyhow::Result<Connection> {
    let key = env::var("TFTSR_DB_KEY")
        .unwrap_or_else(|_| "dev-key-change-in-prod".to_string());
    let conn = if cfg!(debug_assertions) {
        Connection::open(db_path)?           // plain SQLite
    } else {
        open_encrypted_db(db_path, &key)?    // SQLCipher AES-256
    };
    run_migrations(&conn)?;
    Ok(conn)
}
```

---

## Schema (11 Migrations)

### 001 — issues

```sql
CREATE TABLE issues (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    description TEXT,
    severity    TEXT NOT NULL,  -- 'critical', 'high', 'medium', 'low'
    status      TEXT NOT NULL,  -- 'open', 'investigating', 'resolved', 'closed'
    category    TEXT,
    source      TEXT,
    created_at  TEXT NOT NULL,  -- 'YYYY-MM-DD HH:MM:SS'
    updated_at  TEXT NOT NULL,
    resolved_at TEXT,           -- nullable
    assigned_to TEXT,
    tags        TEXT            -- JSON array stored as TEXT
);
```

### 002 — log_files

```sql
CREATE TABLE log_files (
    id           TEXT PRIMARY KEY,
    issue_id     TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    file_name    TEXT NOT NULL,
    file_path    TEXT NOT NULL,
    file_size    INTEGER,
    mime_type    TEXT,
    content_hash TEXT,          -- SHA-256 hex of original content
    uploaded_at  TEXT NOT NULL,
    redacted     INTEGER DEFAULT 0  -- boolean: 0/1
);
```

### 003 — pii_spans

```sql
CREATE TABLE pii_spans (
    id             TEXT PRIMARY KEY,
    log_file_id    TEXT NOT NULL REFERENCES log_files(id) ON DELETE CASCADE,
    pii_type       TEXT NOT NULL,
    start_offset   INTEGER NOT NULL,
    end_offset     INTEGER NOT NULL,
    original_value TEXT NOT NULL,
    replacement    TEXT NOT NULL
);
```

### 004 — ai_conversations

```sql
CREATE TABLE ai_conversations (
    id         TEXT PRIMARY KEY,
    issue_id   TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    provider   TEXT NOT NULL,
    model      TEXT NOT NULL,
    created_at TEXT NOT NULL,
    title      TEXT
);
```

### 005 — ai_messages

```sql
CREATE TABLE ai_messages (
    id              TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
    role            TEXT NOT NULL CHECK(role IN ('system', 'user', 'assistant')),
    content         TEXT NOT NULL,
    token_count     INTEGER DEFAULT 0,
    created_at      TEXT NOT NULL
);
```

### 006 — resolution_steps (5-Whys)

```sql
CREATE TABLE resolution_steps (
    id           TEXT PRIMARY KEY,
    issue_id     TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    step_order   INTEGER NOT NULL,   -- 1–5
    why_question TEXT NOT NULL,
    answer       TEXT,
    evidence     TEXT,
    created_at   TEXT NOT NULL
);
```

### 007 — documents

```sql
CREATE TABLE documents (
    id         TEXT PRIMARY KEY,
    issue_id   TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    doc_type   TEXT NOT NULL,  -- 'rca', 'postmortem'
    title      TEXT NOT NULL,
    content_md TEXT NOT NULL,
    created_at INTEGER NOT NULL,  -- milliseconds since epoch
    updated_at INTEGER NOT NULL
);
```

> **Note:** `documents` uses INTEGER milliseconds; `issues` and `log_files` use TEXT timestamps.

### 008 — audit_log

```sql
CREATE TABLE audit_log (
    id          TEXT PRIMARY KEY,
    timestamp   TEXT NOT NULL DEFAULT (datetime('now')),
    action      TEXT NOT NULL,       -- e.g., 'ai_send', 'publish_to_confluence'
    entity_type TEXT NOT NULL,       -- e.g., 'issue', 'document'
    entity_id   TEXT NOT NULL,
    user_id     TEXT DEFAULT 'local',
    details     TEXT                 -- JSON with hashes, log_file_ids, etc.
);
```

### 009 — settings

```sql
CREATE TABLE settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### 010 — issues_fts (Full-Text Search)

```sql
CREATE VIRTUAL TABLE issues_fts USING fts5(
    id UNINDEXED,
    title,
    description,
    content='issues',
    content_rowid='rowid'
);
```

### 011 — credentials & integration_config (v0.2.3+)

**Integration credentials table:**
```sql
CREATE TABLE credentials (
    id TEXT PRIMARY KEY,
    service TEXT NOT NULL CHECK(service IN ('confluence','servicenow','azuredevops')),
    token_hash TEXT NOT NULL,        -- SHA-256 hash for audit
    encrypted_token TEXT NOT NULL,   -- AES-256-GCM encrypted
    created_at TEXT NOT NULL,
    expires_at TEXT,
    UNIQUE(service)
);
```

**Integration configuration table:**
```sql
CREATE TABLE integration_config (
    id TEXT PRIMARY KEY,
    service TEXT NOT NULL CHECK(service IN ('confluence','servicenow','azuredevops')),
    base_url TEXT NOT NULL,
    username TEXT,              -- ServiceNow only
    project_name TEXT,          -- Azure DevOps only
    space_key TEXT,             -- Confluence only
    auto_create_enabled INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL,
    UNIQUE(service)
);
```

**Encryption:**
- OAuth2 tokens encrypted with AES-256-GCM
- Key derived from `TFTSR_DB_KEY` environment variable
- Random 96-bit nonce per encryption
- Format: `base64(nonce || ciphertext || tag)`

**Usage:**
- OAuth2 flows (Confluence, Azure DevOps): Store encrypted bearer token
- Basic auth (ServiceNow): Store encrypted password
- One credential per service (enforced by UNIQUE constraint)

---

## Key Design Notes

- All primary keys are **UUID v7** (time-sortable)
- Boolean flags stored as `INTEGER` (`0`/`1`)
- JSON arrays (e.g., `tags`) stored as `TEXT`
- `issues` / `log_files` timestamps: `TEXT` (`YYYY-MM-DD HH:MM:SS`)
- `documents` timestamps: `INTEGER` (milliseconds since epoch)
- All foreign keys with `ON DELETE CASCADE`
- Migration history tracked in `_migrations` table (name + applied_at)

---

## Rust Model Types

Key structs in `db/models.rs`:

```rust
pub struct Issue {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub severity: String,
    pub status: String,
    // ...
}

pub struct IssueDetail {         // Nested — returned by get_issue()
    pub issue: Issue,
    pub log_files: Vec<LogFile>,
    pub resolution_steps: Vec<ResolutionStep>,
    pub conversations: Vec<AiConversation>,
}

pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub action: String,          // NOT event_type
    pub entity_type: String,     // NOT destination
    pub entity_id: String,       // NOT status
    pub user_id: String,
    pub details: Option<String>,
}
```
