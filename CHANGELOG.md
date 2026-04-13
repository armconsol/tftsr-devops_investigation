# Changelog

All notable changes to TFTSR are documented here.
Commit types shown: feat, fix, perf, docs, refactor.
CI, chore, and build changes are excluded.

## [Unreleased]

### Bug Fixes
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

### Documentation
- **docker**: Expand rebuild trigger comments to include OpenSSL and Tauri CLI

### Features
- Add automated PR review workflow with Ollama AI

### Performance
- **ci**: Use pre-baked images and add cargo/npm caching

## [0.2.49] — 2026-04-10

### Bug Fixes
- Add missing ai_providers migration (014)

## [0.2.48] — 2026-04-10

### Bug Fixes
- Lint fixes and formatting cleanup

### Features
- Support GenAI datastore file uploads and fix paste image upload

## [0.2.47] — 2026-04-09

### Bug Fixes
- Use 'provider' argument name to match Rust command signature

## [0.2.46] — 2026-04-09

### Bug Fixes
- Add @types/testing-library__react for TypeScript compilation

### Update
- Node_modules from npm install

## [0.2.45] — 2026-04-09

### Bug Fixes
- Force single test thread for Rust tests to eliminate race conditions

## [0.2.43] — 2026-04-09

### Bug Fixes
- Fix encryption test race condition with parallel tests
- OpenWebUI provider connection and missing command registrations

### Features
- Add image attachment support with PII detection

## [0.2.42] — 2026-04-07

### Documentation
- Add AGENTS.md and SECURITY_AUDIT.md

## [0.2.41] — 2026-04-07

### Bug Fixes
- **db,auth**: Auto-generate encryption keys for release builds
- **lint**: Use inline format args in auth.rs
- **lint**: Resolve all clippy warnings for CI compliance
- **fmt**: Apply rustfmt formatting to webview_fetch.rs
- **types**: Replace normalizeApiFormat() calls with direct value

### Documentation
- **architecture**: Add C4 diagrams, ADRs, and architecture overview

### Features
- **ai**: Add tool-calling and integration search as AI data source

## [0.2.40] — 2026-04-06

### Bug Fixes
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

### Documentation
- Add Custom REST provider documentation
- Update wiki for v0.2.6 - integrations and Custom REST provider
- Update CI pipeline wiki and add ticket summary for arm64 fix

### Features
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

### Refactoring
- **ci**: Remove standalone release workflow
- **ollama**: Remove download/install buttons — show plain install instructions only

## [0.2.4] — 2026-04-03

### Features
- Implement OAuth2 token exchange and AES-256-GCM encryption
- Add OAuth2 Tauri commands for integration authentication
- Implement OAuth2 callback server with automatic token exchange
- Add OAuth2 frontend UI and complete integration flow

## [0.2.3] — 2026-04-03

### Bug Fixes
- Improve Cancel button contrast in AI disclaimer modal

### Features
- Add database schema for integration credentials and config

## [0.2.1] — 2026-04-03

### Bug Fixes
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

### Documentation
- Add LiteLLM + AWS Bedrock integration guide

### Features
- Add macOS arm64 act_runner and release build job
- Auto-increment patch tag on every merge to master
- Inline file/screenshot attachment in triage chat
- Close issues, restore history, auto-save resolution steps
- Expand domains to 13 — add Telephony, Security/Vault, Public Safety, Application, Automation/CI-CD
- Add HPE, Dell, Identity domains + expand k8s/security/observability/VESTA NXT
- Add AI disclaimer modal before creating new issues

## [0.1.1] — 2026-03-30

### Bug Fixes
- Remove unused tauri-plugin-updater + SQLCipher 16KB page size
- Prevent WebKit/GTK system theme from overriding input text colors on Linux
- Set SQLCipher cipher_page_size BEFORE first database access

### Documentation
- Update README, wiki, and UI version to v0.1.1

## [0.1.0] — 2026-03-29

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

### Documentation
- Update PLAN.md with accurate implementation status
- Add CLAUDE.md with development guidance
- Add wiki source files and CI auto-sync pipeline
- Update PLAN.md - Phase 11 complete, redact token references
- Update README and wiki for v0.1.0-alpha release
- Remove broken arm64 CI step, document Woodpecker 0.15.4 limitation
- Update README and wiki for Gitea Actions migration

### Features
- Add Windows amd64 cross-compile to release pipeline; add arm64 QEMU agent
- Add native linux/arm64 release build step

### Security
- Rotate exposed token, redact from PLAN.md, add secret patterns to .gitignore

## [0.1.0-test] — 2026-03-15

### Features
- Initial implementation of TFTSR IT Triage & RCA application


