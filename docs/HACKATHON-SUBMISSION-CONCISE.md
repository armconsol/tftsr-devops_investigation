# 2026 Hackathon: TRCAA

**Developer**: Shaun Arman (VFK387) | **ADO**: [#727547](https://dev.azure.com/msi-cie/Apollo/_workitems/edit/727547)

---

## Problem to Solve

An alert fires, engineers swarm it, someone finds the root cause, and the post-mortem gets written from memory three days later with half the context gone. The process loses information at every handoff. Current pain: manual command execution slows triage (copy terminal → paste → ask AI → repeat), cloud SaaS tools require uploading sensitive production data, generic AI lacks infrastructure expertise.

---

## Our Solution

**TRCAA: Local-first AI-powered incident triage that autonomously executes diagnostic commands.**

### Core Innovation: Agentic Shell Execution
The AI doesn't suggest commands—it executes them with intelligent safety:

**Three-Tier Safety:**
- **Tier 1**: Read-only (`kubectl get`, `grep`) auto-execute
- **Tier 2**: Mutating (`kubectl scale`) require approval
- **Tier 3**: Destructive (`rm -rf`) auto-blocked

**Example:** *"Why is nginx pod crashing?"* → AI runs `kubectl get/describe/logs`, analyzes output, explains root cause. No copy-paste.

### Unique Features
- **Local-first**: SQLCipher AES-256 encrypted storage, offline via Ollama, PII auto-redact, tamper-evident audit
- **Domain expertise**: 16 pre-built contexts (Linux RHEL/OEL, Windows, K8s, networking, databases, Proxmox, HPE, observability)
- **Multi-cluster K8s**: Encrypted kubeconfig storage, bundled kubectl v1.30.0
- **Provider-agnostic**: OpenAI, Claude, Gemini, Mistral, Bedrock, Ollama + auto-detect tool calling

---

## What We Built

**v1.0.0** (44 hrs): 35 files, +4089 lines, shell execution module, three-tier classifier (19 tests/100% coverage), approval modal UI, CI/CD

**v1.0.1-v1.0.9** (28 hrs, 24 PRs in 48 hrs): Security updates, LiteLLM Bedrock, Ollama auto-start + function calling, query classification (prevents AI over-investigation), connection reliability (180s timeout, health checks, retry logic), tool calling auto-detect

**Total**: 25 PRs, ~84 files, ~6,100 lines, 431 tests, 72 hours

---

## Competitive Landscape

**SaaS exists**: Rootly, incident.io, Xurrent, TraceRoot—all cloud, subscriptions, data leaves network

**TRCAA uniquely combines**: Local-first + offline + encrypted + PII sanitization + provider-agnostic (6 providers) + 16 domain contexts + autonomous shell execution + tamper-evident audit + air-gap capable

**We win on**: Privacy (local encrypted), air-gap (Ollama), cost (no per-seat fees), domain depth  
**SaaS wins on**: Alert integration (PagerDuty/Datadog), team collaboration, observability correlation

**Target**: Regulated industries, defense, air-gapped environments, privacy-focused teams

---

## Technical Highlights

**Backend (Rust)**: Three-tier classifier with pipe/chain analysis, AES-256-GCM encryption, hash-chained audit, 297 tests  
**Frontend (React)**: Real-time approval modal, multi-cluster manager, 134 tests  
**CI/CD**: Multi-platform builds (Linux amd64/arm64, macOS, Windows), kubectl bundled, branch protection

**Quality**: 3 rounds Copilot review (10 findings resolved), zero Clippy warnings, zero TypeScript errors

---

## Impact

**Development**: 72 hours, 25 PRs, ~6,100 lines, 431 tests  
**Real-world**: Reduced triage from manual copy-paste loop to autonomous sub-second execution  
**Security**: 3 Copilot security findings resolved (prompt injection, tool call dropping, sanitization)

---

## Try It

[GitHub Releases](https://github.com/msicie/apollo_nxt-trcaa/releases) → Upload kubeconfig → Ask *"What pods in default namespace?"* → Watch AI auto-execute. Works fully offline with Ollama.

---

## Fun Fact

Zero to production with 431 passing tests, 25 PRs, comprehensive docs in 72 hours. Zero Clippy warnings. Zero TypeScript errors. 100+ real commands executed without a single false-positive denial.
