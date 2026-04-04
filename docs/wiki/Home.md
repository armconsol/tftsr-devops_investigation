# Troubleshooting and RCA Assistant

**Troubleshooting and RCA Assistant** is a secure desktop application for guided IT incident triage, root cause analysis (RCA), and post-mortem documentation. Built with Tauri 2.x (Rust + WebView) and React 18.

**CI:** ![build](http://172.0.0.29:3000/sarman/tftsr-devops_investigation/actions/workflows/test.yml/badge.svg) — rustfmt · clippy · 64 Rust tests · tsc · vitest — all green

## Quick Navigation

| Topic | Description |
|-------|-------------|
| [Architecture](wiki/Architecture) | Backend, frontend, and data flow |
| [Development Setup](wiki/Development-Setup) | Prerequisites, commands, environment |
| [Database](wiki/Database) | Schema, migrations, encryption |
| [AI Providers](wiki/AI-Providers) | Supported providers and configuration |
| [LiteLLM + Bedrock Setup](wiki/LiteLLM-Bedrock-Setup) | AWS Bedrock integration via LiteLLM proxy |
| [PII Detection](wiki/PII-Detection) | Patterns, redaction flow, security |
| [IPC Commands](wiki/IPC-Commands) | Full list of Tauri backend commands |
| [CI/CD Pipeline](wiki/CICD-Pipeline) | Gitea Actions setup, multi-platform builds, act_runner config |
| [Security Model](wiki/Security-Model) | Encryption, audit trail, capabilities |
| [Integrations](wiki/Integrations) | Confluence, ServiceNow, Azure DevOps (v0.2) |
| [Troubleshooting](wiki/Troubleshooting) | Known issues and fixes |

## Key Features

- **5-Whys AI Triage** — Interactive guided root cause analysis via multi-turn AI chat
- **PII Auto-Redaction** — Detects and redacts sensitive data before any AI send
- **Multi-Provider AI** — OpenAI, Anthropic Claude, Google Gemini, Mistral, AWS Bedrock (via LiteLLM), MSI GenAI (Motorola internal), local Ollama (fully offline)
- **Custom Provider Support** — Flexible authentication (Bearer, custom headers) and API formats (OpenAI-compatible, Custom REST)
- **External Integrations** — Confluence, ServiceNow, Azure DevOps with OAuth2 PKCE flows
- **SQLCipher AES-256** — All issue history and credentials encrypted at rest
- **RCA + Post-Mortem Generation** — Auto-populated Markdown templates, exportable as MD/PDF
- **Ollama Management** — Hardware detection, model recommendations, in-app model management
- **Audit Trail** — Every external data send logged with SHA-256 hash
- **Domain-Specific Prompts** — 8 IT domains: Linux, Windows, Network, Kubernetes, Databases, Virtualization, Hardware, Observability

## Releases

| Version | Status | Highlights |
|---------|--------|-----------|
| v0.2.6 | 🚀 Latest | MSI GenAI support, OAuth2 shell permissions, user ID tracking |
| v0.2.3 | Released | Confluence/ServiceNow/ADO REST API clients (19 TDD tests) |
| v0.1.1 | Released | Core application with PII detection, RCA generation |

**Platforms:** linux/amd64 · linux/arm64 · windows/amd64 (.deb, .rpm, .AppImage, .exe, .msi)

Download from [Releases](https://gogs.tftsr.com/sarman/tftsr-devops_investigation/releases). All builds are produced natively (no QEMU emulation).

## Project Status

| Phase | Status |
|-------|--------|
| Phases 1–8 (Core application) | ✅ Complete |
| Phase 9 (History/Search) | 🔲 Pending |
| Phase 10 (Integrations) | ✅ Complete — Confluence, ServiceNow, Azure DevOps fully implemented with OAuth2 |
| Phase 11 (CI/CD) | ✅ Complete — Gitea Actions fully operational |
| Phase 12 (Release packaging) | ✅ linux/amd64 · linux/arm64 (native) · windows/amd64 |

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
| Testing | Vitest (13 frontend) + `#[cfg(test)]` (64 Rust tests) |
| CI/CD | Gitea Actions (act_runner v0.3.1) + Gitea |
