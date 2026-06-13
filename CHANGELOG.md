# Changelog

All notable changes to TRCAA are documented here.
Commit types shown: feat, fix, perf, docs, refactor.
CI, chore, and build changes are excluded.

## [1.2.1] — 2026-06-13

### Bug Fixes
- Proxmox PDM v1.2.0 bugs and feature parity
- Implement v1.2.1 fixes
- Persist Proxmox settings via localStorage; fix Remotes add/refresh flow
- **fmt**: Apply rustfmt formatting to proxmox commands

### Features
- Move auto-updater to Settings > Updater; collapse Proxmox nav by default
- Add missing proxmox backend client functions and Rust command stubs
- **proxmox**: Implement notes system, resource search, and administration panel (phases 12-13)
- **proxmox**: Implement HA groups manager and user management UI (phases 8-9)
- **proxmox**: Implement certificate manager and subscription registry (phases 10-11)
- **proxmox**: Implement network management, tasks, custom views, and connection health (phases 14-15)
- **proxmox**: Add routes for notes, search, and administration pages

## [1.2.0] — 2026-06-11

### Bug Fixes
- **lint**: Resolve ESLint errors
- **changelog**: Only include current tag commits in release body
- **workflow**: Remove duplicate else block in changelog generation
- **fmt**: Format code with cargo fmt
- Address PR review findings
- Address PR review findings
- Implement proper kubeconfig parsing and validation
- Implement kubeconfig parsing and add kubeconfig storage
- **fmt**: Format code with cargo fmt
- Address clippy warnings
- **fmt**: Format code with cargo fmt
- **changelog**: Use tag range for release notes
- **fmt**: Apply cargo fmt
- Address automated PR review findings
- Address all automated PR review findings
- Properly handle kubectl subprocess with async child management
- Address automated PR review findings
- Add shutdown_port_forwards command for app cleanup
- Add app shutdown cleanup for port forward processes
- **kubernetes**: Address automated PR review findings
- **kube**: Address portforward race condition and temp file leak
- **kube**: Resolve automated PR review blockers and warnings
- **ci**: Replace JS-based Renovate action with direct container invocation
- Use public Gitea URL in test workflow
- **kubernetes**: Address PR #76 review findings
- **kubernetes**: Remove redundant TS cast and fix cargo fmt failures
- **kubernetes**: Use kubeconfig files from Settings instead of duplicate cluster management
- **kubernetes**: Sync active kubeconfig to store's selectedClusterId
- **ci**: Generate per-release changelog body using positional range arg
- **ci**: Exclude internal migration commits from changelog
- **kube**: Bridge kubeconfig storage to in-memory cluster map and fix UI issues
- **classifier**: Fix 3 safety bugs, extract const arrays, make tier UI dynamic
- **kube**: Correct kubectl context, dialog close, icon visibility, cluster name
- **kube**: Use current-context for kubectl auth; fix SelectValue label display
- **kube**: Switch to --kubeconfig flag; add Test Connection diagnostic
- **ui**: Correct font contrast and background colors in dark mode
- **kube**: Add two-stage diagnostics to test_kubectl_connection
- **ci**: Cargo fmt kube.rs + switch pr-review to qwen3-coder-next
- **kube**: Unique temp kubeconfig paths — eliminate concurrent-call race condition
- **ui**: Replace hardcoded colors with semantic Tailwind vars for dark mode
- **kube**: WorkloadOverview loads data; single connect on mount; visible error on failure
- **kube**: Add namespace to PodInfo; pod actions use pod.namespace not filter
- **kube**: Network/config/storage list actions use item.namespace not filter prop
- **kube**: Workload list actions use item.namespace not filter prop
- **performance**: Resolve memory leaks and add polish features
- **ui**: Critical UI fixes - logs, menus, dark mode, YAML
- **lint**: Remove unused variables in test files
- Add PTY command bindings and format Rust code
- **shell**: Resolve TypeScript errors in PTY terminal components
- **security**: Address automated code review findings
- **shell**: Delay KubeconfigGuard disarm until after PTY session starts
- **ci**: Correct Renovate API endpoint for Gitea
- **kube**: Fix PTY param names, ansi-to-react ESM interop, and dark mode badges
- **kube**: Configure Monaco for offline use and fix pod column data (IP/Node/CPU/Memory)
- **fmt**: Collapse single-expression restart count closure per rustfmt

