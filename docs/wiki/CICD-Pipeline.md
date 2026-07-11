# CI/CD Pipeline

## Infrastructure

| Component | URL | Notes |
|-----------|-----|-------|
| Gitea | `https://gogs.tftsr.com` | Git server (migrated from Gogs 0.14) |
| Gitea Actions (direct) | `http://gitea.tftsr.com:8084` | v2.x |
| Gitea Actions (proxy) | `http://gitea.tftsr.com:8085` | nginx reverse proxy |

### CI Agents

| Agent | Platform | Host | Purpose |
|-------|----------|------|---------|
| `gitea_act_runner_amd64` (Docker) | `linux-amd64` | gitea.tftsr.com | Native x86_64 — test builds + amd64/windows release |
| `act_runner` (systemd) | `linux-arm64` | gitea.tftsr.com | Native aarch64 — arm64 release builds |
| `act_runner` (launchd) | `macos-arm64` | sarman's local Mac | Native Apple Silicon — macOS `.dmg` release builds |

Agent labels configured in `~/.config/act_runner/config.yaml`:
```yaml
runner:
  labels:
    - "macos-arm64:host"
```
macOS runner runs jobs **directly on the host** (no Docker container) — macOS SDK cannot run in Docker.

---

## Pre-baked Builder Images

CI build and test jobs use pre-baked Docker images pushed to the internal Gitea container
registry (`${GITEA_REGISTRY}`). These images bake in all system dependencies (Tauri libs,
Node.js, Rust toolchain, cross-compilers) so that CI jobs skip package installation entirely.

| Image tag | Used by jobs | Contents |
|-----------|-------------|----------|
| `tftsr-linux-amd64:rust1.89-node22` | `rust-fmt-check`, `rust-clippy`, `rust-tests`, `build-linux-amd64` | Rust 1.89 + rustfmt + clippy + Tauri amd64 libs + Node.js 22 |
| `tftsr-windows-cross:rust1.89-node22` | `build-windows-amd64` | Rust 1.89 + mingw-w64 + NSIS + Node.js 22 |
| `tftsr-linux-arm64:rust1.89-node22` | `build-linux-arm64` | Rust 1.89 + aarch64 cross-toolchain + arm64 multiarch libs + Node.js 22 |

Full image paths follow the pattern `${GITEA_REGISTRY}/sarman/<tag>` where `GITEA_REGISTRY`
is configured in the act_runner environment.

**Rebuild triggers:** Rust toolchain version bump, webkit2gtk/gtk major version change, Node.js major version change.

**linux-amd64 libsodium:** `Dockerfile.linux-amd64` sets `SODIUM_LIB_DIR=/usr/lib/x86_64-linux-gnu` so that `libsodium-sys-stable` (pulled in via `tauri-plugin-stronghold`) links against the pre-installed `libsodium-dev` static library instead of downloading and compiling libsodium from source at cargo build time. `make` is also installed as a fallback for any future C build deps.

**How to rebuild images:**
1. Trigger `build-images.yml` via `workflow_dispatch` in the Gitea Actions UI
2. Confirm all 3 images appear in the Gitea container registry
3. Only then merge workflow changes that depend on the new image contents

### Bumping the Rust toolchain version

When upgrading the Rust version used in CI (e.g. 1.89 → 1.90), all of the following must
be updated in a single PR and the images **must be built before** the PR is merged:

1. `.docker/Dockerfile.linux-amd64` — `FROM rust:<VER>-slim`
2. `.docker/Dockerfile.windows-cross` — `FROM rust:<VER>-slim`
3. `.docker/Dockerfile.linux-arm64` — `--default-toolchain <VER>.0`
4. `.gitea/workflows/build-images.yml` — `docker build -t` and `docker push` tag lines
5. `.gitea/workflows/release-beta.yml` — `container: image:` lines
6. `.gitea/workflows/auto-tag.yml` — `container: image:` lines

**Before merging the PR:**
- Manually dispatch `build-images.yml` on the `beta` branch via the Gitea Actions UI:
  navigate to the repo → **Actions** → **Build CI Docker Images** → **Run workflow** → select branch `beta` → click Run.
- Wait for the run to finish and confirm the three new images appear in the registry.
- Only then merge the PR.

> **Why manual dispatch?** `build-images.yml` only auto-triggers on `master` pushes (stable
> releases). `beta` is the primary development branch and uses `workflow_dispatch` for
> on-demand image rebuilds. Gitea 1.22 does not expose the workflow dispatch API endpoint,
> so this step must be performed through the web UI.

`scripts/validate-ci-images.sh` (run in CI by `test.yml`) catches version drift between
Dockerfiles and workflow files — but it cannot verify that the images have actually been
pushed to the registry.

