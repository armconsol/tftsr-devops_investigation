# MCP Servers

## Overview

**Model Context Protocol (MCP)** is an open standard that allows AI models to invoke external tools and access external resources through a standardised JSON-RPC interface. TFTSR integrates MCP as a first-class feature, enabling the AI triage assistant to call tools exposed by any compliant MCP server — file search, database queries, monitoring APIs, runbook automation, and more.

MCP support extends the AI's capabilities beyond conversation: during incident triage, the model can autonomously invoke registered tools to gather diagnostic data, check system status, or execute remediation steps — all within the app's security and audit framework.

---

## Architecture

```
┌──────────────────────────────────────────────┐
│  TFTSR App                                   │
│                                              │
│  ┌────────┐   ┌──────────┐   ┌───────────┐  │
│  │Frontend│──▶│ Commands │──▶│  Store    │  │
│  │  React │   │(IPC/Tauri)│   │ (SQLite)  │  │
│  └────────┘   └────┬─────┘   └───────────┘  │
│                    │                         │
│              ┌─────▼─────┐                   │
│              │ Discovery │                   │
│              └─────┬─────┘                   │
│                    │                         │
│       ┌────────────┼────────────┐            │
│       │            │            │            │
│  ┌────▼────┐  ┌────▼────┐  ┌───▼────┐       │
│  │  stdio  │  │  HTTP   │  │Adapter │       │
│  │Transport│  │Transport│  │(AI glue)│       │
│  └────┬────┘  └────┬────┘  └────────┘       │
└───────┼─────────────┼────────────────────────┘
        │             │
        ▼             ▼
  Local process   Remote HTTP
  (e.g. npx)     MCP endpoint
```

**Module layout** (`src-tauri/src/mcp/`):

| File | Responsibility |
|------|----------------|
| `models.rs` | Struct definitions: `McpServer`, `McpTool`, `McpResource`, request types |
| `store.rs` | CRUD operations against SQLite (encrypted at rest) |
| `transport/stdio.rs` | Stdio process spawn via `rmcp` (absolute path enforced) |
| `transport/http.rs` | Streamable HTTP transport via `rmcp` |
| `client.rs` | Connection lifecycle, tool listing, tool invocation |
| `adapter.rs` | Name sanitisation, `McpTool` → AI `Tool` conversion |
| `discovery.rs` | Per-server and bulk startup discovery orchestration |
| `commands.rs` | 8 Tauri IPC command handlers |

---

## Database Schema

Three tables are created by **Migration 018** (`018_mcp_servers`):

### `mcp_servers`

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | TEXT | PRIMARY KEY |
| `name` | TEXT | NOT NULL |
| `url` | TEXT | NOT NULL |
| `transport_type` | TEXT | NOT NULL, CHECK IN (`'stdio'`, `'http'`) |
| `transport_config` | TEXT | NOT NULL DEFAULT `'{}'` (JSON) |
| `auth_type` | TEXT | NOT NULL, CHECK IN (`'none'`, `'api_key'`, `'bearer'`, `'oauth2'`) |
| `auth_value` | TEXT | Nullable — AES-256-GCM encrypted |
| `enabled` | INTEGER | NOT NULL DEFAULT 1 |
| `last_discovered_at` | TEXT | Nullable UTC timestamp |
| `discovery_status` | TEXT | NOT NULL DEFAULT `'pending'`, CHECK IN (`'pending'`, `'connected'`, `'unreachable'`, `'error'`) |
| `discovery_error` | TEXT | Nullable |
| `created_at` | TEXT | NOT NULL DEFAULT `datetime('now')` |
| `updated_at` | TEXT | NOT NULL DEFAULT `datetime('now')` |

### `mcp_tools`

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | TEXT | PRIMARY KEY |
| `server_id` | TEXT | NOT NULL, FK → `mcp_servers(id)` ON DELETE CASCADE |
| `name` | TEXT | NOT NULL (original tool name from server) |
| `tool_key` | TEXT | NOT NULL (sanitised key used by AI) |
| `description` | TEXT | Nullable |
| `parameters` | TEXT | NOT NULL DEFAULT `'{}'` (JSON Schema) |

### `mcp_resources`

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | TEXT | PRIMARY KEY |
| `server_id` | TEXT | NOT NULL, FK → `mcp_servers(id)` ON DELETE CASCADE |
| `uri` | TEXT | NOT NULL |
| `name` | TEXT | Nullable |
| `description` | TEXT | Nullable |

Cascade deletes ensure that removing a server automatically cleans up its tools and resources.

---

## Transport Types

### stdio

The app spawns a local process and communicates over its stdin/stdout using the MCP JSON-RPC protocol.

**Configuration** (`transport_config` JSON):
```json
{
  "command": "/usr/local/bin/my-mcp-server",
  "args": ["--port", "0", "--mode", "stdio"]
}
```

- `command` — **must be an absolute path**. Relative paths are rejected to prevent path traversal attacks.
- `args` — optional array of command-line arguments.

The process is spawned via Tokio and wrapped with `rmcp::transport::TokioChildProcess`.

### http (Streamable HTTP)

The app connects to a remote MCP server over HTTP(S) using the Streamable HTTP transport from `rmcp`.

**Configuration:**
- `url` field on the server record — the HTTP endpoint (e.g., `https://mcp.example.com/v1`).
- If `auth_type` is `bearer` or `api_key`, the decrypted auth value is attached as an `Authorization` header.

