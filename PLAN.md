# TFTSR — IT Triage & Root-Cause Analysis Desktop Application

## Implementation Plan

### Overview

TFTSR is a **desktop-first, offline-capable** application that helps IT teams
perform structured incident triage using the *5-Whys* methodology, backed by
pluggable AI providers (Ollama local, OpenAI, Anthropic, Mistral, Gemini).
It automates PII redaction, guides engineers through root-cause analysis, and
produces post-mortem documents (Markdown / PDF / DOCX).

---

## Architecture Decisions

| Area | Choice | Rationale |
|------|--------|-----------|
| Desktop framework | **Tauri 2.x** | Small binary, native webview, Rust backend for security |
| Frontend framework | **React 18** | Large ecosystem, component model fits wizard-style UX |
| State management | **Zustand** | Minimal boilerplate, TypeScript-friendly, no context nesting |
| Local database | **SQLCipher** (via `rusqlite` + `bundled-sqlcipher`) | Encrypted SQLite — secrets and PII at rest |
| Secret storage | **Tauri Stronghold** | OS-keychain-grade encrypted vault for API keys |
| AI providers | Ollama (local), OpenAI, Anthropic, Mistral, Gemini | User choice; local-first with cloud fallback |
| Unit tests (frontend) | **Vitest** | Fast, Vite-native, first-class TS support |
| E2E tests | **WebdriverIO + tauri-driver** | Official Tauri E2E path, cross-platform |
| CI/CD | **Woodpecker CI** (Gogs at `172.0.0.29:3000`) | Self-hosted, Docker-native, YAML pipelines |
| Bundling | Vite 6 | Dev server + production build, used by Tauri CLI |

---

## Directory Structure

```
tftsr/
├── .woodpecker/
│   ├── test.yml              # lint + unit tests on push / PR
│   └── release.yml           # multi-platform build on tag
├── cli/
│   ├── package.json
│   └── src/
│       └── main.ts           # minimal CLI entry point
├── src/                      # React frontend
│   ├── assets/
│   ├── components/
│   │   ├── common/           # Button, Card, Modal, DropZone …
│   │   ├── dashboard/        # IssueList, StatsCards
│   │   ├── triage/           # WhyStep, ChatBubble, ProgressBar
│   │   ├── rca/              # DocEditor, ExportBar
│   │   ├── settings/         # ProviderForm, ThemeToggle
│   │   └── pii/              # PiiHighlighter, RedactionPreview
│   ├── hooks/                # useInvoke, useListener, useTheme …
│   ├── lib/
│   │   ├── tauriCommands.ts  # typed invoke wrappers & TS types
│   │   └── utils.ts          # date formatting, debounce, etc.
│   ├── pages/
│   │   ├── DashboardPage.tsx
│   │   ├── NewIssuePage.tsx
│   │   ├── TriagePage.tsx
│   │   ├── RcaPage.tsx
│   │   ├── LogViewerPage.tsx
│   │   └── SettingsPage.tsx
│   ├── stores/
│   │   ├── sessionStore.ts   # current triage session state
│   │   └── settingsStore.ts  # theme, providers, preferences
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   ├── src/
│   │   ├── main.rs           # Tauri entry point
│   │   ├── db.rs             # SQLCipher connection & migrations
│   │   ├── commands/         # IPC command modules
│   │   │   ├── mod.rs
│   │   │   ├── issues.rs
│   │   │   ├── triage.rs
│   │   │   ├── logs.rs
│   │   │   ├── pii.rs
│   │   │   ├── rca.rs
│   │   │   ├── ai.rs
│   │   │   └── settings.rs
│   │   ├── ai/               # AI provider abstractions
│   │   │   ├── mod.rs
│   │   │   ├── ollama.rs
│   │   │   ├── openai_compat.rs
│   │   │   └── prompt_templates.rs
│   │   ├── pii/              # PII detection engine
│   │   │   ├── mod.rs
│   │   │   └── patterns.rs
│   │   └── export/           # Document export
│   │       ├── mod.rs
│   │       ├── markdown.rs
│   │       ├── pdf.rs
│   │       └── docx.rs
│   └── migrations/
│       └── 001_init.sql
├── tests/
│   ├── unit/
│   │   ├── setup.ts
│   │   ├── pii.test.ts
│   │   ├── sessionStore.test.ts
│   │   └── settingsStore.test.ts
│   └── e2e/
│       ├── wdio.conf.ts
│       ├── helpers/
│       │   └── app.ts
│       └── specs/
│           ├── onboarding.spec.ts
│           ├── log-upload.spec.ts
│           ├── triage-flow.spec.ts
│           └── rca-export.spec.ts
├── package.json
├── tsconfig.json
├── vite.config.ts
└── PLAN.md                   # ← this file
```

---

## Database Schema (SQLCipher)

All tables live in a single encrypted `tftsr.db` file under the Tauri
app-data directory.

