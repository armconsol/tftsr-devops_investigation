# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## Commands

### Development

```bash
# Start full dev environment (Vite + Tauri hot reload)
cargo tauri dev

# Frontend only (Vite at localhost:1420)
npm run dev

# Frontend production build
npm run build
```

> Rust toolchain must be in PATH: `source ~/.cargo/env`

### Testing

```bash
# Rust unit tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run a single Rust test module
cargo test --manifest-path src-tauri/Cargo.toml pii::detector

# Run a single Rust test by name
cargo test --manifest-path src-tauri/Cargo.toml test_detect_ipv4

# Frontend tests (single run)
npm run test:run

# Frontend tests (watch mode)
npm run test

# Frontend coverage report
npm run test:coverage

# TypeScript type check
npx tsc --noEmit
```

### Linting

```bash
# Rust format check
cargo fmt --manifest-path src-tauri/Cargo.toml --check

# Rust lints
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Rust quick type check (no linking)
cargo check --manifest-path src-tauri/Cargo.toml
```

### System Prerequisites (Linux/Fedora)

```bash
sudo dnf install -y glib2-devel gtk3-devel webkit2gtk4.1-devel \
  libsoup3-devel openssl-devel librsvg2-devel
```

### Production Build

```bash
cargo tauri build  # Outputs to src-tauri/target/release/bundle/
```

### CI/CD

- **Test pipeline**: `.woodpecker/test.yml` — runs on every push/PR
- **Release pipeline**: `.woodpecker/release.yml` — runs on `v*` tags, produces Linux amd64+arm64 bundles, uploads to Gogs release at `http://172.0.0.29:3000/api/v1`

---

## Architecture

### Backend (Rust / Tauri)

**Entry point**: `src-tauri/src/lib.rs` → `run()` initialises tracing, opens the DB, registers Tauri plugins, and calls `generate_handler![]` with all IPC commands.

**Shared state** (`src-tauri/src/state.rs`):
```rust
pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub settings: Arc<Mutex<AppSettings>>,
    pub app_data_dir: PathBuf,  // ~/.local/share/tftsr on Linux
}
```

All command handlers receive `State<'_, AppState>` as a Tauri-injected parameter. Lock the Mutex inside a `{ }` block and release it **before** any `.await` — holding a `MutexGuard` across an await point causes a compile error because `MutexGuard` is not `Send`.

**Module layout**:
| Path | Responsibility |
|------|----------------|
| `commands/db.rs` | Issue CRUD, 5-whys entries, timeline events |
| `commands/ai.rs` | `analyze_logs`, `chat_message`, `list_providers` |
| `commands/analysis.rs` | Log file upload, PII detection, redaction application |
| `commands/docs.rs` | RCA and post-mortem generation, document export |
| `commands/system.rs` | Ollama management, hardware probe, app settings, audit log |
| `commands/integrations.rs` | Confluence / ServiceNow / ADO — **all v0.2 stubs** |
| `ai/provider.rs` | `Provider` trait + `create_provider()` factory |
| `pii/detector.rs` | Multi-pattern PII scanner with overlap resolution |
| `db/migrations.rs` | Versioned schema (10 migrations tracked in `_migrations` table) |
| `db/models.rs` | All DB types — see IssueDetail note below |
| `docs/rca.rs` + `docs/postmortem.rs` | Markdown template builders |
| `audit/log.rs` | `write_audit_event()` — called before every external send |

**AI provider factory**: `ai/provider.rs::create_provider(config)` dispatches on `config.name` to the matching struct. Adding a provider means implementing the `Provider` trait and adding a match arm.

**Database encryption**: `cfg!(debug_assertions)` → plain SQLite; release → SQLCipher AES-256. Key from `TFTSR_DB_KEY` env var (defaults to a dev placeholder). DB path from `TFTSR_DATA_DIR` or platform data dir.

### Frontend (React / TypeScript)

**IPC layer**: All Tauri `invoke()` calls are in `src/lib/tauriCommands.ts`. Every command has a typed wrapper function (e.g., `createIssueCmd`, `chatMessageCmd`). This is the single source of truth for the frontend's API surface.

**Stores** (Zustand):
- `sessionStore.ts` — ephemeral triage session: current issue, chat messages, PII spans, why-level (0–5), loading state. **Not persisted.**
- `settingsStore.ts` — AI providers, theme, Ollama URL. **Persisted** to `localStorage` as `"tftsr-settings"`.
- `historyStore.ts` — read-only cache of past issues for the History page.