```json
{
  "url": "https://mcp.example.com/v1",
  "transport_type": "http",
  "auth_type": "bearer"
}
```

The `transport_config` field for HTTP servers is typically `{}` — connection details come from `url` and `auth_value`.

---

## Authentication Types

| Type | Description | Storage |
|------|-------------|---------|
| `none` | No authentication required | — |
| `api_key` | API key sent as Authorization header | Encrypted in `auth_value` |
| `bearer` | Bearer token sent as Authorization header | Encrypted in `auth_value` |
| `oauth2` | OAuth2 PKCE flow via WebView | Token encrypted in `auth_value` after exchange |

All auth values are encrypted with **AES-256-GCM** before storage (same encryption system as integration credentials). The plaintext is never returned to the frontend — `list_mcp_servers` strips `auth_value` from responses.

### OAuth2 Flow

For servers requiring OAuth2:

1. `transport_config` must include `auth_endpoint`, `token_endpoint`, `client_id`, and optionally `scope`.
2. Call `initiate_mcp_oauth(server_id)` — opens a WebView window at the authorization URL.
3. User authenticates with the MCP provider.
4. On redirect, the code is exchanged for an access token.
5. Token is encrypted and stored in `auth_value`.

---

## Configuration Guide

### Adding an MCP Server (UI)

Navigate to **Settings > MCP Servers** (`/settings/mcp`) to manage servers.

1. Click **Add Server**.
2. Fill in:
   - **Name** — Human-readable label (e.g., "Weather API", "Filesystem Tools").
   - **URL** — For HTTP: the server endpoint. For stdio: can be left as the command path for display.
   - **Transport** — `stdio` or `http`.
   - **Transport Config** — JSON. For stdio: `{"command": "/path/to/binary", "args": [...]}`. For HTTP: typically `{}`.
   - **Auth Type** — `none`, `api_key`, `bearer`, or `oauth2`.
   - **Auth Value** — The token/key (will be encrypted on save). Leave blank for `none`.
   - **Enabled** — Toggle on/off.
3. Click **Save**. The server record is persisted.
4. Click **Discover** to connect and enumerate available tools and resources.

### Discovery

Discovery connects to the server, queries its tool and resource manifests, and persists them locally. Status transitions:

```
pending → connected     (success)
pending → error         (connection/protocol failure)
pending → unreachable   (startup failure, non-fatal)
```

After successful discovery, tools from the server appear in AI conversations automatically.

---

## Tool Naming Convention

When tools are discovered, each gets a **tool key** used by the AI model:

```
mcp_{server_name}_{tool_name}
```

Both parts are sanitised:
- Lowercased
- Non-alphanumeric characters replaced with `_`
- Consecutive underscores collapsed
- Leading/trailing underscores trimmed

**Examples:**

| Server Name | Tool Name | Tool Key |
|-------------|-----------|----------|
| My Weather API | get_forecast | `mcp_my_weather_api_get_forecast` |
| Filesystem | search files | `mcp_filesystem_search_files` |
| simple | ping | `mcp_simple_ping` |

The AI model calls tools by their `tool_key`. The adapter layer resolves this back to the original server and tool name for execution.

---

## Startup Discovery

On application launch, `init_all_servers()` iterates all **enabled** servers and attempts discovery for each:

- Successful connections are stored in `AppState.mcp_connections` (a `HashMap<String, Arc<TokioMutex<McpConnection>>>>`).
- Failed connections are marked as `unreachable` in the database with the error message. A warning is logged, but startup continues.
- This is a best-effort, non-blocking operation — the app launches regardless of MCP server availability.

---

## AI Integration

Enabled MCP tools are automatically injected into AI conversations:

1. `get_enabled_mcp_tools()` queries tools from servers that are both `enabled = 1` and `discovery_status = 'connected'`.
2. Each `McpTool` is converted to an AI `Tool` definition (name, description, JSON Schema parameters).
3. When the AI responds with a tool call matching an `mcp_*` key, the adapter routes it to `call_tool()` on the appropriate live connection.
4. The tool result is fed back to the AI as a tool response message.

---

## IPC Commands

| Command | Parameters | Returns |
|---------|-----------|---------|
| `list_mcp_servers` | — | `McpServer[]` (auth_value always null) |
| `create_mcp_server` | `CreateMcpServerRequest` | `McpServer` |
| `update_mcp_server` | `id`, `UpdateMcpServerRequest` | `McpServer` |
| `delete_mcp_server` | `id` | `void` |
| `toggle_mcp_server` | `id`, `enabled` | `void` |
| `discover_mcp_server` | `id` | `McpServerStatus` |
| `get_mcp_server_status` | `id` | `McpServerStatus` |
| `initiate_mcp_oauth` | `id` | `void` (opens WebView) |

See [IPC Commands](IPC-Commands#mcp-servers) for full type signatures.

---

## Security

- **Encrypted auth values** — AES-256-GCM, same key derivation as integration credentials (`TFTSR_ENCRYPTION_KEY`)
- **Server-side scrubbing** — `auth_value` set to `None` before any response to the frontend
- **Audit logging** — `write_audit_event` called before every MCP tool execution
- **PII scan** — Tool call arguments are scanned for PII patterns (non-blocking warning to user)
- **Absolute path enforcement** — stdio transport rejects relative paths to prevent traversal attacks
- **Cascade deletes** — Removing a server removes all associated tools and resources
- **TLS** — HTTP transport uses `reqwest` with certificate verification for HTTPS endpoints

See [Security Model](Security-Model#mcp-server-security) for the full threat analysis.