**Server prerequisite — insecure registry** (one-time, on each act_runner host):
```sh
echo '{"insecure-registries":["${GITEA_REGISTRY}"]}' | sudo tee /etc/docker/daemon.json
sudo systemctl restart docker
```
This must be configured on every machine running an act_runner for the runner's Docker
daemon to pull from the internal HTTP registry. Replace `${GITEA_REGISTRY}` with the
actual registry host configured for this environment.

---

## Cargo and npm Caching

All Rust and build jobs use `actions/cache@v4` to cache downloaded package artifacts.
Gitea 1.22 implements the Gitea Actions cache API natively.

**Cargo cache** (Rust jobs):
```yaml
- name: Cache cargo registry
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry/index
      ~/.cargo/registry/cache
      ~/.cargo/git/db
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-
```

**npm cache** (frontend and build jobs):
```yaml
- name: Cache npm
  uses: actions/cache@v4
  with:
    path: ~/.npm
    key: ${{ runner.os }}-npm-${{ hashFiles('**/package-lock.json') }}
    restore-keys: |
      ${{ runner.os }}-npm-
```

Cache keys for cross-compile jobs use a suffix to avoid collisions:
- Windows build: `${{ runner.os }}-cargo-windows-${{ hashFiles('**/Cargo.lock') }}`
- arm64 build: `${{ runner.os }}-cargo-arm64-${{ hashFiles('**/Cargo.lock') }}`
- macOS build: `${{ runner.os }}-cargo-macos-${{ hashFiles('**/Cargo.lock') }}`

The macOS release build also caches `src-tauri/target/aarch64-apple-darwin` and runs with
`timeout-minutes: 120` so the full Rust compile has room to complete on a cold cache.

---

## Test Pipeline (`.gitea/workflows/test.yml`)

**Triggers:** Pull requests only.

```
Pipeline jobs (run in parallel):
  1. rust-fmt-check     → cargo fmt --check
  2. rust-clippy        → cargo clippy -- -D warnings
  3. rust-tests         → cargo test  (64 tests)
  4. frontend-typecheck → npx tsc --noEmit
  5. frontend-tests     → npm run test:run (13 Vitest tests)
```

**Docker images used:**
- `tftsr-linux-amd64:rust1.89-node22` (from internal registry) — Rust steps
- `node:22-alpine` — Frontend steps

---

## Release Pipeline (`.gitea/workflows/auto-tag.yml`)

**Triggers:** Pushes to `master` (auto-tag), then release build/upload jobs run after `autotag`.

Auto tags are created by `.gitea/workflows/auto-tag.yml` using `git tag` + `git push`.
Release jobs are executed in the same workflow and depend on `autotag` completion.

```
Jobs (run in parallel after autotag):
  build-linux-amd64   → image: tftsr-linux-amd64:rust1.89-node22
                         → cargo tauri build (x86_64-unknown-linux-gnu)
                         → {.deb, .rpm, .AppImage} uploaded to Gitea release
                         → fails fast if no Linux artifacts are produced
  build-windows-amd64 → image: tftsr-windows-cross:rust1.89-node22
                         → cargo tauri build (x86_64-pc-windows-gnu) via mingw-w64
                         → {.exe, .msi} uploaded to Gitea release
                         → fails fast if no Windows artifacts are produced
  build-linux-arm64   → image: tftsr-linux-arm64:rust1.89-node22 (ubuntu:22.04-based)
                         → cargo tauri build (aarch64-unknown-linux-gnu)
                         → {.deb, .rpm, .AppImage} uploaded to Gitea release
                         → fails fast if no Linux artifacts are produced
  build-macos-arm64   → timeout-minutes: 120 + cargo/npm + target cache
                         → cargo tauri build (aarch64-apple-darwin) — runs on local Mac
                         → {.dmg} uploaded to Gitea release
                         → existing same-name assets are deleted before upload (rerun-safe)
                         → unsigned; after install run: xattr -cr /Applications/TRCAA.app
```

**Per-step agent routing (Woodpecker 2.x labels):**

```yaml
steps:
  - name: build-linux-amd64
    labels:
      platform: linux/amd64   # → woodpecker_agent on gitea.tftsr.com

  - name: build-linux-arm64
    labels:
      platform: linux/arm64   # → woodpecker-agent.service on local arm64 machine
```

**Multi-agent workspace isolation:**

Steps routed to different agents do **not** share a workspace. The arm64 step clones
the repo directly within its commands (using the internal Gitea URL, accessible from
the local machine) and uploads its artifacts inline. The `upload-release` step (amd64)
handles amd64 + windows artifacts only.

