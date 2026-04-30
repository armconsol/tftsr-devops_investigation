# Changelog

All notable changes to TFTSR are documented here.
Commit types shown: feat, fix, perf, docs, refactor.
CI, chore, and build changes are excluded.

## [Unreleased]

### Bug Fixes
- **auto-tag**: Use correct tag range for release notes

### Chores
- Update CHANGELOG.md for v0.2.66 [skip ci]

### Features
- Add devops-incident-responder agent with domain auto-detection

## [0.2.68] — 2026-04-28

### Bug Fixes
- **ci**: Switch PR review from Ollama to liteLLM (qwen2.5-72b)
- **test**: Await async data in auditLog test to prevent race condition
- Harden timeline event input validation and atomic writes

### Documentation
- Update wiki for timeline events and incident response methodology
- Clarify changelog exclusion criteria

### Features
- Add timeline_events table, model, and CRUD commands
- Populate RCA and postmortem docs with real timeline data
- Wire incident response methodology into AI and record triage events

## [0.2.67] — 2026-04-28

### Bug Fixes
- **ci**: Update CHANGELOG.md for v0.2.66 [skip ci]

## [0.2.66] — 2026-04-20

### Bug Fixes
- **ci**: Switch PR review from Ollama to liteLLM (qwen2.5-72b)
- Harden timeline event input validation and atomic writes

### Documentation
- Update wiki for timeline events and incident response methodology
- Clarify changelog exclusion criteria

### Features
- Wire incident response methodology into AI and record triage events
- Populate RCA and postmortem docs with real timeline data

## [0.2.65] — 2026-04-15

### Bug Fixes
- Add --locked to cargo commands and improve version update script
- Remove invalid --locked flag from cargo commands and fix format string
- **integrations**: Security and correctness improvements
- Correct WIQL syntax and escape_wiql implementation

### Features
- Implement dynamic versioning from Git tags
- **integrations**: Implement query expansion for semantic search

### Security
- Fix query expansion issues from PR review
- Address all issues from automated PR review

## [0.2.63] — 2026-04-13

### Bug Fixes
- Add Windows nsis target and update CHANGELOG to v0.2.61

## [0.2.61] — 2026-04-13

### Bug Fixes
- Remove AppImage from upload artifact patterns

## [0.2.59] — 2026-04-13

### Bug Fixes
- Remove AppImage bundling to fix linux-amd64 build

## [0.2.57] — 2026-04-13

### Bug Fixes
- Add fuse dependency for AppImage support

### Refactoring
- Remove custom linuxdeploy install per CI CI uses tauri-downloaded version
- Revert to original Dockerfile without manual linuxdeploy installation

## [0.2.56] — 2026-04-13

### Bug Fixes
- Add missing ai_providers columns and fix linux-amd64 build
- Address AI review findings
- Address critical AI review issues

## [0.2.55] — 2026-04-13

### Bug Fixes
- **ci**: Use Gitea file API to push CHANGELOG.md — eliminates non-fast-forward rejection
- **ci**: Harden CHANGELOG.md API push step per review

## [0.2.54] — 2026-04-13

### Bug Fixes
- **ci**: Correct git-cliff archive path in tar extraction

## [0.2.53] — 2026-04-13

### Features
- **ci**: Add automated changelog generation via git-cliff

## [0.2.52] — 2026-04-13

### Bug Fixes
- **ci**: Add APPIMAGE_EXTRACT_AND_RUN to build-linux-amd64

## [0.2.51] — 2026-04-13

### Bug Fixes
- **ci**: Address AI review — rustup idempotency and cargo --locked
- **ci**: Replace docker:24-cli with alpine + docker-cli in build-images
- **docker**: Add ca-certificates to arm64 base image step 1
- **ci**: Resolve test.yml failures — Cargo.lock, updated test assertions
- **ci**: Address second AI review — || true, ca-certs, cache@v4, key suffixes

### Documentation
- **docker**: Expand rebuild trigger comments to include OpenSSL and Tauri CLI

### Performance
- **ci**: Use pre-baked images and add cargo/npm caching

## [0.2.50] — 2026-04-12

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

### Features
- Add automated PR review workflow with Ollama AI

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
- **ci**: Remove explicit docker.sock mount — act_runner mounts it automatically

## [0.2.36] — 2026-04-06

### Features
- **ci**: Add persistent pre-baked Docker builder images

## [0.2.35] — 2026-04-06

### Bug Fixes
- **ci**: Skip Ollama download on macOS build — runner has no access to GitHub binary assets
- **ci**: Remove all Ollama bundle download steps — use UI download button instead

### Refactoring
- **ollama**: Remove download/install buttons — show plain install instructions only

## [0.2.34] — 2026-04-06

### Bug Fixes
- **security**: Add path canonicalization and actionable permission error in install_ollama_from_bundle

