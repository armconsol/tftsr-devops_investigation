# Architecture

## Overview

TRCAA uses a Tauri 2.x architecture: a Rust backend runs natively, and a React/TypeScript frontend runs in an embedded WebView. Communication between them happens exclusively via typed IPC (`invoke()`).

```
┌─────────────────────────────────────────┐
│            WebView (React)               │
│  pages/ → stores/ → tauriCommands.ts    │
└──────────────────┬──────────────────────┘
                   │  invoke() / IPC
┌──────────────────▼──────────────────────┐
│            Rust Backend (Tauri)          │
│  commands/ → ai/ → pii/ → db/ → docs/  │
└─────────────────────────────────────────┘
              │          │
        SQLCipher      reqwest
           DB        (AI APIs)
```

## Backend — Rust

**Entry point:** `src-tauri/src/lib.rs` → `run()` initialises tracing, opens the DB, registers Tauri plugins, and calls `generate_handler![]` with all IPC commands.

**Backend logging:** tracing now writes to both console and local disk at
`<app_data_dir>/logs/backend.log` (daily rolling file). Default level is `info`;
`debug` can be enabled at runtime via settings (`debug_logging_enabled`).
Current retention behavior is daily rotation only (no fixed max-days purge yet).

### Shared State

```rust
pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub settings: Arc<Mutex<AppSettings>>,
    pub app_data_dir: PathBuf,   // ~/.local/share/tftsr on Linux
}
```

All command handlers receive `State<'_, AppState>` as a Tauri-injected parameter. The Mutex must be **released before any `.await`** — holding a `MutexGuard` across an await point is a compile error because `MutexGuard` is not `Send`.

### Module Layout

| Path | Responsibility |
|------|---------------|
| `lib.rs` | App entry, tracing init, DB setup, plugin registration, command handler list |
| `state.rs` | `AppState` struct |
| `commands/db.rs` | Issue CRUD, 5-Whys entries, timeline events |
| `commands/ai.rs` | `analyze_logs`, `chat_message`, `list_providers` |
| `commands/analysis.rs` | Log file upload, PII detection, redaction |
| `commands/docs.rs` | RCA and post-mortem generation, document export |
| `commands/system.rs` | Ollama management, hardware probe, settings, audit log |
| `commands/image.rs` | Image attachment upload, list, delete, paste |
| `commands/database.rs` | Database connections, query history, bookmarks, import/export |
| `commands/kube.rs` | Kubernetes clusters, resources, port forwards, events |
| `commands/metrics.rs` | Shared metrics and telemetry helpers |
| `commands/remote.rs` | RDP and remote session management |
| `commands/shell.rs` | Shell execution, approvals, kubeconfig handling |
| `commands/table_browser.rs` | Table browser and data exploration |
| `commands/integrations.rs` | Confluence / ServiceNow / ADO integration handlers |
| `ai/provider.rs` | `Provider` trait + `create_provider()` factory |
| `pii/detector.rs` | Multi-pattern PII scanner with overlap resolution |
| `db/migrations.rs` | Versioned schema (23 migrations in `_migrations` table) |
| `db/models.rs` | All DB types — see `IssueDetail` note below |
| `docs/rca.rs` + `docs/postmortem.rs` | Markdown template builders |
| `audit/log.rs` | `write_audit_event()` — called before every external send |
| `proxmox/lxc.rs` | LXC container config retrieval and creation |
| `proxmox/ceph.rs` | Ceph pool/OSD/monitor/manager/filesystem operations (node-scoped `nodes/{node}/ceph/*` endpoints) |
| `proxmox/tfa.rs` | Two-factor authentication entry management |
| `proxmox/pbs.rs` | Proxmox Backup Server datastore, namespace, snapshot, and task queries |
| `proxmox/validate.rs` | Shared input validation (node names, vmid range) used across proxmox modules |

### Directory Structure

