# AGENTS.md — Quick Start for OpenCode

## Commands

| Task | Command |
|------|---------|
| Run full dev (Tauri + Vite) | `cargo tauri dev` |
| Frontend only (port 1420) | `npm run dev` |
| Frontend production build | `npm run build` |
| Rust fmt check | `cargo fmt --manifest-path src-tauri/Cargo.toml --check` |
| Rust fmt fix | `cargo fmt --manifest-path src-tauri/Cargo.toml` |
| Rust clippy | `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` |
| Rust tests | `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1` |
| Rust single test module | `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1 pii::detector` |
| Rust single test | `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1 test_detect_ipv4` |
| Frontend test (single run) | `npm run test:run` |
| Frontend test (watch) | `npm run test` |
| Frontend coverage | `npm run test:coverage` |
| TypeScript type check | `npx tsc --noEmit` |
| Frontend lint | `npx eslint . --quiet` |

**Lint Policy**: **ALWAYS run `cargo fmt` and `cargo clippy` after any Rust code change**. Fix all issues before proceeding.

**Note**: The build runs `npm run build` before Rust build (via `beforeBuildCommand` in `tauri.conf.json`). This ensures TS is type-checked before packaging.

**Requirement**: Rust toolchain must be in PATH: `source ~/.cargo/env`

---

## Project Structure

| Path | Responsibility |
|------|----------------|
| `src-tauri/src/lib.rs` | Entry point: app builder, plugin registration, IPC handler registration |
| `src-tauri/src/state.rs` | `AppState` (DB, settings, integration_webviews) |
| `src-tauri/src/commands/` | Tauri IPC handlers (db, ai, analysis, docs, integrations, system) |
| `src-tauri/src/ai/provider.rs` | `Provider` trait + `create_provider()` factory |
| `src-tauri/src/pii/` | Detection engine (12 patterns) + redaction |
| `src-tauri/src/db/models.rs` | DB types: `Issue`, `IssueDetail` (nested), `LogFile`, `ResolutionStep`, `AiConversation` |
| `src-tauri/src/audit/log.rs` | `write_audit_event()` before every external send |
| `src/lib/tauriCommands.ts` | **Source of truth** for all Tauri IPC calls |
| `src/lib/domainPrompts.ts` | 15 domain system prompts (Linux, Windows, Network, K8s, DBs, etc.) |
| `src/stores/` | Zustand: `sessionStore` (ephemeral), `settingsStore` (persisted), `historyStore` |

---

## Key Patterns

### Rust Mutex Usage
Lock `Mutex` inside a block and release **before** `.await`. Holding `MutexGuard` across await points fails to compile (not `Send`):

```rust
let state: State<'_, AppState> = app.state();
let db = state.db.clone();
// Lock and release before await
{ let conn = state.db.lock().unwrap(); /* use conn */ }
// Now safe to .await
db.query(...).await?;
```

### IssueDetail Nesting
`get_issue()` returns a **nested** struct — use `detail.issue.title`, not `detail.title`:

```rust
pub struct IssueDetail {
    pub issue: Issue,
    pub log_files: Vec<LogFile>,
    pub resolution_steps: Vec<ResolutionStep>,
    pub conversations: Vec<AiConversation>,
}
```

TypeScript mirrors this shape exactly in `tauriCommands.ts`.

### PII Before AI Send
`apply_redactions` **must** be called before sending logs to AI. Record the SHA-256 hash via `audit::log::write_audit_event()`. PII spans are non-overlapping (longest span wins on overlap); redactor iterates in reverse order to preserve offsets.

### State Persistence
- `sessionStore`: ephemeral triage session (issue, messages, PII spans, why-level 0–5, loading) — **not persisted**
- `settingsStore`: persisted to `localStorage` as `"tftsr-settings"`

---

## CI/CD (Gitea Actions)