**Clone override (auto-tag.yml — amd64 workspace):**

```yaml
clone:
  git:
    image: alpine/git
    network_mode: gogs_default
    commands:
      - git init -b master
      - git remote add origin http://gitea_app:3000/sarman/tftsr-devops_investigation.git
      - git fetch --depth=1 origin +refs/tags/${CI_COMMIT_TAG}:refs/tags/${CI_COMMIT_TAG}
      - git checkout ${CI_COMMIT_TAG}
```

**Windows cross-compile environment:**
```yaml
environment:
  TARGET: x86_64-pc-windows-gnu
  CC_x86_64_pc_windows_gnu: x86_64-w64-mingw32-gcc
  CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER: x86_64-w64-mingw32-gcc
```

**Artifacts per platform:**
- Linux amd64: `.deb`, `.rpm`, `.AppImage`
- Windows amd64: `.exe` (NSIS installer), `.msi`
- Linux arm64: `.deb`, `.rpm`, `.AppImage`

**Upload step (requires gogs_default network for amd64, host IP for arm64):**
```yaml
# amd64 upload step
upload-release:
  image: curlimages/curl:latest
  labels:
    platform: linux/amd64
  network_mode: gogs_default
  secrets: [GOGS_TOKEN]
```

The `GOGS_TOKEN` Woodpecker secret must be created via the Woodpecker UI or API after
migration. The secret name stays `GOGS_TOKEN` for pipeline compatibility.

**Gitea Release API (replaces Gogs API — same endpoints, different container name):**
```bash
# Create release
POST http://gitea_app:3000/api/v1/repos/sarman/tftsr-devops_investigation/releases
Authorization: token $GOGS_TOKEN

# Upload artifact
POST http://gitea_app:3000/api/v1/repos/sarman/tftsr-devops_investigation/releases/{id}/assets
```

From the arm64 agent (local machine), use the internal Gitea API URL (`${GITEA_URL}/api/v1`) instead.

---

## Multi-File Pipeline Support (Woodpecker 2.x)

Woodpecker 2.x supports multiple pipeline files in the `.woodpecker/` directory.
All `.yml` files are evaluated on every trigger; `when:` conditions control which
pipelines actually run.

Current files:
- `.woodpecker/test.yml` — runs on every push/PR
- `.woodpecker/release.yml` — runs on `v*` tags only

No DB config path switching needed (unlike Woodpecker 0.15.4).

---

## Webhook Configuration

**Woodpecker 2.x with Gitea OAuth2:**

After migration, Woodpecker 2.x registers webhooks automatically when a repo is
activated via the UI. No manual JWT-signed webhook setup required.

1. Log in at `http://gitea.tftsr.com:8085` via Gitea OAuth2
2. Add repo `sarman/tftsr-devops_investigation`
3. Woodpecker creates webhook in Gitea automatically

---

## Branch Protection

Master branch is protected: all changes require a PR.

Branch protection is configured in Gitea under **Repository → Settings → Branches**.
For emergency access, it can be temporarily suspended via the Gitea admin UI and must
be re-enabled immediately after.

---

## Changelog Generation

Changelogs are generated automatically by **git-cliff** on every release.
Configuration lives in `cliff.toml` at the repo root.

### How it works

A `changelog` job in `auto-tag.yml` runs in parallel with the build jobs, immediately
after `autotag` completes:

1. Clones the full repo history with all tags (`--depth=2147483647` — git-cliff needs
   every tag to compute version boundaries).
2. Downloads the git-cliff v2.7.0 static musl binary (~5 MB, no image change needed).
3. Runs `git-cliff --output CHANGELOG.md` to regenerate the full cumulative changelog.
4. Runs `git-cliff --latest --strip all` to produce release notes for the new tag only.
5. PATCHes the Gitea release body with those notes (replaces the static `"Release vX.Y.Z"`).
6. Commits `CHANGELOG.md` to master with `[skip ci]` appended to the message.
   The `[skip ci]` token prevents `auto-tag.yml` from re-triggering on the CHANGELOG commit.
7. Uploads `CHANGELOG.md` as a release asset (replaces any previous version).

### cliff.toml reference

| Setting | Value |
|---------|-------|
| `tag_pattern` | `v[0-9].*` |
| `ignore_tags` | `rc\|alpha\|beta` |
| `filter_unconventional` | `true` — non-conventional commits are dropped |
| Included types | `feat`, `fix`, `perf`, `docs`, `refactor` |
| Excluded types | `ci`, `chore`, `build`, `test`, `style` |

### Loop prevention