```
src-tauri/src/
├── lib.rs
├── main.rs
├── state.rs
├── ai/
│   ├── provider.rs        # Provider trait + factory
│   ├── openai.rs
│   ├── anthropic.rs
│   ├── gemini.rs
│   ├── mistral.rs
│   └── ollama.rs
├── commands/
│   ├── db.rs
│   ├── ai.rs
│   ├── analysis.rs
│   ├── docs.rs
│   ├── system.rs
│   ├── image.rs
│   ├── integrations.rs
│   ├── database.rs
│   ├── kube.rs
│   ├── metrics.rs
│   ├── remote.rs
│   ├── shell.rs
│   └── table_browser.rs
├── pii/
│   ├── patterns.rs
│   ├── detector.rs
│   └── redactor.rs
├── db/
│   ├── connection.rs
│   ├── migrations.rs
│   └── models.rs
├── docs/
│   ├── rca.rs
│   ├── postmortem.rs
│   └── exporter.rs
├── audit/
│   └── log.rs
├── ollama/
│   ├── installer.rs
│   ├── manager.rs
│   ├── recommender.rs
│   └── hardware.rs
├── mcp/
│   ├── discovery.rs
│   ├── servers.rs
│   └── tools.rs
├── ollama/
│   ├── installer.rs
│   ├── manager.rs
│   ├── recommender.rs
│   └── hardware.rs
├── proxmox/
├── remote/
├── shell/
├── database/
└── integrations/
    ├── confluence.rs
    ├── servicenow.rs
    └── azuredevops.rs
```

## Frontend — React/TypeScript

**IPC layer:** All Tauri `invoke()` calls are in `src/lib/tauriCommands.ts`. Every command has a typed wrapper. This is the single source of truth for the frontend API surface.

### Stores (Zustand)

| Store | Persistence | Contents |
|-------|------------|----------|
| `sessionStore.ts` | Not persisted (ephemeral) | currentIssue, messages, piiSpans, approvedRedactions, whyLevel (0–5), loading state |
| `settingsStore.ts` | `localStorage` as `"trcaa-settings"` | AI providers, theme, Ollama URL, active provider, debug logging toggle |
| `historyStore.ts` | Not persisted (cache) | Past issues list, search query |
| `proxmoxStore.ts` | `localStorage` as `"tftsr-proxmox"` | Selected Proxmox cluster id + selected node per cluster (`Record<clusterId, node>`), shared across every Proxmox subpage so switching pages no longer resets the host dropdown back to the default |

### Shared Proxmox frontend utilities

- `src/lib/format.ts` — `formatBytes` (B → PB, used by Ceph pool/OSD tables so Used/Available
  render as e.g. `1.5 TB` instead of raw byte counts) and `formatUptime`.
- `src/hooks/usePolling.ts` — fires a callback immediately then on a fixed interval while
  enabled; used to auto-load and periodically refresh the Ceph Monitors/Managers/CephFS/Flags
  tabs (they previously only loaded once via a manual "Load" button).
- `src/hooks/useProxmoxClusters.ts` / `src/hooks/useProxmoxNodes.ts` — load the cluster/node
  list and restore the selection from `proxmoxStore` (falling back to the first entry if the
  persisted id/name is no longer present), instead of every page defaulting back to index 0.
- `src/components/SidebarNav.tsx` — recursive sidebar renderer (groups with `children` are
  collapsible buttons, leaves are `NavLink`s) backing the top-level **Tools** group
  (Kubernetes, Proxmox, Database, Remote Desktop — collapsed by default) and the **Settings**
  group (also collapsed by default), replacing the previous single-level, always-expanded
  Settings list in `App.tsx`.

### Page Flow

```
NewIssue → createIssueCmd → startSession(detail.issue) → navigate /issue/:id/triage
LogUpload → uploadLogFileCmd → detectPiiCmd → applyRedactionsCmd
Triage   → chatMessageCmd loop → auto-detect why levels 1–5
Resolution → getIssueCmd → mark 5-Whys steps done
RCA      → generateRcaCmd → DocEditor → exportDocumentCmd
```