| Workflow | Trigger | Jobs |
|----------|---------|------|
| `.gitea/workflows/test.yml` | Every push/PR | `rustfmt` → `clippy` → `cargo test` (64 tests) → `tsc --noEmit` → `vitest run` (13 tests) |
| `.gitea/workflows/auto-tag.yml` | Push to master | Auto-tag, build linux/amd64 + windows/amd64 + linux/arm64 + macOS, upload assets to Gitea release |

**Artifacts**: `src-tauri/target/{target}/release/bundle/`

**Environments**:
- Test CI images at `172.0.0.29:3000` (pull `trcaa-*:rust1.88-node22`)
- Gitea instance: `http://172.0.0.29:3000`
- Wiki: sync from `docs/wiki/*.md` → `https://gogs.tftsr.com/sarman/tftsr-devops_investigation/wiki`

---

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `TFTSR_DATA_DIR` | Platform data dir | Override database location |
| `TFTSR_DB_KEY` | Auto-generated | SQLCipher encryption key override |
| `TFTSR_ENCRYPTION_KEY` | Auto-generated | Credential encryption key override |
| `RUST_LOG` | `info` | Tracing level (`debug`, `info`, `warn`, `error`) |

**Database path**:
- Linux: `~/.local/share/trcaa/trcaa.db`
- macOS: `~/Library/Application Support/trcaa/trcaa.db`
- Windows: `%APPDATA%\trcaa\trcaa.db`

---

## Architecture Highlights

### Rust Backend
- **Entry point**: `src-tauri/src/lib.rs::run()` → init tracing → init DB → register plugins → `generate_handler![]`
- **Database**: `rusqlite` + `bundled-sqlcipher-vendored-openssl` (AES-256). `cfg!(debug_assertions)` → plain SQLite; release → SQLCipher
- **AI providers**: `Provider` trait with factory dispatch on `config.name`. Adding a provider: implement `Provider` trait + add match arm
- **Integration clients**: Confluence, ServiceNow, Azure DevOps stubs (v0.2). OAuth2 via WebView + callback server (warp, port 8765)

### Frontend (React + Vite)
- **Dev server**: port **1420** (hardcoded)
- **IPC**: All `invoke()` calls in `src/lib/tauriCommands.ts` — typed wrappers for every backend command
- **Domain prompts**: 15 expert prompts injected as first message in every triage conversation (Linux, Windows, Network, K8s, DBs, Virtualization, Hardware, Observability, Telephony, Security, Public Safety, Application, Automation, HPE, Dell, Identity)

### Security
- **Database encryption**: AES-256 (SQLCipher in release builds)
- **Credential encryption**: AES-256-GCM, keys stored in `TFTSR_ENCRYPTION_KEY` or auto-generated `.enckey` (mode 0600)
- **Audit trail**: Hash-chained entries (`prev_hash` + `entry_hash`) for tamper evidence
- **PII protection**: 12-pattern detector → user approval gate → hash-chained audit entry

---

## Testing

| Layer | Command | Notes |
|-------|---------|-------|
| Rust | `cargo test --manifest-path src-tauri/Cargo.toml` | 64 tests, runs in `rust:1.88-slim` container |
| TypeScript | `npm run test:run` | Vitest, 13 tests |
| Type check | `npx tsc --noEmit` | `skipLibCheck: true` |
| E2E | `TAURI_BINARY_PATH=./src-tauri/target/release/tftsr npm run test:e2e` | WebdriverIO, requires compiled binary |

**Frontend coverage**: `npm run test:coverage` → `tests/unit/` coverage report

---

## Critical Gotchas

1. **Mutex across await**: Never `lock().unwrap()` and `.await` without releasing the guard
2. **IssueDetail nesting**: `detail.issue.title`, never `detail.title`
3. **PII before AI**: Always redact and record hash before external send
4. **Port 1420**: Vite dev server is hard-coded to 1420, not 3000
5. **Build order**: Rust fmt → clippy → test → TS check → JS test
6. **CI images**: Use `172.0.0.29:3000` registry for pre-baked builder images
