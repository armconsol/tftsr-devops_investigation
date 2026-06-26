# Integrations

> **Status: ✅ Fully Implemented (v0.2.6)** — All three integrations (Confluence, ServiceNow, Azure DevOps) are production-ready with complete OAuth2/authentication flows and REST API clients.

---

## Confluence

**Purpose:** Publish RCA and post-mortem documents to Confluence spaces.

**Status:** ✅ **Implemented** (v0.2.3)

### Features
- OAuth2 authentication with PKCE flow
- List accessible spaces
- Search pages by CQL query
- Create new pages with optional parent
- Update existing pages with version management

### API Client (`src-tauri/src/integrations/confluence.rs`)

**Functions:**
```rust
test_connection(config: &ConfluenceConfig) -> Result<ConnectionResult, String>
list_spaces(config: &ConfluenceConfig) -> Result<Vec<Space>, String>
search_pages(config: &ConfluenceConfig, query: &str, space_key: Option<&str>) -> Result<Vec<Page>, String>
publish_page(config: &ConfluenceConfig, space_key: &str, title: &str, content_html: &str, parent_page_id: Option<&str>) -> Result<PublishResult, String>
update_page(config: &ConfluenceConfig, page_id: &str, title: &str, content_html: &str, version: i32) -> Result<PublishResult, String>
```

### Configuration (Settings → Integrations → Confluence)
```
Base URL:         https://yourorg.atlassian.net
Authentication:   OAuth2 (bearer token, encrypted at rest)
Default Space:    PROJ
```

### Implementation Details
- **API**: Confluence REST API v1 (`/rest/api/`)
- **Auth**: OAuth2 bearer token (encrypted with AES-256-GCM)
- **Endpoints**:
  - `GET /rest/api/user/current` — Test connection
  - `GET /rest/api/space` — List spaces
  - `GET /rest/api/content/search` — Search with CQL
  - `POST /rest/api/content` — Create page
  - `PUT /rest/api/content/{id}` — Update page
- **Page format**: Confluence Storage Format (XHTML)
- **TDD Tests**: 6 tests with mockito HTTP mocking

---

## ServiceNow

**Purpose:** Create and manage incident records in ServiceNow.

**Status:** ✅ **Implemented** (v0.2.3)

### Features
- Basic authentication (username/password)
- Search incidents by description
- Create new incidents with urgency/impact
- Get incident by sys_id or number
- Update existing incidents

### API Client (`src-tauri/src/integrations/servicenow.rs`)

**Functions:**
```rust
test_connection(config: &ServiceNowConfig) -> Result<ConnectionResult, String>
search_incidents(config: &ServiceNowConfig, query: &str) -> Result<Vec<Incident>, String>
create_incident(config: &ServiceNowConfig, short_description: &str, description: &str, urgency: &str, impact: &str) -> Result<TicketResult, String>
get_incident(config: &ServiceNowConfig, incident_id: &str) -> Result<Incident, String>
update_incident(config: &ServiceNowConfig, sys_id: &str, updates: serde_json::Value) -> Result<TicketResult, String>
```

### Configuration (Settings → Integrations → ServiceNow)
```
Instance URL:     https://yourorg.service-now.com
Username:         admin
Password:         (encrypted with AES-256-GCM)
```

### Implementation Details
- **API**: ServiceNow Table API (`/api/now/table/incident`)
- **Auth**: HTTP Basic authentication
- **Severity mapping**: TRCAA P1-P4 → ServiceNow urgency/impact (1-3)
- **Incident lookup**: Supports both sys_id (UUID) and incident number (INC0010001)
- **TDD Tests**: 7 tests with mockito HTTP mocking

---

## Azure DevOps

**Purpose:** Create and manage work items (bugs/tasks) in Azure DevOps.

**Status:** ✅ **Implemented** (v0.2.3)

### Features
- OAuth2 authentication with PKCE flow
- Search work items via WIQL queries
- Create work items (Bug, Task, User Story)
- Get work item details by ID
- Update work items with JSON-PATCH operations

### API Client (`src-tauri/src/integrations/azuredevops.rs`)

**Functions:**
```rust
test_connection(config: &AzureDevOpsConfig) -> Result<ConnectionResult, String>
search_work_items(config: &AzureDevOpsConfig, query: &str) -> Result<Vec<WorkItem>, String>
create_work_item(config: &AzureDevOpsConfig, title: &str, description: &str, work_item_type: &str, severity: &str) -> Result<TicketResult, String>
get_work_item(config: &AzureDevOpsConfig, work_item_id: i64) -> Result<WorkItem, String>
update_work_item(config: &AzureDevOpsConfig, work_item_id: i64, updates: serde_json::Value) -> Result<TicketResult, String>
```