### Documentation
- **kubernetes**: Add comment about dynamic port allocation limitation
- Update documentation for Kubernetes Management UI
- Add ticket summary for kube action namespace and stability fixes
- Update to v1.1.0 release with Kubernetes Management UI
- Add Proxmox implementation documentation
- Update Proxmox implementation documentation for v1.2.0
- Add Proxmox implementation summary
- Add Proxmox PDM feature parity completion summary

### Features
- **kube**: Add Kubernetes management GUI components
- **kube**: Implement delete_port_forward command
- **kube**: Implement complete kubectl port-forward runtime
- Add comprehensive Windows and Linux command support to shell classifier
- **kubernetes**: Add database persistence for clusters and port_forwards
- **k8s**: Implement clean-room Kubernetes management GUI
- Implement full Lens-like Kubernetes UI with resource discovery and management
- Implement additional Kubernetes resource discovery and management commands
- Add Kubernetes Management Implementation Plan
- **kubernetes**: Implement Phase 1 & 2: resource discovery UIs and advanced features
- **kubernetes**: Implement Phase 3 - detail views and cluster management
- **kubernetes**: Implement Phase 7 - real-time updates
- **kubernetes**: Implement Lens Desktop v5 feature-parity UI
- **kube**: Add TypeScript types and command stubs for all new K8s resources
- **kube**: Nav restructure, action menus, new resource lists, advanced components
- **kube**: Implement 44 new Rust K8s commands + helm binary support
- **kube**: Merge backend — 44 Rust commands, helm binary, 363 tests
- **network**: Add dedicated port forwarding management page
- **workloads**: Add logs action to all 7 workload resource types
- **config**: Add edit/delete actions to all policy resources and secret viewer
- **shell**: Implement PTY-based interactive terminals
- **tables**: Implement configurable columns infrastructure
- **metrics**: Add frontend metrics integration with Chart.js
- **metrics**: Implement kubectl top metrics backend
- **tables**: Roll out configurable columns to all workload lists
- **kube**: Add YAML edit action to NamespaceList
- Implement Proxmox cluster management foundation
- Implement VM management operations for Proxmox VE
- Implement Proxmox Backup Server operations
- Implement Ceph management operations for Proxmox VE
- Implement SDN management operations for Proxmox VE
- Implement Firewall management operations for Proxmox VE
- Implement HA groups management operations for Proxmox VE
- Implement Update management operations for Proxmox VE
- Implement Proxmox Datacenter Manager feature parity - Phases 1-11
- Implement remaining PDM features - Phases 12-15
- Add missing PDM UI components for feature parity
- Implement 100% Proxmox PDM feature parity - UI components

### Security
- **kube**: Restrict temp kubeconfig files to owner-only permissions

## [1.1.0] — 2026-06-06

### Bug Fixes
- **ci**: Use public rust:1.82-bookworm image instead of custom image
- Revert incorrect sanitization - use 172.0.0.29 for CI runners
- Remove GitHub-specific files and fix remaining URLs
- Update tests to use .gitea workflows and disable GitHub-specific tests
- Comprehensive trcaa→tftsr conversion and URL corrections
- Remove remaining proprietary references and fix branding
- Remove ALL remaining proprietary references (MSI/Vesta/VNXT)
- **ci**: Remove actions/cache steps to fix Node.js requirement
- **ci**: Install rustfmt and clippy components in workflows
- **ci**: Upgrade Rust from 1.82 to 1.83 for edition2024 support
- **ci**: Use Rust nightly for edition2024 dependency support
- **ci**: Install Tauri system dependencies in nightly containers
- **ci**: Remove kubectl from externalBin to fix CI build
- **clippy**: Fix Rust nightly clippy lints
- Align Tauri npm packages with Rust crate versions
- Pin plugin-stronghold npm version to match Rust crate (2.3.1)

### Features
- **kube**: Add Kubernetes management support

## [0.3.12] — 2026-06-05