### 1. `issues`
```sql
CREATE TABLE issues (
  id          TEXT PRIMARY KEY,
  title       TEXT NOT NULL,
  domain      TEXT NOT NULL CHECK(domain IN
    ('linux','windows','network','k8s','db','virt','hw','obs')),
  status      TEXT NOT NULL DEFAULT 'open'
    CHECK(status IN ('open','triaging','resolved','closed')),
  severity    TEXT CHECK(severity IN ('p1','p2','p3','p4')),
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);
```

### 2. `triage_messages`
```sql
CREATE TABLE triage_messages (
  id          TEXT PRIMARY KEY,
  issue_id    TEXT NOT NULL REFERENCES issues(id),
  role        TEXT NOT NULL CHECK(role IN ('user','assistant','system')),
  content     TEXT NOT NULL,
  why_level   INTEGER NOT NULL DEFAULT 0,
  created_at  INTEGER NOT NULL
);
CREATE INDEX idx_triage_msg_issue ON triage_messages(issue_id);
```

### 3. `log_files`
```sql
CREATE TABLE log_files (
  id          TEXT PRIMARY KEY,
  issue_id    TEXT NOT NULL REFERENCES issues(id),
  filename    TEXT NOT NULL,
  content     TEXT NOT NULL,
  mime_type   TEXT,
  size_bytes  INTEGER,
  created_at  INTEGER NOT NULL
);
```

### 4. `pii_spans`
```sql
CREATE TABLE pii_spans (
  id           TEXT PRIMARY KEY,
  log_file_id  TEXT NOT NULL REFERENCES log_files(id),
  pii_type     TEXT NOT NULL,
  start_pos    INTEGER NOT NULL,
  end_pos      INTEGER NOT NULL,
  original     TEXT NOT NULL,
  replacement  TEXT NOT NULL
);
```

### 5. `rca_documents`
```sql
CREATE TABLE rca_documents (
  id          TEXT PRIMARY KEY,
  issue_id    TEXT NOT NULL REFERENCES issues(id) UNIQUE,
  content     TEXT NOT NULL DEFAULT '',
  format      TEXT NOT NULL DEFAULT 'markdown',
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);
```

### 6. `ai_providers`
```sql
CREATE TABLE ai_providers (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL UNIQUE,
  api_url     TEXT NOT NULL,
  model       TEXT NOT NULL,
  created_at  INTEGER NOT NULL
);
```

### 7. `settings`
```sql
CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
```

### 8. `export_history`
```sql
CREATE TABLE export_history (
  id          TEXT PRIMARY KEY,
  issue_id    TEXT NOT NULL REFERENCES issues(id),
  format      TEXT NOT NULL CHECK(format IN ('md','pdf','docx')),
  file_path   TEXT NOT NULL,
  created_at  INTEGER NOT NULL
);
```

---

## IPC Command Interface

All frontend ↔ backend communication goes through Tauri's `invoke()`.

### Issue commands
| Command | Payload | Returns |
|---------|---------|---------|
| `create_issue` | `{ title, domain, severity }` | `Issue` |
| `list_issues` | `{ status?, domain? }` | `Issue[]` |
| `get_issue` | `{ id }` | `Issue` |
| `update_issue` | `{ id, title?, status?, severity? }` | `Issue` |
| `delete_issue` | `{ id }` | `void` |

### Triage commands
| Command | Payload | Returns |
|---------|---------|---------|
| `send_triage_message` | `{ issueId, content, whyLevel }` | `TriageMessage` (assistant reply) |
| `get_triage_history` | `{ issueId }` | `TriageMessage[]` |
| `set_why_level` | `{ issueId, level }` | `void` |

### Log commands
| Command | Payload | Returns |
|---------|---------|---------|
| `upload_log` | `{ issueId, filename, content }` | `LogFile` |
| `list_logs` | `{ issueId }` | `LogFile[]` |
| `delete_log` | `{ id }` | `void` |

### PII commands
| Command | Payload | Returns |
|---------|---------|---------|
| `detect_pii` | `{ logFileId }` | `PiiDetectionResult` |
| `apply_redactions` | `{ logFileId, spanIds }` | `string` (redacted text) |

### RCA / Export commands
| Command | Payload | Returns |
|---------|---------|---------|
| `generate_rca` | `{ issueId }` | `RcaDocument` |
| `update_rca` | `{ id, content }` | `RcaDocument` |
| `export_document` | `{ issueId, format }` | `string` (file path) |

### AI / Settings commands
| Command | Payload | Returns |
|---------|---------|---------|
| `test_provider` | `{ name, apiUrl, apiKey?, model }` | `{ ok, message }` |
| `save_provider` | `{ provider }` | `void` |
| `get_settings` | `{}` | `Settings` |
| `update_settings` | `{ key, value }` | `void` |

---

## CI/CD Approach