**Page flow**:
```
NewIssue → createIssueCmd → startSession(detail.issue) → navigate /issue/:id/triage
LogUpload → uploadLogFileCmd → detectPiiCmd → applyRedactionsCmd
Triage   → chatMessageCmd loop, parse AI response for "why 2..5", detect root cause
Resolution → getIssueCmd, mark 5-whys steps done
RCA      → generateRcaCmd → DocEditor → exportDocumentCmd
```

**Domain system prompts**: `src/lib/domainPrompts.ts` contains expert-level system prompts for Linux, Windows, Network, Kubernetes, Databases, Virtualization, Hardware, and Observability. Each prompt is injected as the first message in every triage conversation.

### Key Type: `IssueDetail`

`get_issue()` returns a **nested** struct, not a flat `Issue`. Use `detail.issue.title`, not `detail.title`:

```rust
pub struct IssueDetail {
    pub issue: Issue,                          // Base issue fields
    pub log_files: Vec<LogFile>,
    pub resolution_steps: Vec<ResolutionStep>, // 5-whys entries
    pub conversations: Vec<AiConversation>,
}
```

On the TypeScript side, `tauriCommands.ts` mirrors this shape exactly.

### PII Detection

`PiiDetector::detect(&str)` returns `Vec<PiiSpan>` with non-overlapping spans (longest match wins on overlap). Spans carry `start`/`end` byte offsets and a `replacement` string (`[IPv4]`, `[EMAIL]`, etc.). The redactor applies spans by iterating in reverse order to preserve offsets.

Before any text is sent to an AI provider, `apply_redactions` must be called and the resulting SHA-256 hash recorded via `audit::log::write_audit_event`.

### Woodpecker CI + Gogs Compatibility

**Status**: Woodpecker CI v0.15.4 is deployed at `http://172.0.0.29:8084` (direct) and `http://172.0.0.29:8085` (nginx proxy). Webhook delivery from Gogs works, but CI builds are not yet triggering due to hook authentication issues. See `PLAN.md § Phase 11` for full details.

Known issues with Woodpecker 0.15.4 + Gogs 0.14:
- `token.ParseRequest()` does not read `?token=` URL params (only `Authorization` header and `user_sess` cookie)
- The SPA login form uses `login=` field; Gogs backend reads `username=` — a custom login page is served by nginx at `/login` and `/login/form`
- Gogs 0.14 has no OAuth2 provider support, blocking upgrade to Woodpecker 2.x

Gogs token quirk: the `sha1` value returned by `POST /api/v1/users/{user}/tokens` is the **actual bearer token**. The `sha1` and `sha256` columns in the Gogs DB are hashes of that token, not the token itself.

---

## Wiki Maintenance

The project wiki lives at `https://gogs.tftsr.com/sarman/tftsr-devops_investigation/wiki`.

**Source of truth**: `docs/wiki/*.md` in this repo. The `wiki-sync` CI step (in `.woodpecker/test.yml`) automatically pushes any changes to the Gogs wiki on every push to master.

**When making code changes, update the corresponding wiki file in `docs/wiki/` before committing:**

| Changed area | Wiki file to update |
|---|---|
| New/changed Tauri commands (`commands/*.rs`, `tauriCommands.ts`) | `docs/wiki/IPC-Commands.md` |
| DB schema or migrations (`db/migrations.rs`, `db/models.rs`) | `docs/wiki/Database.md` |
| New/changed AI provider (`ai/*.rs`) | `docs/wiki/AI-Providers.md` |
| PII patterns or detection logic (`pii/`) | `docs/wiki/PII-Detection.md` |
| CI/CD pipeline changes (`.woodpecker/*.yml`) | `docs/wiki/CICD-Pipeline.md` |
| Rust architecture or module layout (`lib.rs`, `state.rs`) | `docs/wiki/Architecture.md` |
| Security-relevant changes (capabilities, audit, Stronghold) | `docs/wiki/Security-Model.md` |
| Dev setup, prerequisites, build commands | `docs/wiki/Development-Setup.md` |
| Integration stubs or v0.2 progress (`integrations/`) | `docs/wiki/Integrations.md` |
| Recurring bugs and fixes | `docs/wiki/Troubleshooting.md` |

To manually push wiki changes without waiting for CI:
```bash
cd /tmp/tftsr-wiki   # local clone of the wiki git repo
# edit *.md files, then:
git add -A && git commit -m "docs: ..." && git push
```