### Directory Structure

```
src/
├── main.tsx
├── App.tsx
├── components/
│   ├── ChatWindow.tsx
│   ├── TriageProgress.tsx
│   ├── PiiDiffViewer.tsx
│   ├── DocEditor.tsx
│   ├── HardwareReport.tsx
│   ├── ModelSelector.tsx
│   └── ui/index.tsx       # Custom components (Card, Button, Input, etc.)
├── pages/
│   ├── Dashboard/
│   ├── NewIssue/
│   ├── LogUpload/
│   ├── Triage/
│   ├── Resolution/
│   ├── RCA/
│   ├── Postmortem/
│   ├── History/
│   └── Settings/
│       ├── AIProviders.tsx
│       ├── Ollama.tsx
│       ├── Integrations.tsx
│       └── Security.tsx
├── stores/
│   ├── sessionStore.ts
│   ├── settingsStore.ts
│   └── historyStore.ts
└── lib/
    ├── tauriCommands.ts
    └── domainPrompts.ts
```

## Key Type: IssueDetail

`get_issue()` returns a **nested** struct, not flat. Always use `detail.issue.*`:

```rust
pub struct IssueDetail {
    pub issue: Issue,                          // Base fields (title, severity, etc.)
    pub log_files: Vec<LogFile>,
    pub resolution_steps: Vec<ResolutionStep>, // 5-Whys entries
    pub conversations: Vec<AiConversation>,
}
```

Use `detail.issue.title`, **not** `detail.title`.

## Incident Response Methodology

The application integrates a comprehensive incident response framework via system prompt injection. The `INCIDENT_RESPONSE_FRAMEWORK` constant in `src/lib/domainPrompts.ts` is appended to all 15 domain-specific system prompts (Linux, Windows, Network, Kubernetes, Databases, Virtualization, Hardware, Observability, Telephony, Security, Public Safety, Application, Automation, HPE, Dell, Identity).

**5-Phase Framework:**

1. **Detection & Evidence Gathering** — Initial issue assessment, log collection, PII redaction
2. **Diagnosis & Hypothesis Testing** — AI-assisted analysis, pattern matching against known incidents
3. **Root Cause Analysis with 5-Whys** — Iterative questioning to identify underlying cause (steps 1–5)
4. **Resolution & Prevention** — Remediation planning and implementation
5. **Post-Incident Review** — Timeline-based blameless post-mortem and lessons learned

**System Prompt Injection:**

The `chat_message` command accepts an optional `system_prompt` parameter. If provided, it prepends domain expertise before the conversation history. If omitted, the framework selects the appropriate domain prompt based on the issue category. This allows:

- **Specialized expertise**: Different frameworks for Linux vs. Kubernetes vs. Network incidents
- **Flexible override**: Users can inject custom system prompts for cross-domain problems
- **Consistent methodology**: All 17 domain prompts follow the same 5-phase incident response structure

**Timeline Event Recording:**

Timeline events are recorded non-blockingly at key triage moments:

```
Issue Creation → triage_started
   ↓
Log Upload → log_uploaded (metadata: file_name, file_size)
   ↓
Why-Level Progression → why_level_advanced (metadata: from_level → to_level)
   ↓
Root Cause Identified → root_cause_identified (metadata: root_cause, confidence)
   ↓
RCA Generated → rca_generated (metadata: doc_id, section_count)
   ↓
Postmortem Generated → postmortem_generated (metadata: doc_id, timeline_events_count)
   ↓
Document Exported → document_exported (metadata: format, file_path)
```

**Document Generation:**

RCA and Postmortem generators now use real timeline event data instead of placeholders:

- **RCA**: Incorporates timeline to show detection-to-root-cause progression
- **Postmortem**: Uses full timeline to demonstrate the complete incident lifecycle and response effectiveness

Timeline events are stored in the `timeline_events` table (indexed by issue_id and created_at for fast retrieval) and dual-written to `audit_log` for security/compliance purposes.