### Infrastructure
- **Git server**: Gogs at `http://172.0.0.29:3000`
- **CI runner**: Woodpecker CI with Docker executor
- **Artifacts**: Uploaded to Gogs releases via API

### Pipelines

| Pipeline | Trigger | Steps |
|----------|---------|-------|
| `.woodpecker/test.yml` | push, PR | `rustfmt` check → Clippy → Rust tests → TS typecheck → Vitest → coverage (main only) |
| `.woodpecker/release.yml` | `v*` tag | Build linux-amd64 → Build linux-arm64 → Upload to Gogs release |

---

## Security Implementation

1. **Database encryption** — SQLCipher with a key derived from Tauri Stronghold.
2. **API key storage** — Stronghold vault, never stored in plaintext.
3. **PII redaction** — Regex + heuristic engine runs before any text leaves the device.
4. **CSP** — Strict Content-Security-Policy in `tauri.conf.json`; only allowlisted AI API origins.
5. **Least-privilege capabilities** — `capabilities/default.json` grants only required Tauri permissions.
6. **No remote code** — All assets bundled; no CDN scripts.

---

## Testing Strategy

| Layer | Tool | Location | What it covers |
|-------|------|----------|----------------|
| Rust unit | `cargo test` | `src-tauri/src/**` | DB operations, PII regex, AI prompt building |
| Frontend unit | Vitest | `tests/unit/` | Stores, command wrappers, component logic |
| E2E | WebdriverIO + tauri-driver | `tests/e2e/` | Full user flows: onboarding, triage, export |
| Lint | `rustfmt` + Clippy + `tsc --noEmit` | CI | Code style, type safety |

---

## Implementation Phases

### Phase 1 — Project Scaffold & CI [IN PROGRESS]
- [x] Initialise repo with Tauri 2.x + React 18 + Vite
- [x] Configure `tauri.conf.json` and capabilities
- [x] Set up Woodpecker CI pipelines (`test.yml`, `release.yml`)
- [x] Write Vitest setup and mock harness
- [x] Write initial unit tests (PII, sessionStore, settingsStore)
- [x] Write E2E scaffolding (wdio config, helpers, skeleton specs)
- [x] Create CLI stub (`cli/`)
- [ ] Verify CI green on first push

### Phase 2 — Database & Migrations
- [ ] Integrate `rusqlite` + `bundled-sqlcipher`
- [ ] Write `001_init.sql` migration with all 8 tables
- [ ] Implement migration runner in `db.rs`
- [ ] Unit-test DB operations

### Phase 3 — Stronghold Integration
- [ ] Add `tauri-plugin-stronghold`
- [ ] Store/retrieve DB encryption key
- [ ] Store/retrieve AI API keys
- [ ] Test key lifecycle

### Phase 4 — Issue CRUD
- [ ] Implement `commands/issues.rs`
- [ ] Wire IPC commands
- [ ] Build `DashboardPage` and `NewIssuePage` UI
- [ ] Unit-test issue store + commands

### Phase 5 — Log Ingestion & PII Detection
- [ ] Implement `commands/logs.rs` and `pii/` engine
- [ ] Build `DropZone` + `PiiHighlighter` components
- [ ] Write comprehensive PII regex tests
- [ ] E2E: log upload flow

### Phase 6 — AI Provider Abstraction
- [ ] Implement `ai/ollama.rs` and `ai/openai_compat.rs`
- [ ] Build `SettingsPage` provider configuration UI
- [ ] `test_provider` command with connectivity check
- [ ] Unit-test prompt templates

### Phase 7 — 5-Whys Triage Engine
- [ ] Implement `commands/triage.rs` with streaming support
- [ ] Build `TriagePage` with `WhyStep` + `ChatBubble`
- [ ] Wire progress bar to why-level state
- [ ] E2E: full triage flow

### Phase 8 — RCA Document Generation
- [ ] Implement `commands/rca.rs` + `generate_rca`
- [ ] Build `RcaPage` with `DocEditor`
- [ ] Test RCA generation with mock AI responses

### Phase 9 — Document Export
- [ ] Implement `export/markdown.rs`, `pdf.rs`, `docx.rs`
- [ ] Build export bar with format selection
- [ ] Test each export format
- [ ] E2E: export flow

### Phase 10 — Polish & Accessibility
- [ ] Dark/light theme toggle
- [ ] Keyboard navigation
- [ ] Loading states and error boundaries
- [ ] Responsive layout adjustments

### Phase 11 — Release Pipeline Validation
- [ ] Tag `v0.1.0-alpha`
- [ ] Verify Woodpecker builds Linux amd64 + arm64
- [ ] Verify artifacts upload to Gogs release
- [ ] Smoke-test installed packages

### Phase 12 — Documentation & Handoff
- [ ] Write user-facing README
- [ ] Document AI provider setup guide
- [ ] Record architecture decision log
- [ ] Final CI badge + release notes
