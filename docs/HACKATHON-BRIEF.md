# 2026 Hackathon Submission: TRCAA

**Project**: TRCAA (Troubleshooting and RCA Assistant)  
**Feature**: Autonomous AI-Powered Incident Triage with Shell Command Execution  
**Developer**: Shaun Arman (VFK387)  
**ADO Work Item**: [#727547](https://dev.azure.com/tftsr/Apollo/_workitems/edit/727547)

---

## Problem to Solve

An alert fires, engineers swarm it, someone eventually finds the root cause, and then the post-mortem gets written from memory three days later with half the context already gone. The process loses information at every handoff. 

**Current workflow pain points:**
- Incident context scattered across Slack, PagerDuty, logs, and memory
- Manual command execution slows triage (copy terminal output → paste → ask AI → repeat)
- Cloud SaaS RCA tools require uploading sensitive production data
- Generic AI assistants lack infrastructure domain expertise
- Post-mortems written days later miss critical context

---

## Our Solution

**TRCAA is a local-first, AI-powered incident triage assistant that autonomously executes diagnostic commands while you work.**

### Core Innovation: Agentic Shell Execution
The AI doesn't just suggest commands—it executes them directly with intelligent safety controls:

**Three-Tier Safety System:**
- **Tier 1 (Auto-Execute)**: Read-only diagnostics (`kubectl get`, `grep`, `ps`) run immediately
- **Tier 2 (User Approval)**: Mutating operations (`kubectl scale`, `systemctl restart`) require explicit consent
- **Tier 3 (Always Deny)**: Destructive commands (`rm -rf`, `shutdown`) automatically blocked

**Example:** You say *"Why is the nginx pod crashing?"* — the AI autonomously runs `kubectl get pods`, `kubectl describe`, and `kubectl logs`, analyzes the output, and explains the root cause. No copy-paste, no manual terminal work.

### Key Differentiators

**Local-First Architecture:**
- SQLCipher AES-256 encrypted local storage (not cloud SaaS)
- Offline-capable via Ollama local AI models
- PII auto-detection and redaction before any cloud API calls
- Tamper-evident hash-chained audit log

**Infrastructure Domain Expertise:**
- Pre-built expert context for 16 domains: Linux (RHEL/OEL), Windows, Kubernetes (k3s/OpenShift/Rancher), Networking (Fortigate/Cisco/Aruba), Databases (PostgreSQL/Redis/RabbitMQ), Proxmox, HPE Synergy/iLO, Observability (Kibana/Elasticsearch)
- AI understands your stack's specifics, not generic troubleshooting

**Multi-Cluster Kubernetes Support:**
- Upload multiple kubeconfig files with encrypted AES-256-GCM storage
- Bundled kubectl v1.30.0 (no external dependencies)
- Switch contexts seamlessly during triage

**Provider-Agnostic AI:**
- OpenAI, Anthropic Claude, Google Gemini, Mistral, AWS Bedrock (via LiteLLM), local Ollama
- Auto-detect tool calling support for custom providers
- No vendor lock-in

---

## What We Built (v1.0.0 → v1.0.9)

### Initial Hackathon Release (v1.0.0)
**35 files changed, +4089 lines**
- Shell execution module with three-tier classifier (19 tests, 100% coverage)
- kubectl binary bundling for all platforms
- Real-time approval modal UI
- 4 new database tables (migrations 024-027)
- 7 Tauri commands + 1 AI tool registration
- Cross-platform CI/CD with GitHub Actions

### Post-Hackathon Iterations (v1.0.1 → v1.0.9)
**24 additional PRs merged in 48 hours**, addressing real-world usage issues:

**v1.0.1-v1.0.2**: Security updates (vitest 4.1.8, postcss, vite), LiteLLM AWS Bedrock support, Ollama auto-start  
**v1.0.3-v1.0.4**: Query classification (prevents AI from running 20+ commands for simple questions), graceful iteration limit handling, TFTSR GenAI gateway support  
**v1.0.5-v1.0.6**: Agent prompt cleanup (fixed JSON output in natural language responses)  
**v1.0.7**: Ollama function calling support (tools parameter was ignored)  
**v1.0.8**: Connection reliability (180s timeout, health checks, 3-attempt retry logic), model recommendations (≥3B parameters required)  
**v1.0.9** (PR #44, in review): Auto-detect tool calling support—eliminates guesswork about whether custom AI providers support function calling

**Total impact:** 60 files modified, ~6,100 lines of production code, 297 backend + 134 frontend tests passing

---

## The Competitive Landscape

### What Exists (Cloud SaaS)
- **Rootly**: Automates postmortem/RCA process (cloud SaaS, subscription)
- **incident.io**: Triaging/investigating alerts in Slack/Teams (cloud SaaS, data leaves network)
- **Xurrent**: Auto-compiles postmortems from logs/metrics (cloud SaaS)
- **TraceRoot** (AWS Marketplace): 5-step investigation with AI assist (cloud SaaS, compliance framing)

**Critical gap:** Every competitor is cloud-hosted SaaS requiring sensitive incident data to leave your network.

### What Doesn't Exist
**No tool combines:**
- Local-first + offline-capable execution
- Encrypted local storage (SQLCipher AES-256)
- PII sanitization before AI send
- Provider-agnostic AI (swap models without workflow changes)
- Infrastructure domain depth (16 pre-built expert contexts)
- Autonomous command execution with safety controls
- Tamper-evident audit trail
- Air-gap capable (via Ollama local models)

**TRCAA occupies this unique gap.**

### Where We Win vs SaaS
| Dimension | TRCAA | SaaS Competitors |
|-----------|-------|------------------|
| **Privacy** | All data local, encrypted | Incident logs on vendor servers |
| **Air-gap capable** | Yes (Ollama local models) | No (requires cloud) |
| **Cost** | One-time install | Per-seat subscription fees |
| **Domain depth** | 16 pre-built infrastructure contexts | Generalist troubleshooting |
| **Provider choice** | 6 AI providers + custom | Vendor-locked backend |
| **PII protection** | Auto-redact before send | Raw logs ingested |
| **Compliance** | Hash-chained audit trail | Varies by vendor |

### Where SaaS Wins
- **Alert integration**: PagerDuty/Datadog/CloudWatch auto-triggers (TRCAA is manually initiated)
- **Team collaboration**: Multiple engineers on same incident simultaneously (TRCAA is single-user)
- **Observability correlation**: Tight integration with metrics/traces (incident.io cuts context-switching from 15min → 30sec)

**Target market:** Regulated-industry DevOps teams, defense contractors, small MSPs, air-gapped environments, solo infrastructure engineers who prioritize privacy and cost over team collaboration features.

---

## Technical Highlights

**Backend (Rust + Tauri):**
- Three-tier command classifier with pipe/chain analysis and tier escalation
- Platform-specific shell execution (`cmd /C` on Windows, `sh -c` on Unix)
- AES-256-GCM kubeconfig encryption with hand-rolled YAML parser (licensing constraints)
- 30-second command timeout with environment isolation (strips `AWS_ACCESS_KEY_ID`, etc.)
- Hash-chained audit log (tamper-evident)

**Frontend (React + TypeScript):**
- Real-time approval modal with risk factor display
- Multi-cluster kubeconfig manager with drag-drop upload
- Execution history with exit codes and timing
- Settings UI for tier architecture visualization

**CI/CD (GitHub Actions):**
- Multi-platform builds: Linux (amd64/arm64 DEB/RPM), macOS (Intel/ARM DMG), Windows (NSIS)
- kubectl binary auto-bundled for all platforms
- Branch protection requires passing tests + Copilot review before merge

**Quality Assurance:**
- 297 backend tests + 134 frontend tests (100% classifier coverage)
- 3 rounds of GitHub Copilot automated review (10 security/reliability findings, all resolved)
- Zero Clippy warnings, zero TypeScript errors
- TDD approach throughout

---

## Lessons Learned

### What Went Well
- TDD caught bugs early (19 classifier tests prevented regressions)
- Three-tier classification proved robust in real usage
- GitHub Copilot review identified real security issues (prompt injection risk, tool call dropping)
- Rapid iteration post-launch (24 PRs in 48 hours) addressed real user pain points

### What We'd Improve
- Should have built multi-context kubeconfig support in v1.0.0 (added v1.0.9)
- Domain prompts initially didn't instruct AI to use shell execution tool (fixed v1.0.1)
- Integration tests need more coverage (mostly unit tests currently)
- Should have updated hackathon summary after each PR merge (created documentation debt)

### Challenges Solved
1. **Cross-platform shell execution**: `sh -c` doesn't exist on Windows → platform-specific shell selection with `cfg!` macros
2. **AI over-investigation**: Simple query "What pods are running?" triggered 20+ commands → three-tier query classification (Simple/Diagnostic/Incident)
3. **Ollama function calling**: Provider ignored `tools` parameter → implemented proper tool formatting in request body
4. **Connection reliability**: Intermittent timeouts → extended timeout (180s for tool calling), health checks, 3-attempt retry logic
5. **Tool calling detection**: Users unsure if custom providers support it → auto-detect with test tool call (v1.0.9)

---

## Impact Metrics

**Development Time:**
- Initial hackathon (v1.0.0): ~44 hours
- Post-release iterations (v1.0.1-v1.0.9): ~28 hours
- **Total: ~72 hours**

**Code Produced:**
- Rust: ~2,200 lines (shell module + commands + AI improvements)
- TypeScript/React: ~900 lines (components + types)
- Tests: ~800 lines (431 tests total)
- Documentation: ~2,200 lines (wiki + summaries)
- **Total: ~6,100 lines**

**PRs Merged:** 25 PRs (v1.0.0 initial + 24 post-release iterations)

**Real-World Usage:** Reduced troubleshooting time from "copy terminal output → paste → ask AI → repeat" loop to autonomous execution with sub-second command completion.

---

## Future Roadmap

**Immediate (v1.1.0):**
- Multi-context kubeconfig support (currently first context only)
- PII blocking mode (auto-escalate to Tier 2 when PII detected)
- Command templates (pre-defined diagnostic runbooks)

**Near-term (v1.2.0):**
- Team collaboration (multi-user on same incident)
- Alert integration (PagerDuty/Datadog webhooks auto-open issues)
- Execution rollback (undo last command where possible)

**Long-term:**
- Terraform/Ansible command support
- Database query execution (read-only mode)
- Log streaming (tail -f equivalent)
- SSH agent integration for direct remote execution

---

## Documentation Delivered

- **docs/wiki/Shell-Execution.md**: 700+ line comprehensive guide (architecture, API reference, 6 manual integration tests, troubleshooting)
- **docs/wiki/AI-Providers.md**: Provider comparison, tool calling compatibility matrix
- **docs/2026-HACKATHON-SUMMARY.md**: 940-line detailed project chronicle
- **CLAUDE.md**: Updated architecture documentation
- **.github/COPILOT_SETUP.md**: Code review configuration
- **docs/v1.0.{1-8}-summary.md**: Per-version release notes

---

## Try It Yourself

**Install:** Download from [GitHub Releases](https://github.com/tftsr/apollo_nxt-trcaa/releases)  
**Quick Start:**
1. Upload a kubeconfig via Settings → Kubeconfig Manager
2. Create new issue, select "Kubernetes" domain
3. Ask: *"What pods are in the default namespace?"*
4. Watch the AI autonomously execute `kubectl get pods -n default` and explain the results

**No cloud required** — works fully offline with Ollama local models.

---

## Team Members We're Looking For

N/A (solo project)

---

**Fun Fact:** This entire feature—from zero to production with 431 passing tests, 25 merged PRs, and comprehensive documentation—was built in 72 hours while maintaining zero Clippy warnings and zero TypeScript errors. The three-tier safety classifier has handled 100+ real diagnostic commands without a single false-positive denial.
