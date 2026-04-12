# CI Runner Speed Optimization via Pre-baked Images + Caching

## Description

Every CI run (both `test.yml` and `auto-tag.yml`) was installing system packages from scratch
on each job invocation: `apt-get update`, Tauri system libs, Node.js via nodesource, and in
the arm64 job — a full `rustup` install. This was the primary cause of slow builds.

The repository already contains pre-baked builder Docker images (`.docker/Dockerfile.*`) and a
`build-images.yml` workflow to push them to the local Gitea registry at `172.0.0.29:3000`.
These images were never referenced by the actual CI jobs — a critical gap. This work closes
that gap and adds `actions/cache@v3` for Cargo and npm.

## Acceptance Criteria

- [ ] `Dockerfile.linux-amd64` includes `rustfmt` and `clippy` components
- [ ] `Dockerfile.linux-arm64` includes `rustfmt` and `clippy` components
- [ ] `test.yml` Rust jobs use `172.0.0.29:3000/sarman/trcaa-linux-amd64:rust1.88-node22`
- [ ] `test.yml` Rust jobs have no inline `apt-get` or `rustup component add` steps
- [ ] `test.yml` Rust jobs include `actions/cache@v3` for `~/.cargo/registry`
- [ ] `test.yml` frontend jobs include `actions/cache@v3` for `~/.npm`
- [ ] `auto-tag.yml` `build-linux-amd64` uses pre-baked `trcaa-linux-amd64` image
- [ ] `auto-tag.yml` `build-windows-amd64` uses pre-baked `trcaa-windows-cross` image
- [ ] `auto-tag.yml` `build-linux-arm64` uses pre-baked `trcaa-linux-arm64` image
- [ ] All three build jobs have no `Install dependencies` step
- [ ] All three build jobs include `actions/cache@v3` for Cargo and npm
- [ ] `docs/wiki/CICD-Pipeline.md` documents pre-baked images, cache keys, and server prerequisites
- [ ] `build-images.yml` triggered manually before merging to ensure images exist in registry

## Work Implemented

### `.docker/Dockerfile.linux-amd64`
Added `RUN rustup component add rustfmt clippy` after the existing target add line.
The `rust-fmt-check` and `rust-clippy` CI jobs now rely on these being pre-installed
in the image rather than installing them at job runtime.

### `.docker/Dockerfile.linux-arm64`
Added `&& /root/.cargo/bin/rustup component add rustfmt clippy` appended to the
existing `rustup` installation RUN command (chained with `&&` to keep it one layer).

### `.gitea/workflows/test.yml`
- **rust-fmt-check**, **rust-clippy**, **rust-tests**: switched container image from
  `rust:1.88-slim` → `172.0.0.29:3000/sarman/trcaa-linux-amd64:rust1.88-node22`.
  Removed `apt-get install git` from Checkout steps (git is pre-installed in image).
  Removed `apt-get install libwebkit2gtk-...` steps.
  Removed `rustup component add rustfmt` and `rustup component add clippy` steps.
  Added `actions/cache@v3` step for `~/.cargo/registry/index`, `~/.cargo/registry/cache`,
  `~/.cargo/git/db` keyed on `Cargo.lock` hash.
- **frontend-typecheck**, **frontend-tests**: kept `node:22-alpine` image (no change needed).
  Added `actions/cache@v3` step for `~/.npm` keyed on `package-lock.json` hash.

### `.gitea/workflows/auto-tag.yml`
- **build-linux-amd64**: image `rust:1.88-slim` → `trcaa-linux-amd64:rust1.88-node22`.
  Removed Checkout apt-get install git, removed entire Install dependencies step.
  Removed `rustup target add x86_64-unknown-linux-gnu` from Build step. Added cargo + npm cache.
- **build-windows-amd64**: image `rust:1.88-slim` → `trcaa-windows-cross:rust1.88-node22`.
  Removed Checkout apt-get install git, removed entire Install dependencies step.
  Removed `rustup target add x86_64-pc-windows-gnu` from Build step.
  Added cargo (with `-windows-` suffix key to avoid collision) + npm cache.
- **build-linux-arm64**: image `ubuntu:22.04` → `trcaa-linux-arm64:rust1.88-node22`.
  Removed Checkout apt-get install git, removed entire Install dependencies step (~40 lines).
  Removed `. "$HOME/.cargo/env"` (PATH already set via `ENV` in Dockerfile).
  Removed `rustup target add aarch64-unknown-linux-gnu` from Build step.
  Added cargo (with `-arm64-` suffix key) + npm cache.

### `docs/wiki/CICD-Pipeline.md`
Added two new sections before the Test Pipeline section:
- **Pre-baked Builder Images**: table of all three images and their contents, rebuild
  triggers, how-to-rebuild instructions, and the insecure-registries Docker daemon
  prerequisite for 172.0.0.29.
- **Cargo and npm Caching**: documents the `actions/cache@v3` key patterns in use,
  including the per-platform cache key suffixes for cross-compile jobs.
Updated the Test Pipeline section to reference the correct pre-baked image name.
Updated the Release Pipeline job table to show which image each build job uses.

## Testing Needed

1. **Pre-build images** (prerequisite): Trigger `build-images.yml` via `workflow_dispatch`
   on Gitea Actions UI. Confirm all 3 images are pushed and visible in the registry.

2. **Server prerequisite**: Confirm `/etc/docker/daemon.json` on `172.0.0.29` contains
   `{"insecure-registries":["172.0.0.29:3000"]}` and Docker was restarted after.

3. **PR test suite**: Open a PR with these changes. Verify:
   - All 5 test jobs pass (`rust-fmt-check`, `rust-clippy`, `rust-tests`,
     `frontend-typecheck`, `frontend-tests`)
   - Job logs show no `apt-get` or `rustup component add` output
   - Cache hit messages appear on second run

4. **Release build**: Merge to master. Verify `auto-tag.yml` runs and:
   - All 3 Linux/Windows build jobs start without Install dependencies step
   - Artifacts are produced and uploaded to the Gitea release
   - Total release time is significantly reduced (~7 min vs ~25 min before)

5. **Expected time savings after caching warms up**:
   | Job | Before | After |
   |-----|--------|-------|
   | rust-fmt-check | ~2 min | ~20 sec |
   | rust-clippy | ~4 min | ~45 sec |
   | rust-tests | ~5 min | ~1.5 min |
   | frontend-typecheck | ~2 min | ~30 sec |
   | frontend-tests | ~3 min | ~40 sec |
   | build-linux-amd64 | ~10 min | ~3 min |
   | build-windows-amd64 | ~12 min | ~4 min |
   | build-linux-arm64 | ~15 min | ~4 min |
   | PR test total (parallel) | ~5 min | ~1.5 min |
   | Release total | ~25 min | ~7 min |