### Features
- **ui**: Fix model dropdown, auth prefill, PII persistence, theme toggle, and Ollama bundle

## [0.2.33] — 2026-04-05

### Features
- **rebrand**: Rename binary to trcaa and auto-generate DB key

## [0.2.32] — 2026-04-05

### Bug Fixes
- **ci**: Restrict arm64 bundles to deb,rpm — skip AppImage

## [0.2.31] — 2026-04-05

### Bug Fixes
- **ci**: Set APPIMAGE_EXTRACT_AND_RUN=1 for arm64 AppImage bundling

## [0.2.30] — 2026-04-05

### Bug Fixes
- **ci**: Add make to arm64 host tools for OpenSSL vendored build

## [0.2.28] — 2026-04-05

### Bug Fixes
- **ci**: Use POSIX dot instead of source in arm64 build step

## [0.2.27] — 2026-04-05

### Bug Fixes
- **ci**: Remove GITHUB_PATH append that was breaking arm64 install step

## [0.2.26] — 2026-04-05

### Bug Fixes
- **ci**: Switch build-linux-arm64 to Ubuntu 22.04 with ports mirror

### Documentation
- Update CI pipeline wiki and add ticket summary for arm64 fix

## [0.2.25] — 2026-04-05

### Bug Fixes
- **ci**: Rebuild apt sources with per-arch entries before arm64 cross-compile install
- **ci**: Add workflow_dispatch and concurrency guard to auto-tag
- **ci**: Replace heredoc with printf in arm64 install step

## [0.2.24] — 2026-04-05

### Bug Fixes
- **ci**: Fix arm64 cross-compile, drop cargo install tauri-cli, move wiki-sync

## [0.2.23] — 2026-04-05

### Bug Fixes
- **ci**: Unblock release jobs and namespace linux artifacts by arch
- **security**: Harden secret handling and audit integrity
- **pii**: Remove lookahead from hostname regex, fix fmt in analysis test
- **security**: Enforce PII redaction before AI log transmission
- **ci**: Unblock release jobs and namespace linux artifacts by arch

## [0.2.22] — 2026-04-05

### Bug Fixes
- **ci**: Run linux arm release natively and enforce arm artifacts

## [0.2.21] — 2026-04-05

### Bug Fixes
- **ci**: Force explicit linux arm64 target for release artifacts

## [0.2.20] — 2026-04-05

### Refactoring
- **ci**: Remove standalone release workflow

## [0.2.19] — 2026-04-05

### Bug Fixes
- **ci**: Guarantee release jobs run after auto-tag
- **ci**: Use stable auto-tag job outputs for release fanout
- **ci**: Run post-tag release builds without job-output gating
- **ci**: Repair auto-tag workflow yaml so jobs trigger

## [0.2.18] — 2026-04-05

### Bug Fixes
- **ci**: Trigger release workflow from auto-tag pushes

## [0.2.17] — 2026-04-05

### Bug Fixes
- **ci**: Harden release asset uploads for reruns

## [0.2.16] — 2026-04-05

### Bug Fixes
- **ci**: Make release artifacts reliable across platforms

## [0.2.14] — 2026-04-04

### Bug Fixes
- Resolve macOS bundle path after app rename

## [0.2.13] — 2026-04-04

### Bug Fixes
- Resolve clippy uninlined_format_args in integrations and related modules
- Resolve clippy format-args failures and OpenSSL vendoring issue

### Features
- Add custom_rest provider mode and rebrand application name

## [0.2.12] — 2026-04-04

### Bug Fixes
- ARM64 build uses native target instead of cross-compile

## [0.2.11] — 2026-04-04

### Bug Fixes
- Persist integration settings and implement persistent browser windows

## [0.2.10] — 2026-04-03

### Features
- Complete webview cookie extraction implementation

## [0.2.9] — 2026-04-03

### Features
- Add multi-mode authentication for integrations (v0.2.10)

## [0.2.8] — 2026-04-03

### Features
- Add temperature and max_tokens support for Custom REST providers (v0.2.9)

## [0.2.7] — 2026-04-03

### Bug Fixes
- Use Wiki secret for authenticated wiki sync (v0.2.8)

### Documentation
- Update wiki for v0.2.6 - integrations and Custom REST provider

### Features
- Add automatic wiki sync to CI workflow (v0.2.7)

## [0.2.6] — 2026-04-03

### Bug Fixes
- Add user_id support and OAuth shell permission (v0.2.6)

## [0.2.5] — 2026-04-03

### Documentation
- Add Custom REST provider documentation

### Features
- Implement Confluence, ServiceNow, and Azure DevOps REST API clients
- Add Custom REST provider support

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
- Implement native DOCX export without pandoc dependency

### Features
- Add AI disclaimer modal before creating new issues

## [0.1.0] — 2026-04-03

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

### Security
- Rotate exposed token, redact from PLAN.md, add secret patterns to .gitignore


