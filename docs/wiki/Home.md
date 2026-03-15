# TFTSR — IT Triage & RCA Desktop Application

**TFTSR** is a secure desktop application for guided IT incident triage, root cause analysis (RCA), and post-mortem documentation. Built with Tauri 2.x (Rust + WebView) and React 18.

## Quick Navigation

| Topic | Description |
|-------|-------------|
| [Architecture](wiki/Architecture) | Backend, frontend, and data flow |
| [Development Setup](wiki/Development-Setup) | Prerequisites, commands, environment |
| [Database](wiki/Database) | Schema, migrations, encryption |
| [AI Providers](wiki/AI-Providers) | Supported providers and configuration |
| [PII Detection](wiki/PII-Detection) | Patterns, redaction flow, security |
| [IPC Commands](wiki/IPC-Commands) | Full list of Tauri backend commands |
| [CI/CD Pipeline](wiki/CICD-Pipeline) | Woodpecker CI + Gogs setup |
| [Security Model](wiki/Security-Model) | Encryption, audit trail, capabilities |
| [Integrations](wiki/Integrations) | Confluence, ServiceNow, Azure DevOps (v0.2) |
| [Troubleshooting](wiki/Troubleshooting) | Known issues and fixes |

## Key Features

- **5-Whys AI Triage** — Interactive guided root cause analysis via multi-turn AI chat
- **PII Auto-Redaction** — Detects and redacts sensitive data before any AI send
- **Multi-Provider AI** — OpenAI, Anthropic Claude, Google Gemini, Mistral, local Ollama (fully offline)
- **SQLCipher AES-256** — All issue history encrypted at rest
- **RCA + Post-Mortem Generation** — Auto-populated Markdown templates, exportable as MD/PDF
- **Ollama Management** — Hardware detection, model recommendations, in-app model management
- **Audit Trail** — Every external data send logged with SHA-256 hash
- **Domain-Specific Prompts** — 8 IT domains: Linux, Windows, Network, Kubernetes, Databases, Virtualization, Hardware, Observability

## Project Status

| Phase | Status |
|-------|--------|
| Phases 1–8 (Core) | ✅ Complete |
| Phase 9 (History/Search FTS) | 🔄 Partially integrated |
| Phase 10 (Integrations) | 🕐 v0.2 stubs only |
| Phase 11 (CLI) | 🕐 Planned |
| Phase 12 (Release packaging) | 🔄 Linux done; macOS/Windows pending |

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | Tauri 2.x |
| Backend | Rust (async/await, tokio) |
| Frontend | React 18 + TypeScript + Vite |
| Styling | Tailwind CSS + custom components |
| Database | rusqlite + SQLCipher (AES-256) |
| Secret storage | tauri-plugin-stronghold |
| State | Zustand |
| Testing | Vitest (frontend) + `#[cfg(test)]` (Rust) |
| CI/CD | Woodpecker CI v0.15.4 + Gogs |
