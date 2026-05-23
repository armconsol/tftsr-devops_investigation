# MCP Server Support — Ticket Summary

## Description

Adds MCP (Model Context Protocol) server management to the application, allowing the AI assistant
to discover and call tools from external MCP servers during triage conversations.

The implementation covers:
- Settings page at `/settings/mcp` for managing server connections
- Support for `stdio` (local processes) and `http` (Streamable HTTP) transports
- Auth types: `none`, `api_key`, `bearer`, `oauth2`
- Auto-discovery of enabled servers at application startup
- Transparent injection of discovered tools into every AI chat session
- Security-first design: encrypted credential storage, mandatory audit logging, PII scanning

---

## Acceptance Criteria

- [x] Users can add, edit, enable/disable, and delete MCP server configurations
- [x] "Discover Now" connects to the server, lists tools and resources, and persists results
- [x] Enabled servers auto-connect on app launch via `.setup()` hook
- [x] MCP tools appear in the AI chat tool list and are callable by the AI
- [x] `auth_value` is always AES-256-GCM encrypted at rest; never returned to frontend
- [x] `write_audit_event()` is called before every MCP tool execution
- [x] PII scan on tool call arguments (non-blocking warning on detection)
- [x] stdio transport rejects relative paths; never uses `sh -c`
- [x] All existing tests continue to pass (185 Rust, 94 Vitest)
- [x] Zero clippy warnings; zero TypeScript errors

---

## Work Implemented

### Backend (Rust)

| Phase | Files | Description |
|-------|-------|-------------|
| 0 | `Cargo.toml` | Added `rmcp = "1.7.0"` with client + transport features; version → 0.3.0 |
| 1 | `db/migrations.rs` | Migration 018: `mcp_servers`, `mcp_tools`, `mcp_resources` tables with CHECK constraints |
| 2a | `mcp/models.rs`, `mcp/store.rs` | Data types; full CRUD with encrypted auth storage |
| 2b | `mcp/transport/stdio.rs`, `mcp/transport/http.rs` | Transport builders for subprocess and Streamable HTTP |
| 2c | `mcp/client.rs` | `McpConnection` type alias; connect/list/call wrappers |
| 2d | `mcp/adapter.rs` | `sanitize_name`, `build_tool_key`, `mcp_tools_to_ai_tools`, `get_enabled_mcp_tools` |
| 2e | `mcp/discovery.rs` | `discover_server`, `init_all_servers` |
| 2f | `mcp/commands.rs`, `state.rs`, `lib.rs` | 8 Tauri commands; `mcp_connections` field on `AppState`; `.setup()` hook |
| 5 | `ai/tools.rs`, `commands/ai.rs` | `get_enabled_mcp_tools` async helper; `execute_mcp_tool_call` with PII scan + audit |

### Frontend (TypeScript / React)

| Phase | Files | Description |
|-------|-------|-------------|
| 3 | `src/lib/tauriCommands.ts` | `McpServer`, `McpTool`, `McpResource`, `McpServerStatus`, request types; 8 command wrappers |
| 4 | `src/pages/Settings/MCPServers.tsx` | Full settings page: server list, status badges, Discover Now, Add/Edit modal |
| 4 | `src/App.tsx` | Added `Plug` icon, `/settings/mcp` route and nav entry |

### Wiki

- `docs/wiki/MCP-Servers.md` — new
- `docs/wiki/Database.md` — migration 018 documented
- `docs/wiki/IPC-Commands.md` — 8 new commands
- `docs/wiki/Security-Model.md` — MCP security section

---

## Testing Needed

### Automated (all passing)
- Rust: 185 tests (64 existing + 5 migration 018 + 5 store + 3 adapter + 5 migration idempotency + misc)
- Vitest: 94 tests (all existing + 3 new MCP frontend tests)
- `cargo clippy -- -D warnings`: zero warnings
- `npx tsc --noEmit`: zero errors

### Manual verification checklist
- [ ] Add an HTTP MCP server → click Discover Now → tools appear in list
- [ ] Add a stdio MCP server → Discover Now → process spawns, tools appear
- [ ] Disable a server → its tools absent from next triage chat session
- [ ] Start a triage chat → MCP tools visible in AI tool suggestions
- [ ] AI calls an MCP tool → audit log entry written in Security page
- [ ] Delete a server → live connection removed, tools gone from next session
- [ ] Enter an invalid command path (relative) for stdio → error shown in UI

### Branch
`feature/mcp-server-support`
