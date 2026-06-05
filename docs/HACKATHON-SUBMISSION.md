# 2026 Hackathon Submission: TRCAA

**Developer**: Shaun Arman (VFK387)  
**ADO**: [#727547](https://dev.azure.com/tftsr/Apollo/_workitems/edit/727547)

---

## Problem to Solve

An alert fires, engineers swarm it, someone eventually finds the root cause, and then the post-mortem gets written from memory three days later with half the context already gone. The process loses information at every handoff.

**Pain points:**
- Manual command execution slows triage (copy terminal → paste → ask AI → repeat)
- Cloud SaaS RCA tools require uploading sensitive production data
- Generic AI assistants lack infrastructure domain expertise
- Post-mortems written days later miss critical context

---

## Our Solution

**TRCAA: A local-first, AI-powered incident triage assistant that autonomously executes diagnostic commands while you work.**

### Core Innovation: Agentic Shell Execution

The AI doesn't just suggest commands—it executes them with intelligent safety controls:

**Three-Tier Safety System:**
- **Tier 1 (Auto-Execute)**: Read-only diagnostics (`kubectl get`, `grep`) run immediately
- **Tier 2 (User Approval)**: Mutating operations (`kubectl scale`, `systemctl restart`) require consent
- **Tier 3 (Always Deny)**: Destructive commands (`rm -rf`, `shutdown`) blocked

**Example:** You say *"Why is the nginx pod crashing?"* — the AI autonomously runs `kubectl get pods`, `kubectl describe`, `kubectl logs`, analyzes the output, and explains the root cause. No copy-paste, no manual terminal work.

### What Makes TRCAA Unique

**Local-First Architecture:**
- SQLCipher AES-256 encrypted local storage (not cloud SaaS)
- Offline-capable via Ollama local AI models
- PII auto-detection and redaction before cloud API calls
- Tamper-evident hash-chained audit log

**Infrastructure Domain Expertise:**
- Pre-built expert context for 16 domains: Linux (RHEL/OEL), Windows, Kubernetes (k3s/OpenShift/Rancher), Networking (Fortigate/Cisco/Aruba), Databases (PostgreSQL/Redis/RabbitMQ), Proxmox, HPE Synergy/iLO, Observability (Kibana/Elasticsearch)

**Multi-Cluster Kubernetes:**
- Upload multiple kubeconfig files with AES-256-GCM encryption
- Bundled kubectl v1.30.0 (no external dependencies)

**Provider-Agnostic AI:**
- OpenAI, Anthropic Claude, Google Gemini, Mistral, AWS Bedrock (via LiteLLM), local Ollama
- Auto-detect tool calling support for custom providers
- No vendor lock-in

---

## What We Built

**Initial Hackathon (v1.0.0):** 35 files changed, +4089 lines
- Shell execution module with three-tier classifier (19 tests, 100% coverage)
- Real-time approval modal UI
- Cross-platform CI/CD with GitHub Actions

**Post-Hackathon Iterations (v1.0.1 → v1.0.9):** 24 PRs merged in 48 hours
- Security updates (vitest 4.1.8, postcss, vite)
- LiteLLM AWS Bedrock support
- Ollama auto-start + function calling support
- Query classification (prevents 20+ commands for simple questions)
- Connection reliability (180s timeout, health checks, 3-attempt retry)
- Tool calling auto-detect (eliminates guesswork about provider support)

**Total:** 25 PRs, ~84 files modified, ~6,100 lines, 431 tests passing, 72 hours

---

## The Competitive Landscape

### What Exists (Cloud SaaS)
- **Rootly**, **incident.io**, **Xurrent**: Cloud SaaS, subscription, data leaves network
- **TraceRoot** (AWS Marketplace): Cloud SaaS, compliance framing

**Critical gap:** Every competitor requires sensitive incident data to leave your network.

### What Doesn't Exist
**No tool combines:**
- Local-first + offline-capable + encrypted storage
- PII sanitization before AI send
- Provider-agnostic AI (6 providers + custom)
- Infrastructure domain depth (16 pre-built expert contexts)
- Autonomous command execution with safety controls
- Tamper-evident audit trail
- Air-gap capable (Ollama local models)

### Where We Win vs SaaS

| TRCAA | SaaS Competitors |
|-------|------------------|
| All data local, encrypted | Incident logs on vendor servers |
| Air-gap capable (Ollama) | Requires cloud |
| One-time install cost | Per-seat subscriptions |
| 16 pre-built infrastructure contexts | Generalist troubleshooting |
| 6 AI providers + custom | Vendor-locked backend |
| Auto-redact PII before send | Raw logs ingested |

**Where SaaS Wins:** Alert integration (PagerDuty/Datadog auto-triggers), team collaboration (multi-user), observability correlation

**Target market:** Regulated-industry DevOps teams, defense contractors, air-gapped environments, solo infrastructure engineers prioritizing privacy and cost over team collaboration.

---

## Technical Highlights

**Backend (Rust + Tauri):**
- Three-tier command classifier with pipe/chain analysis
- AES-256-GCM kubeconfig encryption
- Hash-chained audit log (tamper-evident)
- 297 backend tests

**Frontend (React + TypeScript):**
- Real-time approval modal with risk factor display
- Multi-cluster kubeconfig manager
- 134 frontend tests

**CI/CD (GitHub Actions):**
- Multi-platform builds: Linux (amd64/arm64), macOS (Intel/ARM), Windows
- kubectl binary auto-bundled
- Branch protection requires tests + Copilot review

---

## Impact

**Development:** 72 hours, 25 PRs, ~6,100 lines, 431 tests  
**Real-world:** Reduced troubleshooting from manual copy-paste loop to autonomous execution with sub-second command completion  
**Quality:** 3 rounds GitHub Copilot review (10 security/reliability findings, all resolved), zero Clippy warnings, zero TypeScript errors

---

## Try It

**Install:** [GitHub Releases](https://github.com/tftsr/apollo_nxt-trcaa/releases)  
**Quick Start:**
1. Upload kubeconfig via Settings
2. Create issue, select "Kubernetes" domain
3. Ask: *"What pods are in default namespace?"*
4. Watch AI autonomously execute `kubectl get pods -n default`

**No cloud required** — works fully offline with Ollama.

---

## Team Members We're Looking For

N/A (solo project)

---

## Fun Fact

This entire feature—from zero to production with 431 passing tests, 25 merged PRs, and comprehensive documentation—was built in 72 hours while maintaining zero Clippy warnings and zero TypeScript errors. The three-tier safety classifier has handled 100+ real diagnostic commands without a single false-positive denial.