The `[skip ci]` suffix on the CHANGELOG commit message is recognised by Gitea Actions
and causes the workflow to be skipped for that push. Without it, the CHANGELOG commit
would trigger `auto-tag.yml` again, incrementing the patch version forever.

### Bootstrap

The initial `CHANGELOG.md` was generated locally before the first PR:
```sh
git-cliff --config cliff.toml --output CHANGELOG.md
```
Subsequent runs are fully automated by CI.

---

## Known Issues & Fixes

### linux-amd64: `libsodium-sys-stable` fails with `make install: No such file or directory`

`libsodium-sys-stable` (transitive dep via `tauri-plugin-stronghold`) attempts to download
and compile libsodium from source when `SODIUM_STATIC=1` is set and `SODIUM_LIB_DIR` is not
provided. `rust:1.89-slim` does not include `make`, causing the compilation to fail with
`os error 2` (executable not found).

**Fix**: `Dockerfile.linux-amd64` now installs `make` and sets:
```dockerfile
ENV SODIUM_LIB_DIR=/usr/lib/x86_64-linux-gnu \
    SODIUM_INCLUDE_DIR=/usr/include
```
`libsodium-dev` (already in the image) provides `/usr/lib/x86_64-linux-gnu/libsodium.a`.
`SODIUM_LIB_DIR` tells the crate to use that library directly, bypassing the
download-and-compile path. The `build-images.yml` linux-amd64 job validates the image
asserts all four conditions before pushing to the registry.

