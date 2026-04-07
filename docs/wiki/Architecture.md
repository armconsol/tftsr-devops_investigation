# Architecture

## Overview

TFTSR uses a Tauri 2.x architecture: a Rust backend runs natively, and a React/TypeScript frontend runs in an embedded WebView. Communication between them happens exclusively via typed IPC (`invoke()`).

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

### Shared State

```rust
pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub settings: Arc<Mutex<AppSettings>>,
    pub app_data_dir: PathBuf,   // ~/.local/share/trcaa on Linux
    pub integration_webviews: Arc<Mutex<HashMap<String, String>>>,
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
| `commands/integrations.rs` | Confluence / ServiceNow / ADO — OAuth2, WebView auth, tool calling |
| `ai/provider.rs` | `Provider` trait + `create_provider()` factory |
| `pii/detector.rs` | Multi-pattern PII scanner with overlap resolution |
| `db/migrations.rs` | Versioned schema (14 migrations tracked in `_migrations` table) |
| `db/models.rs` | All DB types — see `IssueDetail` note below |
| `docs/rca.rs` + `docs/postmortem.rs` | Markdown template builders |
| `audit/log.rs` | `write_audit_event()` — called before every external send |

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
│   └── integrations.rs
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
| `settingsStore.ts` | `localStorage` as `"tftsr-settings"` | AI providers, theme, Ollama URL, active provider |
| `historyStore.ts` | Not persisted (cache) | Past issues list, search query |

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

## Application Startup Sequence

```
1. Initialize tracing (RUST_LOG controls level)
2. Determine data directory (state::get_app_data_dir() or TFTSR_DATA_DIR)
3. Auto-generate or load .dbkey / .enckey (mode 0600) — see ADR-005
4. Open / create SQLCipher encrypted database
   - If plain SQLite detected (debug→release upgrade): auto-migrate + backup
5. Run DB migrations (14 schema versions)
6. Create AppState (db + settings + app_data_dir + integration_webviews)
7. Register Tauri plugins (stronghold, dialog, fs, shell, http)
8. Register all IPC command handlers via generate_handler![]
9. Start WebView with React app
```

## Architecture Documentation

Full architecture documentation with C4 diagrams, data flow diagrams, and Architecture Decision Records (ADRs) is available in [`docs/architecture/`](../architecture/README.md):

| Document | Contents |
|----------|----------|
| [Architecture Overview](../architecture/README.md) | C4 diagrams, data flows, security model |
| [ADR-001](../architecture/adrs/ADR-001-tauri-desktop-framework.md) | Why Tauri over Electron |
| [ADR-002](../architecture/adrs/ADR-002-sqlcipher-encrypted-database.md) | SQLCipher encryption choices |
| [ADR-003](../architecture/adrs/ADR-003-provider-trait-pattern.md) | AI provider trait design |
| [ADR-004](../architecture/adrs/ADR-004-pii-regex-aho-corasick.md) | PII detection implementation |
| [ADR-005](../architecture/adrs/ADR-005-auto-generate-encryption-keys.md) | Key auto-generation design |
| [ADR-006](../architecture/adrs/ADR-006-zustand-state-management.md) | Frontend state management |

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