### Configuration (Settings → Integrations → Azure DevOps)
```
Organization URL: https://dev.azure.com/yourorg
Authentication:   OAuth2 (bearer token, encrypted at rest)
Project:          MyProject
```

### Implementation Details
- **API**: Azure DevOps REST API v7.0
- **Auth**: OAuth2 bearer token (encrypted with AES-256-GCM)
- **WIQL**: Work Item Query Language for advanced search
- **Work item types**: Bug, Task, User Story, Issue, Incident
- **Severity mapping**: Bug-specific field `Microsoft.VSTS.Common.Severity`
- **TDD Tests**: 6 tests with mockito HTTP mocking

---

## OAuth2 Authentication Flow

All integrations using OAuth2 (Confluence, Azure DevOps) follow the same flow:

1. **User clicks "Connect"** in Settings → Integrations
2. **Backend generates PKCE challenge** and stores code verifier
3. **Local callback server starts** on `http://localhost:8765`
4. **Browser opens** with OAuth authorization URL
5. **User authenticates** with service provider
6. **Service redirects** to `http://localhost:8765/callback?code=...`
7. **Callback server extracts code** and triggers token exchange
8. **Backend exchanges code for token** using PKCE verifier
9. **Token encrypted** with AES-256-GCM and stored in DB
10. **UI shows "Connected"** status

**Implementation:**
- `src-tauri/src/integrations/auth.rs` — PKCE generation, token exchange, encryption
- `src-tauri/src/integrations/callback_server.rs` — Local HTTP server (warp)
- `src-tauri/src/commands/integrations.rs` — IPC command handlers

**Security:**
- Tokens encrypted at rest with AES-256-GCM (256-bit key)
- Key derived from environment variable `TRCAA_DB_KEY` (or legacy `TRCAA_DB_KEY`)
- PKCE prevents authorization code interception
- Callback server only accepts from `localhost`

---

## Database Schema

**Credentials Table (`migration 011`):**
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

**Integration Config Table:**
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

---

## Testing

All integrations have comprehensive test coverage:

```bash
# Run all integration tests
cargo test --manifest-path src-tauri/Cargo.toml --lib integrations

# Run specific integration tests
cargo test --manifest-path src-tauri/Cargo.toml confluence
cargo test --manifest-path src-tauri/Cargo.toml servicenow
cargo test --manifest-path src-tauri/Cargo.toml azuredevops
```

**Test statistics:**
- **Confluence**: 6 tests (connection, spaces, search, publish, update)
- **ServiceNow**: 7 tests (connection, search, create, get by sys_id, get by number, update)
- **Azure DevOps**: 6 tests (connection, WIQL search, create, get, update)
- **Total**: 19 integration tests (all passing)

**Test approach:**
- TDD methodology (tests written first)
- HTTP mocking with `mockito` crate
- No external API calls in tests
- All auth flows tested with mock responses

---

## CSP Configuration

All integration domains are whitelisted in `src-tauri/tauri.conf.json`:

```json
"connect-src": "... https://auth.atlassian.com https://*.atlassian.net https://login.microsoftonline.com https://dev.azure.com"
```

---

## Adding a New Integration

1. **Create API client**: `src-tauri/src/integrations/{name}.rs`
2. **Implement functions**: `test_connection()`, create/read/update operations
3. **Add TDD tests**: Use `mockito` for HTTP mocking
4. **Update migration**: Add service to `credentials` and `integration_config` CHECK constraints
5. **Add IPC commands**: `src-tauri/src/commands/integrations.rs`
6. **Update CSP**: Add API domains to `tauri.conf.json`
7. **Wire up UI**: `src/pages/Settings/Integrations.tsx`
8. **Update capabilities**: Add any required Tauri permissions
9. **Document**: Update this wiki page

---

## Troubleshooting

### OAuth "Command plugin:shell|open not allowed"
**Fix**: Add `"shell:allow-open"` to `src-tauri/capabilities/default.json`

### Token Exchange Fails
**Check**:
1. PKCE verifier matches challenge
2. Redirect URI exactly matches registered callback
3. Authorization code hasn't expired
4. Client ID/secret are correct

### ServiceNow 401 Unauthorized
**Check**:
1. Username/password are correct
2. User has API access enabled
3. Instance URL is correct (no trailing slash)

### Confluence API 404
**Check**:
1. Base URL format: `https://yourorg.atlassian.net` (no `/wiki/`)
2. Space key exists and user has access
3. OAuth token has required scopes (`read:confluence-content.all`, `write:confluence-content`)

### Azure DevOps 403 Forbidden
**Check**:
1. OAuth token has required scopes (`vso.work_write`)
2. User has permissions in the project
3. Project name is case-sensitive