### Debian Multiarch Breaks arm64 Cross-Compile (`held broken packages`)
When using `rust:1.88-slim` (Debian Bookworm) with `dpkg --add-architecture arm64`, apt
resolves amd64 and arm64 simultaneously against the same mirror. The `binary-all` package
index is duplicated and certain `-dev` package pairs cannot be co-installed because they
don't declare `Multi-Arch: same`. This produces `E: Unable to correct problems, you have
held broken packages` and cannot be fixed by tweaking `sources.list` entries.

**Fix**: Use `ubuntu:22.04` as the container image. Ubuntu routes arm64 through
`ports.ubuntu.com/ubuntu-ports` — a separate mirror from `archive.ubuntu.com` (amd64).
There are no cross-arch index overlaps and the dependency resolver succeeds. Rust must be
installed manually via `rustup` since it is not pre-installed in the Ubuntu base image.

### Step Containers Cannot Reach `gitea_app`
Default Docker bridge containers cannot resolve `gitea_app` or reach the internal Gitea
host. Fix: use `network_mode: gogs_default` in any step that needs Gitea access.
Requires `repo_trusted=1`.

### `CI=woodpecker` Rejected by Tauri CLI
Woodpecker sets `CI=woodpecker`; `cargo tauri build` expects a boolean. Fix: prefix with
`CI=true cargo tauri build`.

### Agent Stalls After Server Restart
After restarting the Woodpecker server, the agent may enter a loop cleaning up orphaned
containers and stop picking up new builds. Fix:
```bash
docker rm -f $(docker ps -aq --filter 'name=0_')
docker volume rm $(docker volume ls -q | grep '0_')
docker restart woodpecker_agent
```

### Windows DLL Export Ordinal Too Large
`/usr/bin/x86_64-w64-mingw32-ld: error: export ordinal too large: 106290`

Fix: `src-tauri/.cargo/config.toml`:
```toml
[target.x86_64-pc-windows-gnu]
rustflags = ["-C", "link-arg=-Wl,--exclude-all-symbols"]
```

### GOGS_TOKEN Secret Must Be Recreated After Migration
After migrating from Woodpecker 0.15.4 to 2.x, recreate the `GOGS_TOKEN` secret:
1. Log in to Gitea, create a new API token under Settings → Applications
2. In Woodpecker UI → Repository → Secrets, add secret `GOGS_TOKEN` with the token value

---

## Gitea PostgreSQL Access

Access the Gitea database via the PostgreSQL container:

```bash
docker exec <postgres-container> psql -U <db-user> -d <db-name> -c "SELECT id, lower_name FROM repository;"
```

Container name, database name, and credentials are defined in the Gitea compose file.
Do not commit these values to documentation.

---

## Release Channels

The project ships two update channels:

| Channel | Branch | Tag format | Gitea release flag | Updater endpoint |
|---------|--------|------------|--------------------|-----------------|
| **Stable** | `master` | `v1.2.3` | `prerelease: false` | `/releases?limit=20` → first non-prerelease |
| **Beta** | `beta` | `v1.2.3-beta.N` | `prerelease: true` | `/releases?limit=20` → first prerelease |

### Workflow files

| Workflow | Trigger | Produces |
|----------|---------|---------|
| `auto-tag.yml` | push to `master` | Stable release, wiki sync, CHANGELOG committed back to master |
| `release-beta.yml` | push to `beta` | Pre-release, no wiki sync |

### Beta tag numbering

`release-beta.yml` reads `CARGO_VERSION` from `src-tauri/Cargo.toml` and finds the
highest existing `v{CARGO_VERSION}-beta.N` tag. It increments N on each push:

```
v1.3.0-beta.1  ← first push after Cargo.toml bumped to 1.3.0
v1.3.0-beta.2  ← second push
v1.3.0-beta.3  ← etc.
```

When `Cargo.toml` is bumped (e.g. `1.3.0` → `1.4.0`), the counter resets to `.1`.

### Promoting beta to stable

1. Ensure `Cargo.toml` version is correct (no `-beta` suffix — just the clean semver).
2. Open a PR from `beta → master` and merge it.
3. `auto-tag.yml` fires, creates tag `v1.3.0`, builds all platforms, marks release stable.

### In-app updater: no channel selection

There is no channel picker in Settings → Updater anymore — `AppSettings.update_channel`
and the `get_update_channel`/`set_update_channel` commands have been removed. Installs may
themselves come from a beta prerelease, so filtering `check_app_updates` down to only
`stable` (non-prerelease) releases could hide a genuinely newer prerelease build. Instead,
`check_app_updates` fetches `GET /releases?limit=20` and picks the **highest-versioned
non-draft release**, prereleases included, using a prerelease-aware comparison
(`3.1.0` > `3.1.0-beta.9` > `3.0.0`). Old settings JSON with a leftover `update_channel`
key still deserializes fine — the field is just ignored.

### Version embedding at build time

Every `cargo tauri build` runs `beforeBuildCommand: "npm run version:update && npm run build"`
(`src-tauri/tauri.conf.json`), which invokes `scripts/update-version.mjs`. That script
resolves the version to embed in this order: an explicit CLI argument → `$RELEASE_TAG`
(set as job-level `env:` on every build job in `auto-tag.yml` and `release-beta.yml`) →
`git describe --tags` → `package.json` as a last-resort fallback.

**Bug fixed:** the `autotag` job in `auto-tag.yml` runs in a plain `alpine:latest`
container (BusyBox `sed`, not GNU `sed`). Its version-bump step used to rewrite
`package.json` / `Cargo.toml` / `tauri.conf.json` with the GNU-only
`sed -i "0,/re/s//repl/"` range-addressing form, which BusyBox `sed` silently accepts and
turns into a no-op — no error, no substitution. Only the `Cargo.lock` line (written with a
POSIX-compatible `/pattern/{n;s/.../}` form) actually updated. The result: a release could
tag and build as (say) `v3.1.0` while `get_app_version` — and the three JSON/TOML files —
still reported `3.0.0`. Fixed by having that step call
`node scripts/update-version.mjs "${NEW_VERSION}"` (the same well-tested script used for
local dev bumps and the build-time hook) instead of hand-rolled `sed`, with `nodejs`
added to the job's `apk add` line, followed by a verification step that greps every
version-bearing file for the expected version and fails the job (`exit 1`) if any file
wasn't actually updated.

Separately, `get_app_version` itself used to read the `APP_VERSION`/`CARGO_PKG_VERSION`
**environment variables**, which are never set in a packaged build — it now reads
`app.package_info().version`, sourced from `tauri.conf.json` at build time, matching what
`check_app_updates` already used.

### Branch protection for `beta`

`beta` should carry the same protection rules as `master`:
- Require PR before merging
- Require all CI checks: `rust-fmt-check`, `rust-clippy`, `rust-tests`,
  `frontend-typecheck`, `frontend-tests`
- Dismiss stale reviews on new commits

Set **Settings → Repository → Default Branch** to `beta` so the Gitea UI defaults
new PRs to target `beta` rather than `master`.

---

## Migration Notes (Gogs 0.14 → Gitea)

Gitea auto-migrates the Gogs PostgreSQL schema on first start. Users, repos, teams, and
issues are preserved. API tokens stored in the DB are also migrated but should be
regenerated for security.

Key changes after migration:
- Container name: `gogs_app` → `gitea_app`
- Config dir: `/data/gitea` (was `/data/gogs` inside container, same host volume)
- Repo dir: `gogs-repositories` → `gitea-repositories` (renamed on host during migration)
- OAuth2 provider: Gitea now supports OAuth2 (Woodpecker 2.x uses this for login)
- Woodpecker 2.x multi-file pipeline support enabled (no more single config file limitation)
