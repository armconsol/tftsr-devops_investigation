# Troubleshooting and RCA Assistant

A structured, AI-backed desktop tool for IT incident triage, 5-Whys root cause analysis, RCA document generation, and blameless post-mortems. Runs fully offline via Ollama local models, or connects to cloud AI providers.

Built with **Tauri 2** (Rust + WebView), **React 18**, **TypeScript**, and **SQLCipher AES-256** encrypted storage.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
![CI](http://172.0.0.29:3000/sarman/tftsr-devops_investigation/actions/workflows/test.yml/badge.svg)

---

## Features

- **5-Whys AI Triage** — Guided root cause analysis via AI chat, with auto-detection of why levels 1–5
- **PII Sanitization** — Automatic detection and redaction of IPv4/IPv6, emails, tokens, passwords, SSNs, and more before any data leaves the machine
- **Multi-Provider AI** — OpenAI, Anthropic Claude, Google Gemini, Mistral, and local [Ollama](https://ollama.com) (offline)
- **Encrypted Database** — SQLCipher AES-256 encrypted SQLite; all issue history stays local
- **RCA + Post-Mortem Generation** — Auto-populated Markdown templates, exportable to `.md` and `.pdf`
- **Ollama Management** — Hardware detection, model recommendations, pull/delete models in-app
- **Audit Trail** — Every external data send logged with SHA-256 hash
- **Domain System Prompts** — Pre-built expert context for 8 IT domains (Linux, Windows, Network, Kubernetes, Databases, Virtualization, Hardware, Observability)
- **Integrations** *(v0.2, coming soon)* — Confluence, ServiceNow, Azure DevOps

---

## Supported Domains

| Domain | Coverage |
|---|---|
| Linux | RHEL/OEL, systemd, journald, SELinux, kernel panics |
| Windows | Event IDs, WinRM, BSOD codes, Server 2019/2022 |
| Network | Fortigate, Cisco IOS, Aruba AOS-CX, Nokia SR-OS, VoIP SIP/RTP |
| Kubernetes | k3s, OpenShift, CrashLoopBackOff, OOMKill, etcd, Rancher |
| Databases | PostgreSQL WAL, Redis AOF/RDB, RabbitMQ, MSSQL |
| Virtualization | Proxmox VE/PBS, VDI sessions |
| Hardware | HPE Synergy 12000, DL-20/320/360/380, iLO event logs |
| Observability | Kibana/ECK, Elasticsearch shard failures |

---

## Architecture

| Component | Technology |
|---|---|
| App framework | Tauri 2.x (Rust + WebView) |
| Frontend | React 18 + TypeScript + Vite |
| UI | Tailwind CSS (custom shadcn-style components) |
| Database | rusqlite + `bundled-sqlcipher` (AES-256) |
| Secret storage | `tauri-plugin-stronghold` |
| State management | Zustand (persisted settings store with API key redaction) |
| AI providers | reqwest (async HTTP) |
| PII detection | regex + aho-corasick multi-pattern engine |

---

## Prerequisites

### System Libraries (Linux — Fedora/RHEL)

```bash
sudo dnf install -y \
  glib2-devel gtk3-devel webkit2gtk4.1-devel \
  libsoup3-devel openssl-devel librsvg2-devel
```

### System Libraries (Linux — Debian/Ubuntu)

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf pkg-config
```

### Toolchain

```bash
# Rust (minimum 1.88 — required by cookie_store, time, darling)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Node.js 22+ (via your package manager)
# Verify:
rustc --version   # 1.88+
node --version    # 22+
```

---

## Getting Started

```bash
# Clone
git clone https://gogs.tftsr.com/sarman/tftsr-devops_investigation.git
cd tftsr-devops_investigation
npm install --legacy-peer-deps

# Development mode (hot reload)
source ~/.cargo/env
cargo tauri dev

# Production build
cargo tauri build
# Output: src-tauri/target/release/bundle/
```

---

## Releases

Pre-built installers are attached to each [tagged release](https://gogs.tftsr.com/sarman/tftsr-devops_investigation/releases):

| Platform | Format | Notes |
|---|---|---|
| Linux amd64 | `.deb`, `.rpm`, `.AppImage` | Standard package or universal AppImage |
| Windows amd64 | `.exe` (NSIS), `.msi` | From cross-compile via mingw-w64 |
| Linux arm64 | `.deb`, `.rpm`, `.AppImage` | Built natively on arm64 runner |
| macOS | — | Requires macOS runner — build locally |

---

## AI Provider Setup

Launch the app and go to **Settings → AI Providers** to add a provider:

| Provider | API URL | Notes |
|---|---|---|
| OpenAI | `https://api.openai.com/v1` | Requires API key |
| Anthropic | `https://api.anthropic.com` | Requires API key |
| Google Gemini | `https://generativelanguage.googleapis.com` | Requires API key |
| Mistral | `https://api.mistral.ai/v1` | Requires API key |
| Ollama (local) | `http://localhost:11434` | No key needed — fully offline |
| Azure OpenAI | `https://<resource>.openai.azure.com/openai/deployments/<deployment>` | Requires API key |
| **AWS Bedrock (via LiteLLM)** | `http://localhost:8000/v1` | See [LiteLLM + AWS Bedrock](#litellm--aws-bedrock-setup) below |
| **Custom REST Gateway** | Your gateway URL | See [Custom REST format](docs/wiki/AI-Providers.md) |

For offline use, install [Ollama](https://ollama.com) and pull a model:
```bash
ollama pull llama3.2:3b   # Good for most hardware (≥8 GB RAM)
ollama pull llama3.1:8b   # Better quality (≥16 GB RAM)
```

Or use **Settings → Ollama** to pull models directly from within the app.

### LiteLLM + AWS Bedrock Setup

To use Claude via AWS Bedrock (ideal for enterprise environments with existing AWS contracts):

1. **Install LiteLLM:**
   ```bash
   pip install litellm[proxy]
   ```

2. **Create config file** at `~/.litellm/config.yaml`:
   ```yaml
   model_list:
     - model_name: bedrock-claude
       litellm_params:
         model: bedrock/us.anthropic.claude-sonnet-4-6
         aws_region_name: us-east-1
         # Optionally specify aws_profile_name if not using default

   general_settings:
     master_key: sk-your-secure-key  # Any value for API auth
   ```

3. **Start LiteLLM proxy:**
   ```bash
   nohup litellm --config ~/.litellm/config.yaml --port 8000 > ~/.litellm/litellm.log 2>&1 &
   ```

4. **Configure in Troubleshooting and RCA Assistant:**
   - Provider: **OpenAI** (OpenAI-compatible)
   - Base URL: `http://localhost:8000/v1`
   - API Key: `sk-your-secure-key` (from config)
   - Model: `bedrock-claude`

For detailed setup including multiple AWS accounts and Claude Code integration, see the [LiteLLM + Bedrock wiki page](https://gogs.tftsr.com/sarman/tftsr-devops_investigation/wiki/LiteLLM-Bedrock-Setup).

---

## Triage Workflow

```
1. New Issue      → Select domain, enter title and severity
2. Log Upload     → Drag-and-drop log files, review PII redactions
3. Triage         → 5-Whys AI conversation, auto-tracked why levels 1–5
4. Resolution     → Review and confirm each root cause and action
5. RCA            → Auto-generated RCA document, export as MD or PDF
6. Post-Mortem    → Blameless post-mortem document with action items
```

---

## Project Structure

```
tftsr/
├── src-tauri/src/
│   ├── ai/           # AI provider clients (OpenAI, Anthropic, Gemini, Mistral, Ollama)
│   ├── pii/          # PII detection + redaction engine
│   ├── db/           # SQLCipher connection, migrations, models
│   ├── ollama/       # Hardware detection, model recommendations, download manager
│   ├── docs/         # RCA + post-mortem generators, PDF/MD exporters
│   ├── integrations/ # Confluence, ServiceNow, Azure DevOps (v0.2 stubs)
│   ├── audit/        # Audit log writer
│   ├── commands/     # Tauri IPC command handlers
│   ├── lib.rs        # App builder, plugin registration, command handler registration
│   └── state.rs      # AppState (DB connection, settings)
├── src/
│   ├── pages/        # Dashboard, NewIssue, LogUpload, Triage, Resolution, RCA, Postmortem, History, Settings
│   ├── components/   # ChatWindow, TriageProgress, PiiDiffViewer, DocEditor, HardwareReport, ModelSelector, UI
│   ├── stores/       # sessionStore, settingsStore (persisted), historyStore
│   ├── lib/          # tauriCommands.ts (typed IPC wrappers), domainPrompts.ts
│   └── styles/       # Tailwind + CSS custom properties
├── tests/
│   ├── unit/         # Vitest unit tests (PII, session store, settings store)
│   └── e2e/          # WebdriverIO + tauri-driver E2E skeletons
├── docs/wiki/        # Source of truth for Gitea wiki
└── .gitea/
    └── workflows/
        ├── test.yml     # CI: rustfmt · clippy · cargo test · tsc · vitest (every push/PR)
        └── auto-tag.yml # Auto tag + release: linux/amd64 + windows/amd64 + linux/arm64 + macOS
```

---

## Testing

```bash
# Unit tests (Vitest) — 13/13 passing
npm run test:run

# Frontend coverage
npm run test:coverage

# TypeScript type check
npx tsc --noEmit

# Rust checks — 64/64 tests passing
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml

# E2E tests (requires compiled app binary)
TAURI_BINARY_PATH=./src-tauri/target/release/tftsr npm run test:e2e
```

---

## CI/CD — Gitea Actions

The project uses **Gitea Actions** (act_runner v0.3.1) connected to the Gitea instance at `gogs.tftsr.com`.

| Workflow | Trigger | Jobs |
|---|---|---|
| `.gitea/workflows/test.yml` | Every push / PR | rustfmt · clippy · cargo test (64) · tsc · vitest (13) |
| `.gitea/workflows/auto-tag.yml` | Push to `master` | Auto-tag, then build linux/amd64 + windows/amd64 + linux/arm64 + macOS and upload assets |

**Runners:**

| Runner | Platform | Host | Purpose |
|---|---|---|---|
| `amd64-docker-runner` | linux/amd64 | 172.0.0.29 (Docker) | Test pipeline + amd64/windows release builds |
| `arm64-native-runner` | linux/arm64 | Local arm64 machine | Native arm64 release builds |

**Branch protection:** master requires a PR approved by `sarman`, with all 5 CI checks passing before merge.

> See [CI/CD Pipeline wiki](https://gogs.tftsr.com/sarman/tftsr-devops_investigation/wiki/CICD-Pipeline) for full infrastructure docs.

---

## Security

| Concern | Implementation |
|---|---|
| API keys / tokens | AES-256-GCM encrypted at rest (backend), not persisted in browser storage |
| Database at rest | SQLCipher AES-256; key derived via PBKDF2 |
| PII before AI send | Rust-side detection + mandatory user approval in UI |
| Audit trail | Hash-chained audit entries (`prev_hash` + `entry_hash`) for tamper evidence |
| Network | `reqwest` with TLS; HTTP blocked by Tauri capability config |
| Capabilities | Least-privilege: scoped fs access, no arbitrary shell by default |
| CSP | Strict CSP in `tauri.conf.json`; no inline scripts |
| Telemetry | None — zero analytics, crash reporting, or usage tracking |

---

## Database

All data is stored locally in a SQLCipher-encrypted database at:

| OS | Path |
|---|---|
| Linux | `~/.local/share/trcaa/trcaa.db` |
| macOS | `~/Library/Application Support/trcaa/trcaa.db` |
| Windows | `%APPDATA%\trcaa\trcaa.db` |

Override with the `TFTSR_DATA_DIR` environment variable.

---

## Environment Variables

| Variable | Default | Purpose |
|---|---|---|
| `TFTSR_DATA_DIR` | Platform data dir | Override database location |
| `TFTSR_DB_KEY` | _(auto-generated)_ | Database encryption key override — auto-generated at first launch if unset |
| `TFTSR_ENCRYPTION_KEY` | _(auto-generated)_ | Credential encryption key override — auto-generated at first launch if unset |
| `RUST_LOG` | `info` | Tracing log level (`debug`, `info`, `warn`, `error`) |

---

## Implementation Status

| Phase | Description | Status |
|---|---|---|
| 1 | Scaffold & Foundation | ✅ Complete |
| 2 | Security & Database Layer | ✅ Complete |
| 3 | PII Sanitization Engine | ✅ Complete |
| 4 | AI Provider Layer | ✅ Complete |
| 5 | Ollama Integration | ✅ Complete |
| 6 | Log Upload & Analysis | ✅ Complete |
| 7 | 5-Whys Triage Engine | ✅ Complete |
| 8 | RCA & Post-Mortem Generation | ✅ Complete |
| 9 | History & Search | 🔲 Pending |
| 10 | Integrations (Confluence, ServiceNow, ADO) | 🔲 v0.2 |
| 11 | CI/CD Pipeline | ✅ Complete — Gitea Actions, all checks green |
| 12 | Release Packaging | ✅ linux/amd64 · linux/arm64 (native) · windows/amd64 |

---

## Support

If this tool has been useful to you, consider buying me a coffee!

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-buymeacoffee.com%2Ftftsr-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/tftsr)

---

## License

MIT © 2025 [Shaun Arman](https://github.com/sarman)

See [LICENSE](./LICENSE) for the full text. You are free to use, modify, and distribute this software — personal, commercial, or enterprise — as long as the original copyright notice is retained.
