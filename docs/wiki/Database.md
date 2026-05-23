# Database

## Overview

TFTSR uses **SQLite** via `rusqlite` with the `bundled-sqlcipher` feature for AES-256 encryption in production. 18 versioned migrations are tracked in the `_migrations` table.

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

## Schema (18 Migrations)

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

### 012 — image_attachments (v0.2.7+)

```sql
CREATE TABLE image_attachments (
    id TEXT PRIMARY KEY,
    issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL DEFAULT '',
    file_size INTEGER NOT NULL DEFAULT 0,
    mime_type TEXT NOT NULL DEFAULT 'image/png',
    upload_hash TEXT NOT NULL DEFAULT '',
    uploaded_at TEXT NOT NULL DEFAULT (datetime('now')),
    pii_warning_acknowledged INTEGER NOT NULL DEFAULT 1,
    is_paste INTEGER NOT NULL DEFAULT 0
);
```

**Features:**
- Image file metadata stored in database
- `upload_hash`: SHA-256 hash of file content (for deduplication)
- `pii_warning_acknowledged`: User confirmation that PII may be present
- `is_paste`: Flag for screenshots copied from clipboard

**Encryption:**
- OAuth2 tokens encrypted with AES-256-GCM
- Key derived from `TFTSR_DB_KEY` environment variable
- Random 96-bit nonce per encryption
- Format: `base64(nonce || ciphertext || tag)`

**Usage:**
- OAuth2 flows (Confluence, Azure DevOps): Store encrypted bearer token
- Basic auth (ServiceNow): Store encrypted password
- One credential per service (enforced by UNIQUE constraint)

### 017 — timeline_events (Incident Response Timeline)

```sql
CREATE TABLE timeline_events (
    id TEXT PRIMARY KEY,
    issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    description TEXT NOT NULL,
    metadata TEXT,          -- JSON object with event-specific data
    created_at TEXT NOT NULL
);

CREATE INDEX idx_timeline_events_issue ON timeline_events(issue_id);
CREATE INDEX idx_timeline_events_time ON timeline_events(created_at);
```

**Event Types:**
- `triage_started` — Incident response begins, initial issue properties recorded
- `log_uploaded` — Log file uploaded and analyzed
- `why_level_advanced` — 5-Whys entry completed, progression to next level
- `root_cause_identified` — Root cause determined from analysis
- `rca_generated` — Root Cause Analysis document created
- `postmortem_generated` — Post-mortem document created
- `document_exported` — Document exported to file (MD or PDF)

**Metadata Structure (JSON):**
```json
{
  "triage_started": {"severity": "high", "category": "network"},
  "log_uploaded": {"file_name": "app.log", "file_size": 2048576},
  "why_level_advanced": {"from_level": 2, "to_level": 3, "question": "Why did the service timeout?"},
  "root_cause_identified": {"root_cause": "DNS resolution failure", "confidence": 0.95},
  "rca_generated": {"doc_id": "doc_abc123", "section_count": 7},
  "postmortem_generated": {"doc_id": "doc_def456", "timeline_events_count": 12},
  "document_exported": {"format": "pdf", "file_path": "/home/user/docs/rca.pdf"}
}
```

**Design Notes:**
- Timeline events are **queryable** (indexed by issue_id and created_at) for document generation
- Dual-write: Events recorded to both `timeline_events` and `audit_log` — timeline for chronological reporting, audit_log for security/compliance
- `created_at`: TEXT UTC timestamp (`YYYY-MM-DD HH:MM:SS`)
- Non-blocking writes: Timeline events recorded asynchronously at key triage moments
- Cascade delete from issues ensures cleanup

### 018 — mcp_servers, mcp_tools, mcp_resources (MCP Server Support)

**MCP server registry:**
```sql
CREATE TABLE mcp_servers (
    id                TEXT PRIMARY KEY,
    name              TEXT NOT NULL,
    url               TEXT NOT NULL,
    transport_type    TEXT NOT NULL CHECK(transport_type IN ('stdio', 'http')),
    transport_config  TEXT NOT NULL DEFAULT '{}',
    auth_type         TEXT NOT NULL CHECK(auth_type IN ('none', 'api_key', 'bearer', 'oauth2')),
    auth_value        TEXT,              -- AES-256-GCM encrypted
    enabled           INTEGER NOT NULL DEFAULT 1,
    last_discovered_at TEXT,
    discovery_status  TEXT NOT NULL DEFAULT 'pending'
                      CHECK(discovery_status IN ('pending','connected','unreachable','error')),
    discovery_error   TEXT,
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**Discovered tools (populated by discovery):**
```sql
CREATE TABLE mcp_tools (
    id          TEXT PRIMARY KEY,
    server_id   TEXT NOT NULL,
    name        TEXT NOT NULL,           -- Original tool name from server
    tool_key    TEXT NOT NULL,           -- Sanitised key: mcp_{server}_{tool}
    description TEXT,
    parameters  TEXT NOT NULL DEFAULT '{}',  -- JSON Schema
    FOREIGN KEY(server_id) REFERENCES mcp_servers(id) ON DELETE CASCADE
);
```

**Discovered resources:**
```sql
CREATE TABLE mcp_resources (
    id          TEXT PRIMARY KEY,
    server_id   TEXT NOT NULL,
    uri         TEXT NOT NULL,
    name        TEXT,
    description TEXT,
    FOREIGN KEY(server_id) REFERENCES mcp_servers(id) ON DELETE CASCADE
);
```

**Design notes:**
- `auth_value` stored as AES-256-GCM ciphertext (same encryption as integration credentials)
- `transport_type` and `auth_type` enforce valid values via CHECK constraints
- `discovery_status` tracks connection state: `pending` → `connected` | `unreachable` | `error`
- Cascade deletes ensure removing a server cleans up all associated tools and resources
- Tools and resources are replaced atomically on each discovery run (delete-all + re-insert)

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

pub struct TimelineEvent {
    pub id: String,
    pub issue_id: String,
    pub event_type: String,
    pub description: String,
    pub metadata: Option<String>, // JSON
    pub created_at: String,
}
```
