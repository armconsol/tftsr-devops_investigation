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
| [Integrations](wiki/Integrations) | Confluence, ServiceNow, Azure DevOps |
| [Shell Execution](wiki/Shell-Execution) | Tiered shell safety, kubectl, kubeconfig, approvals |
| [Kubernetes Management](wiki/Kubernetes-Management) | Lens-style cluster management and terminals |
| [Database Management](wiki/Database-Management) | Connections, query history, bookmarks, import/export |
| [MCP Servers](wiki/MCP-Servers) | Discovery, tools, resources, and connection state |
| [Troubleshooting](wiki/Troubleshooting) | Known issues and fixes |

## Key Features

- **5-Whys AI Triage** — Interactive guided root cause analysis via multi-turn AI chat
- **PII Auto-Redaction** — Detects and redacts sensitive data before any AI send
- **Multi-Provider AI** — OpenAI, Anthropic Claude, Google Gemini, Mistral, AWS Bedrock (via LiteLLM), Custom REST gateways, local Ollama (fully offline)
- **Custom Provider Support** — Flexible authentication (Bearer, custom headers) and API formats (OpenAI-compatible, Custom REST)
- **Workflow Surfaces** — Confluence, ServiceNow, Azure DevOps, shell execution, Kubernetes, Proxmox, database management, and MCP
- **SQLCipher AES-256** — All issue history and credentials encrypted at rest
- **RCA + Post-Mortem Generation** — Auto-populated Markdown templates, exportable as MD/PDF
- **Ollama Management** — Hardware detection, model recommendations, in-app model management
- **Audit Trail** — Every external data send logged with SHA-256 hash
- **Domain-Specific Prompts** — 16 IT domains spanning infrastructure, security, automation, and identity
- **Image Attachments** — Upload and manage image files with PII detection and mandatory user approval

## Releases

| Version | Status | Highlights |
|---------|--------|-----------|
| v3.0.x | 🚀 Latest | Triage/RCA platform with integrations, shell execution, Kubernetes, Proxmox, database management, and MCP |
| v1.1.0 | Released | Kubernetes Management UI with PTY terminals, metrics, port forwarding, YAML editor |
| v1.0.1 | Released | Domain prompt fix, UI contrast improvements, ARM64 Linux build |
| v1.0.0 | Released | Core application with PII detection, Shell Execution, 5-Whys AI triage |
| v0.2.6 | Released | Custom REST AI gateway support, OAuth2 shell permissions, user ID tracking |
| v0.2.5 | Released | Image attachments with PII detection and approval workflow |
| v0.2.3 | Released | Confluence/ServiceNow/ADO REST API clients (19 TDD tests) |
| v0.1.1 | Released | Core application with PII detection, RCA generation |

**Platforms:** linux/amd64 · linux/arm64 · windows/amd64 (.deb, .rpm, .AppImage, .exe, .msi)

Download from [Releases](https://gogs.tftsr.com/sarman/tftsr-devops_investigation/releases). All builds are produced natively (no QEMU emulation).

## Project Status

| Area | Status |
|------|--------|
| Core triage, history/search, integrations, shell, Kubernetes, Proxmox, database, MCP | ✅ Complete |
| CI/CD | ✅ Complete — Gitea Actions fully operational |
| Release packaging | ✅ linux/amd64 · linux/arm64 (native) · windows/amd64 |

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
