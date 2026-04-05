# Fix: build-linux-arm64 — Switch to Ubuntu 22.04 with ports mirror

## Description

The `build-linux-arm64` CI job failed repeatedly with
`E: Unable to correct problems, you have held broken packages` during the
Install dependencies step. Root cause: `rust:1.88-slim` (Debian Bookworm) uses a single
mirror for all architectures. When both `[arch=amd64]` and `[arch=arm64]` entries point at
the same Debian repo, apt's dependency resolver hits unavoidable conflicts — the `binary-all`
package index is duplicated and certain `-dev` package pairs cannot be co-installed because
they lack `Multi-Arch: same`. This is a structural Debian single-mirror multiarch limitation
that cannot be fixed by tweaking `sources.list`.

Ubuntu 22.04 solves this by routing arm64 through a separate mirror:
`ports.ubuntu.com/ubuntu-ports`. amd64 and arm64 packages come from entirely different repos,
eliminating all cross-arch index overlaps and resolution conflicts.

## Acceptance Criteria

- `build-linux-arm64` Install dependencies step completes without apt errors
- `ubuntu:22.04` is the container image for the arm64 job
- Ubuntu's `ports.ubuntu.com/ubuntu-ports` is used for arm64 packages
- `libayatana-appindicator3-dev:arm64` is removed (no tray icon in this app)
- Rust is installed via `rustup` (not pre-installed in Ubuntu base)
- All 51 frontend tests pass
- YAML is syntactically valid

## Work Implemented

### `.gitea/workflows/auto-tag.yml`

- **Container**: `rust:1.88-slim` → `ubuntu:22.04` for `build-linux-arm64` job
- **Install dependencies step**: Full replacement
  - Step 1: Host tools + aarch64 cross-compiler (amd64 packages, installed before multiarch registration)
  - Step 2: Register arm64 architecture; `sed` existing `sources.list` entries to `[arch=amd64]`; add `arm64-ports.list` pointing at `ports.ubuntu.com/ubuntu-ports jammy`
  - Step 3: ARM64 dev libs (`libwebkit2gtk-4.1-dev`, `libssl-dev`, `libgtk-3-dev`, `librsvg2-dev`) — `libayatana-appindicator3-dev:arm64` removed
  - Step 4: Node.js via NodeSource
  - Step 5: Rust 1.88.0 via `rustup --no-modify-path`; `$HOME/.cargo/bin` appended to `$GITHUB_PATH`
- **Build step**: Added `source "$HOME/.cargo/env"` as first line (belt-and-suspenders for Rust PATH)

### `tests/unit/releaseWorkflowCrossPlatformArtifacts.test.ts`

- Added new test: `"uses Ubuntu 22.04 with ports mirror for arm64 cross-compile"` — asserts workflow contains `ubuntu:22.04`, `ports.ubuntu.com/ubuntu-ports`, and `jammy`
- All previously passing assertions continue to pass (build step env vars and upload paths unchanged)

### `docs/wiki/CICD-Pipeline.md`

- `build-linux-arm64` job entry now mentions Ubuntu 22.04 + ports mirror
- New Known Issue entry: **Debian Multiarch Breaks arm64 Cross-Compile** — documents the root cause and the Ubuntu 22.04 fix for future reference

## Testing Needed

- [ ] YAML validation: `python3 -c "import yaml; yaml.safe_load(open('.gitea/workflows/auto-tag.yml'))" && echo OK` — **PASSED**
- [ ] Frontend tests: `npm run test:run` — **51/51 PASSED** (50 existing + 1 new)
- [ ] CI integration: Push branch → merge PR → observe `build-linux-arm64` Install dependencies step completes without `held broken packages` error
- [ ] Verify arm64 `.deb`, `.rpm`, `.AppImage` artifacts are uploaded to the Gitea release