### Bug Fixes
- **ci**: Fix YAML syntax error in test.yml
- Address valid PR review findings
- Add missing @testing-library/dom dependency and fix clippy warning

### Documentation
- Add ADRs for shell safety, MCP transport, kubectl bundling
- Update wiki with shell execution, Ollama function calling, and CI/CD changes
- Add v1.0.7 and v1.0.8 release notes

### Features
- Add three-tier shell execution with kubectl support
- Add shell execution database migrations (migrations #24-28)
- Add Ollama function calling and tool calling auto-detection
- Add shell execution and kubeconfig management UI
- Add kubectl binary bundling for cross-platform support

## [0.3.11] — 2026-06-01

### Bug Fixes
- **mcp**: Treat missing resources/list as non-fatal for servers that don't implement it

### Documentation
- **wiki**: Update MCP-Servers.md with env var support, PATH requirement, and new schema column

## [0.3.10] — 2026-06-01

### Bug Fixes
- **mcp**: Add env encryption to store layer
- **mcp**: Parse and merge env vars in discovery layer
- **mcp**: Add environment variable and HTTP header support for MCP servers
- **mcp**: Improve UX clarity for encrypted env vars during edit
- **mcp**: Change plaintext env input to type=text
- **mcp**: Add validation to block dangerous environment variables
- **mcp**: Fix test_allows_safe_env_vars test failure

## [0.3.9] — 2026-06-01

### Bug Fixes
- **security**: Expand Password PII patterns; add regression tests

## [0.3.8] — 2026-06-01

### Bug Fixes
- **security**: Block PII in chat attachments and typed messages
- **security**: Address PR review — move attachment handling to backend, auto-redact PII
- **security**: Backend-only PII redaction; fix fmt CI failure
- **security**: Frontend attachment scan notice, bubble redaction update, fmt fix
- **security**: Full-content PII scan, clippy, IPC null fix, scan size cap
- Audit PII redaction metadata, safe bubble update, update ticket

## [0.3.7] — 2026-05-31

### Bug Fixes
- Address PR review findings — compress errors, size guard, modal error display

### Features
- Attachment DB storage and cross-incident recall

## [0.3.6] — 2026-05-31

### Bug Fixes
- **ci**: Push detached HEAD to master using HEAD:master refspec
- **ci**: Consolidate all auto-tag changelog fixes

## [0.3.5] — 2026-05-31

### Bug Fixes
- **ci**: Changelog job creates release to avoid race with build jobs
- **ci**: Verify tag exists locally before running git-cliff

## [0.3.4] — 2026-05-31

### Bug Fixes
- **ci**: Pass release_tag as job output; fix equal-version case; drop git-describe [skip ci]
- **ai,search**: Load history across all conversations; deep search related tables
- **ci**: Reduce AI review hallucinations in pr-review workflow
- **agentic**: Inline format arg in writeln! to satisfy clippy::uninlined_format_args
- **ci**: Rewrite pr-review to send full file contents instead of diffs
- **ci**: Fix secret scrubbing regex that was deleting legitimate code lines
- **ci**: Add post-generation evidence verification to pr-review
- **ci**: Add codebase index to prompt; verify findings against full repo
- **ci**: Fix backtick command substitution crash in pr-review prompt
- **ci**: Remove concurrency group that silently dropped pr-review runs
- **ci**: Replace heredoc with printf to fix YAML block scalar breakage
- **ci**: Fix grep invalid range and printf invalid option in pr-review
- **ci**: Remove remaining printf -- calls in Analyze with LLM step
- **ci**: Use printf '%s' form to avoid format strings starting with hyphen
- **ci**: Write curl body to file to avoid ARG_MAX limit
- **ci**: Install python3 in pr-review container (ubuntu:22.04 omits it)
- **sudo**: Enforce username scope and singleton row in sudo_config

### Documentation
- **analysis**: Document zip-slip safety guarantee in extract_docx_text

### Features
- **upload**: Add safe file extension validation and binary text extraction

## [0.3.3] — 2026-05-23

### Bug Fixes
- Resolve all clippy lints (uninlined format args, range::contains, push_str single chars)
- Inline format args for Rust 1.88 clippy compatibility
- Retain GPU-VRAM-eligible models in recommender even when RAM is low
- Use alpine/git with explicit checkout for tag-based release builds
- Set CI=true for cargo tauri build — Woodpecker sets CI=woodpecker which Tauri CLI rejects
- Arm64 cross-compilation — add multiarch pkg-config sysroot setup
- Remove arm64 from release pipeline — webkit2gtk multiarch conflict on x86_64 host
- Write artifacts to workspace (shared between steps), not /artifacts/
- Upload step needs gogs_default network to reach Gogs API (host firewall blocks default bridge)
- Use bundled-sqlcipher-vendored-openssl for portable Windows cross-compilation
- Add make to windows build step (required by vendored OpenSSL)
- Replace empty icon placeholder files with real app icons
- Suppress MinGW auto-export to resolve Windows DLL ordinal overflow
- Use when: platform: for arm64 step routing (Woodpecker 0.15.4 compat)
- Remove unused tauri-plugin-cli causing startup crash
- Use $GITHUB_REF_NAME env var instead of ${{ github.ref_name }} expression
- Remove unused tauri-plugin-updater + SQLCipher 16KB page size
- Prevent WebKit/GTK system theme from overriding input text colors on Linux
- Set SQLCipher cipher_page_size BEFORE first database access
- Button text visibility, toggle contrast, create_issue IPC, ad-hoc codesign
- Dropdown text invisible on macOS + correct codesign order for DMG
- Add explicit text-foreground to SelectTrigger, SelectValue, and SelectItem
- Ollama detection, install guide UI, and AI Providers auto-fill
- Provider test FK error, model pull white screen, RECOMMENDED badge
- Provider routing uses provider_type, Active badge, fmt
- Navigate to /logs after issue creation, fix dashboard category display
- Dashboard shows — while loading, exposes errors, adds refresh button
- ListIssuesCmd was sending {query} but Rust expects {filter} — caused dashboard to always show 0 open issues
- Arm64 linux cross-compilation — add multiarch and pkg-config env vars
- Close from chat works before issue loads; save user reason as resolution step; dynamic version
- DomainPrompts closing brace too early; arm64 use native platform image
- UI contrast issues and ARM64 build failure
- Remove Woodpecker CI and fix Gitea Actions ARM64 build
- UI visibility issues, export errors, filtering, and audit log enhancement
- ARM64 build native compilation instead of cross-compilation
- Improve release artifact upload error handling
- Install jq in Linux/Windows build containers
- Improve download button visibility and add DOCX export
- Implement native DOCX export without pandoc dependency
- Improve Cancel button contrast in AI disclaimer modal
- Add user_id support and OAuth shell permission (v0.2.6)
- Use Wiki secret for authenticated wiki sync (v0.2.8)
- Persist integration settings and implement persistent browser windows
- ARM64 build uses native target instead of cross-compile
- Resolve clippy uninlined_format_args in integrations and related modules
- Resolve clippy format-args failures and OpenSSL vendoring issue
- Resolve macOS bundle path after app rename
- **ci**: Make release artifacts reliable across platforms
- **ci**: Harden release asset uploads for reruns
- **ci**: Trigger release workflow from auto-tag pushes
- **ci**: Guarantee release jobs run after auto-tag
- **ci**: Use stable auto-tag job outputs for release fanout
- **ci**: Run post-tag release builds without job-output gating
- **ci**: Repair auto-tag workflow yaml so jobs trigger
- **ci**: Force explicit linux arm64 target for release artifacts
- **ci**: Run linux arm release natively and enforce arm artifacts
- **ci**: Unblock release jobs and namespace linux artifacts by arch
- **security**: Harden secret handling and audit integrity
- **pii**: Remove lookahead from hostname regex, fix fmt in analysis test
- **security**: Enforce PII redaction before AI log transmission
- **ci**: Unblock release jobs and namespace linux artifacts by arch
- **ci**: Fix arm64 cross-compile, drop cargo install tauri-cli, move wiki-sync
- **ci**: Rebuild apt sources with per-arch entries before arm64 cross-compile install
- **ci**: Add workflow_dispatch and concurrency guard to auto-tag
- **ci**: Replace heredoc with printf in arm64 install step
- **ci**: Switch build-linux-arm64 to Ubuntu 22.04 with ports mirror
- **ci**: Remove GITHUB_PATH append that was breaking arm64 install step
- **ci**: Use POSIX dot instead of source in arm64 build step
- **ci**: Add make to arm64 host tools for OpenSSL vendored build
- **ci**: Set APPIMAGE_EXTRACT_AND_RUN=1 for arm64 AppImage bundling
- **ci**: Restrict arm64 bundles to deb,rpm — skip AppImage
- **security**: Add path canonicalization and actionable permission error in install_ollama_from_bundle
- **ci**: Skip Ollama download on macOS build — runner has no access to GitHub binary assets
- **ci**: Remove all Ollama bundle download steps — use UI download button instead
- **ci**: Remove explicit docker.sock mount — act_runner mounts it automatically
- **db,auth**: Auto-generate encryption keys for release builds
- **lint**: Use inline format args in auth.rs
- **lint**: Resolve all clippy warnings for CI compliance
- **fmt**: Apply rustfmt formatting to webview_fetch.rs
- **types**: Replace normalizeApiFormat() calls with direct value
- Fix encryption test race condition with parallel tests
- OpenWebUI provider connection and missing command registrations
- Force single test thread for Rust tests to eliminate race conditions
- Add @types/testing-library__react for TypeScript compilation
- Use 'provider' argument name to match Rust command signature
- Lint fixes and formatting cleanup
- Add missing ai_providers migration (014)
- Rename GITEA_TOKEN to TF_TOKEN to comply with naming restrictions
- Remove actions/checkout to avoid Node.js dependency
- Use ubuntu container with git installed
- Use actions/checkout with token auth and self-hosted runner
- Use IP addresses for internal services
- Simplified workflow syntax
- Add debugging output for Ollamaresponse
- Correct Ollama URL, API endpoint, and JSON construction in pr-review workflow
- Add diagnostics to identify empty Ollama response root cause
- Use bash shell and remove bash-only substring expansion in pr-review
- Restore migration 014, bump version to 0.2.50, harden pr-review workflow
- Harden pr-review workflow and sync versions to 0.2.50
- Configure container DNS to resolve ollama-ui.tftsr.com
- Harden pr-review workflow — URLs, DNS, correctness and reliability
- Resolve AI review false positives and address high/medium issues
- Replace github.server_url with hardcoded gogs.tftsr.com for container access
- Revert to two-dot diff — three-dot requires merge base unavailable in shallow clone
- Harden pr-review workflow — secret redaction, log safety, auth header
- **ci**: Address AI review — rustup idempotency and cargo --locked
- **ci**: Replace docker:24-cli with alpine + docker-cli in build-images
- **docker**: Add ca-certificates to arm64 base image step 1
- **ci**: Resolve test.yml failures — Cargo.lock, updated test assertions
- **ci**: Address second AI review — || true, ca-certs, cache@v4, key suffixes
- **ci**: Add APPIMAGE_EXTRACT_AND_RUN to build-linux-amd64
- **ci**: Correct git-cliff archive path in tar extraction
- **ci**: Use Gitea file API to push CHANGELOG.md — eliminates non-fast-forward rejection
- **ci**: Harden CHANGELOG.md API push step per review
- Add missing ai_providers columns and fix linux-amd64 build
- Address AI review findings
- Address critical AI review issues
- Add fuse dependency for AppImage support
- Remove AppImage bundling to fix linux-amd64 build
- Remove AppImage from upload artifact patterns
- Add Windows nsis target and update CHANGELOG to v0.2.61
- Add --locked to cargo commands and improve version update script
- Remove invalid --locked flag from cargo commands and fix format string
- **integrations**: Security and correctness improvements
- Correct WIQL syntax and escape_wiql implementation
- Harden timeline event input validation and atomic writes
- **ci**: Switch PR review from Ollama to liteLLM (qwen2.5-72b)
- **test**: Await async data in auditLog test to prevent race condition
- **auto-tag**: Use correct tag range for release notes
- **auto-tag**: Use tea CLI instead of hardcoded tokens
- **ci**: Use qwen3-coder-next model for PR review
- **mcp**: Add timeouts, delete audit log, OAuth state nonce; improve PR review prompt
- **ci**: Replace tea with curl, honour Cargo.toml version [skip ci]
- **ci**: Replace tea CLI with curl in changelog steps; read Cargo.toml for version
- Bump tauri.conf.json version to 0.3.0

### Documentation
- Update PLAN.md with accurate implementation status
- Add CLAUDE.md with development guidance
- Add wiki source files and CI auto-sync pipeline
- Update PLAN.md - Phase 11 complete, redact token references
- Update README and wiki for v0.1.0-alpha release
- Remove broken arm64 CI step, document Woodpecker 0.15.4 limitation
- Update README and wiki for Gitea Actions migration
- Update README, wiki, and UI version to v0.1.1
- Add LiteLLM + AWS Bedrock integration guide
- Add Custom REST provider documentation
- Update wiki for v0.2.6 - integrations and Custom REST provider
- Update CI pipeline wiki and add ticket summary for arm64 fix
- **architecture**: Add C4 diagrams, ADRs, and architecture overview
- Add AGENTS.md and SECURITY_AUDIT.md
- **docker**: Expand rebuild trigger comments to include OpenSSL and Tauri CLI
- Update wiki for timeline events and incident response methodology
- Clarify changelog exclusion criteria
- Add v0.2.66 changelog entry
- Update CHANGELOG.md for v0.2.68
- Update CHANGELOG.md for v0.2.69-v0.2.71
- Update CHANGELOG.md for v0.2.71

### Features
- Initial implementation of TFTSR IT Triage & RCA application
- Add Windows amd64 cross-compile to release pipeline; add arm64 QEMU agent
- Add native linux/arm64 release build step
- Add macOS arm64 act_runner and release build job
- Auto-increment patch tag on every merge to master
- Inline file/screenshot attachment in triage chat
- Close issues, restore history, auto-save resolution steps
- Expand domains to 13 — add Telephony, Security/Vault, Public Safety, Application, Automation/CI-CD
- Add HPE, Dell, Identity domains + expand k8s/security/observability/VESTA NXT
- Add AI disclaimer modal before creating new issues
- Add database schema for integration credentials and config
- Implement OAuth2 token exchange and AES-256-GCM encryption
- Add OAuth2 Tauri commands for integration authentication
- Implement OAuth2 callback server with automatic token exchange
- Add OAuth2 frontend UI and complete integration flow
- Implement Confluence, ServiceNow, and Azure DevOps REST API clients
- Add Custom REST provider support
- Add automatic wiki sync to CI workflow (v0.2.7)
- Add temperature and max_tokens support for Custom REST providers (v0.2.9)
- Add multi-mode authentication for integrations (v0.2.10)
- Complete webview cookie extraction implementation
- Add custom_rest provider mode and rebrand application name
- **rebrand**: Rename binary to trcaa and auto-generate DB key
- **ui**: Fix model dropdown, auth prefill, PII persistence, theme toggle, and Ollama bundle
- **ci**: Add persistent pre-baked Docker builder images
- **ai**: Add tool-calling and integration search as AI data source
- Add image attachment support with PII detection
- Support GenAI datastore file uploads and fix paste image upload
- Add automated PR review workflow with Ollama AI
- **ci**: Add automated changelog generation via git-cliff
- Implement dynamic versioning from Git tags
- **integrations**: Implement query expansion for semantic search
- Add timeline_events table, model, and CRUD commands
- Populate RCA and postmortem docs with real timeline data
- Wire incident response methodology into AI and record triage events
- **ai**: Add devops-incident-responder agent with domain auto-detection
- **mcp**: Add MCP Server Support with TDD implementation

### Performance
- **ci**: Use pre-baked images and add cargo/npm caching

### Refactoring
- **ci**: Remove standalone release workflow
- **ollama**: Remove download/install buttons — show plain install instructions only
- Remove custom linuxdeploy install per CI CI uses tauri-downloaded version
- Revert to original Dockerfile without manual linuxdeploy installation

### Security
- Rotate exposed token, redact from PLAN.md, add secret patterns to .gitignore
- Fix query expansion issues from PR review
- Address all issues from automated PR review

### Update
- Node_modules from npm install