## Application Startup Sequence

```
1. Initialize tracing (RUST_LOG controls level)
2. Determine data directory (~/.local/share/tftsr or TRCAA_DATA_DIR)
3. Open / create SQLite database (run migrations)
4. Create AppState (db + settings + app_data_dir)
5. Register Tauri plugins (stronghold, dialog, fs, shell, http, cli, updater)
6. Register all 39 IPC command handlers
7. Start WebView with React app
```

## Image Attachments

The app supports uploading and managing image files (screenshots, diagrams) as attachments:

1. **Upload** via `upload_image_attachmentCmd()` or `upload_paste_imageCmd()` (clipboard paste)
2. **PII detection** runs automatically on upload
3. **User approval** required before image is stored
4. **Database storage** in `image_attachments` table with SHA-256 hash

## Data Flow

```
User Input
  ↓
[New Issue] ──── UUID assigned, stored in DB
  ↓
[Upload Log] ─── File read, SHA-256 hash computed, path stored
  ↓
[Detect PII] ─── 13 regex patterns applied, overlaps resolved
  ↓
[Review PII] ─── User approves/rejects each span
  ↓
[Apply Redactions] ─ Text rewritten, audit event logged
  ↓
[AI Chat] ──────── Domain system prompt injected
                   Redacted text sent to provider
                   Auto-detect why level from response
  ↓
[5-Whys] ───────── Answers stored as resolution_steps
  ↓
[Generate RCA] ─── Markdown from template + answers
  ↓
[Export] ────────── MD or PDF to user-chosen directory
```

## Remote Desktop (RDP)

The `remote/` module provides an in-app RDP client. Pixels are decoded in Rust
(IronRDP) and streamed to a `<canvas>` in the webview over a local WebSocket;
keyboard/mouse input flows back over the same socket.

### Module Layout

| Path | Responsibility |
|------|---------------|
| `remote/rdp.rs` | Session lifecycle (`start_session_async`): spawns the connect task, owns the per-session input channel + dispatch task |
| `remote/rdp_client.rs` | IronRDP connect/handshake, frame capture, input translation to fastpath PDUs |
| `remote/input.rs` | `RawInputEvent` wire type, JS `KeyboardEvent.code` → RDP scancode map, coordinate clamping |
| `remote/websocket_server.rs` | Local WS server: streams frames out, decodes JSON input frames in, per-session routing |
| `remote/ssh_tunnel.rs` | Optional SSH tunnel for the TCP connection |

### Pipeline

```
Browser <canvas>  ──WS binary frame [u32 LE w][u32 LE h][RGBA]──  websocket_server
        │                                                              ▲
        │ WS JSON text {type,...}                                      │ frame_rx (poll)
        ▼                                                              │
  websocket_server ──input mpsc──▶ rdp.rs dispatch ──▶ session.handle_input
                                                          ▼
                                            IronRDP fastpath input  ──▶  RDP host
```

### Key Implementation Notes

- **Frame forwarder uses polling, not `recv().await`.** Frames are produced by a
  blocking IronRDP `connect()` loop on a tokio worker. Channel wakeups land on
  that worker's local run-queue, which it never drains while blocked in a socket
  read, so an awaiting `recv()` never wakes. The forwarder instead polls
  `try_recv()` with a 5 ms async sleep, decoupling delivery from the starved
  waker.
- **Socket read-timeout duality.** The IronRDP blocking handshake does not
  tolerate `WouldBlock`/`TimedOut`, so a 30 s read timeout is used during
  negotiation, then lowered to 50 ms after `connect_finalize` so queued input is
  serviced promptly even when the server sends no graphics.
- **Slow-path graphics** (xrdp-style `ShareDataPdu::Update`) require
  ironrdp-session ≥ 0.10 (ironrdp 0.16 generation, MSRV Rust 1.89).
- **Input hardening.** The WS server caps messages at 4 KiB, validates the
  session id from the path, and drops input on channel saturation (`try_send`).
