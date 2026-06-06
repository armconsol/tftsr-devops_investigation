# ADR-008: Model Context Protocol for External Tools

**Date**: 2026-06-02  
**Status**: Accepted  
**Deciders**: Shaun Arman, Henry Castle  
**Context**: Hackathon v1.0.0 — Extensible Tool Integration

---

## Context

TFTSR DevOps Investigation v1.0.0 introduced agentic shell execution with statically-defined tools (`execute_shell_command`, `add_ado_comment`). As the application grows, we need a way to integrate external tools and services without hardcoding every integration into the Rust backend.

**Requirements**:
- AI agents need access to third-party tools (GitHub, Slack, monitoring systems, etc.)
- Tool definitions should be discoverable and documented
- Tool execution should be sandboxed and timeout-protected
- New tools should be addable without recompiling the application
- Support both local processes (stdio) and remote services (HTTP)

**Alternatives Considered**:

1. **Plugin system (dynamic library loading)**
   - ✅ Native Rust plugins with full system access
   - ❌ Security risk — malicious plugins have full process access
   - ❌ Unsafe Rust (`dlopen`, FFI) for plugin loading
   - ❌ Platform-specific (.so, .dylib, .dll)
   - ❌ No sandboxing

2. **WebAssembly plugins (wasmtime)**
   - ✅ Sandboxed execution with WASI
   - ✅ Cross-platform (single .wasm file)
   - ❌ Complex WASI interface design
   - ❌ WASI preview2 still unstable
   - ❌ Limited async support

3. **gRPC tool server protocol**
   - ✅ Industry-standard RPC
   - ✅ Strongly typed with protobuf
   - ❌ Complex setup for simple tools
   - ❌ Every tool server needs gRPC boilerplate
   - ❌ No existing ecosystem

4. **Model Context Protocol (MCP)**
   - ✅ Designed specifically for AI tool integration
   - ✅ Existing ecosystem (Anthropic, community servers)
   - ✅ Supports stdio (local processes) and HTTP (remote services)
   - ✅ JSON-RPC 2.0 protocol (simple, well-understood)
   - ✅ Tool discovery built into protocol
   - ❌ New protocol (May 2024), potential churn

---

## Decision

**Adopt the Model Context Protocol (MCP) for external tool integration, using the `rmcp` Rust client library.**

### Architecture

```
AI Agent → MCP Adapter → MCP Client → Transport (stdio/HTTP) → MCP Server
                                                                   ↓
                                                              External Tool
```

**Components**:

| Module | Responsibility |
|--------|---------------|
| `mcp/client.rs` | Connect to MCP servers (stdio/HTTP) |
| `mcp/adapter.rs` | Merge MCP tools with static tools |
| `mcp/discovery.rs` | Health check servers, update status |
| `mcp/store.rs` | Persist server configs and tools to database |
| `mcp/models.rs` | McpServer, McpTool, McpResource types |
| `mcp/transport/stdio.rs` | Spawn processes with env vars |
| `mcp/transport/http.rs` | HTTP POST with auth headers |

**Database Schema** (Migration 018):

```sql
CREATE TABLE mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    transport_type TEXT NOT NULL CHECK(transport_type IN ('stdio', 'http')),
    auth_type TEXT NOT NULL CHECK(auth_type IN ('none', 'api_key', 'bearer', 'oauth2')),
    auth_value TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    discovery_status TEXT NOT NULL DEFAULT 'pending'
        CHECK(discovery_status IN ('pending','connected','unreachable','error')),
    env_config TEXT, -- JSON map of environment variables
    ...
);

CREATE TABLE mcp_tools (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    name TEXT NOT NULL,
    tool_key TEXT NOT NULL, -- "server_name.tool_name"
    description TEXT,
    parameters TEXT NOT NULL, -- JSON schema
    FOREIGN KEY(server_id) REFERENCES mcp_servers(id) ON DELETE CASCADE
);
```

**Tool Calling Flow**:

1. User configures MCP server in Settings (name, URL/command, transport type, auth)
2. Application connects and calls `list_tools()` to discover available tools
3. Tools stored in `mcp_tools` table with namespaced key (`server_name.tool_name`)
4. AI agent requests tools via `get_enabled_mcp_tools()`
5. MCP tools merged with static tools (`execute_shell_command`, `add_ado_comment`)
6. AI agent calls tool by key (e.g., `github.create_issue`)
7. Adapter routes to correct MCP client
8. Client invokes tool with **30-second hard timeout**
9. Result returned to AI agent

**Safety Features**:

- **Timeout protection**: 30-second hard timeout prevents indefinite hangs from misbehaving servers
- **Process isolation**: Stdio servers run as separate processes with isolated env vars
- **Auth encryption**: API keys encrypted with AES-256-GCM before storage
- **User control**: Users explicitly enable/disable each MCP server
- **Status tracking**: Connection health displayed in UI (connected, unreachable, error)

---

## Consequences

### Positive

- **Extensibility**: New tools without recompiling (add MCP server in Settings)
- **Ecosystem**: Can use community MCP servers (GitHub, Slack, Prometheus, etc.)
- **Simplicity**: JSON-RPC 2.0 protocol is simple to implement and debug
- **Dual transport**: Supports both local tools (stdio) and cloud services (HTTP)
- **Discovery**: Tool schemas fetched automatically via `list_tools()`
- **Sandboxing**: Stdio processes isolated, HTTP calls timeout-protected

### Negative

- **Protocol churn risk**: MCP is new (May 2024), spec may evolve
- **Dependency**: Relies on `rmcp` crate maintenance
- **Stdio complexity**: Process spawning platform-dependent (Windows cmd.exe vs Unix bash)
- **Debugging**: Tool call failures require inspecting both application logs and MCP server logs

### Trade-offs

We chose **extensibility and ecosystem over protocol maturity**. MCP's design aligns with our use case (AI tool calling), and the 30-second timeout mitigates the risk of server misbehavior.

---

## Implementation Notes

**Example: Stdio MCP Server**

```bash
# User configures in Settings UI:
Name: GitHub Tools
Transport: stdio
Command: npx
Args: @modelcontextprotocol/server-github
Env: GITHUB_TOKEN=ghp_...
```

Application spawns process, sends JSON-RPC 2.0 requests over stdin/stdout:

```json
{"jsonrpc":"2.0","method":"tools/list","id":1}
```

Server responds:

```json
{
  "jsonrpc":"2.0",
  "id":1,
  "result":{
    "tools":[
      {"name":"create_issue","description":"Create a GitHub issue","inputSchema":{...}},
      {"name":"list_commits","description":"List commits","inputSchema":{...}}
    ]
  }
}
```

**Example: HTTP MCP Server**

```bash
# User configures:
Name: Internal Monitoring
Transport: http
URL: https://monitoring.internal.com/mcp
Auth Type: bearer
Auth Value: eyJ...
```

Application sends HTTP POST to `/mcp` with `Authorization: Bearer eyJ...` header.

---

## Related Decisions

- **ADR-007**: Three-Tier Shell Safety (MCP tools bypass shell classification — server responsibility)
- Future: **ADR-010**: MCP Tool Approval System (extend three-tier safety to MCP tools)

---

## References

- **MCP Specification**: https://spec.modelcontextprotocol.io/
- **rmcp Rust Client**: https://github.com/tankeez/rmcp
- **Implementation PR**: #32 (Hackathon v1.0.0)
- **Database Schema**: Migration 018 (`mcp_servers`, `mcp_tools`, `mcp_resources`)
- **Wiki**: `docs/wiki/AI-Providers.md` (Tool Calling section)
