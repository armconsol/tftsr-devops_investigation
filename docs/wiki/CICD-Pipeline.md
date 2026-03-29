# CI/CD Pipeline

## Infrastructure

| Component | URL | Notes |
|-----------|-----|-------|
| Gitea | `https://gogs.tftsr.com` / `http://172.0.0.29:3000` | Git server (migrated from Gogs 0.14) |
| Woodpecker CI (direct) | `http://172.0.0.29:8084` | v2.x |
| Woodpecker CI (proxy) | `http://172.0.0.29:8085` | nginx reverse proxy |
| PostgreSQL (Gitea DB) | Container: `gogs_postgres_db` | DB: `gogsdb`, User: `gogs` |

### CI Agents

| Agent | Platform | Host | Purpose |
|-------|----------|------|---------|
| `woodpecker_agent` (Docker) | `linux/amd64` | 172.0.0.29 | Native x86_64 — test builds + amd64/windows release |
| `woodpecker-agent` (systemd) | `linux/arm64` | sarman's local machine | Native aarch64 — arm64 release builds |
| `woodpecker_agent_arm64` (Docker) | `linux/arm64` | 172.0.0.29 | QEMU fallback — kept as backup |

Agent labels configured via `WOODPECKER_LABELS`:
- Docker agents: `WOODPECKER_LABELS=platform=linux/amd64` (or arm64)
- Local systemd agent: `~/.config/woodpecker-agent/config.env` → `WOODPECKER_LABELS=platform=linux/arm64`

---

## Test Pipeline (`.woodpecker/test.yml`)

**Triggers:** Every push and pull request to any branch.

```
Pipeline steps:
  1. rust-fmt-check     → cargo fmt --check
  2. rust-clippy        → cargo clippy -- -D warnings
  3. rust-tests         → cargo test  (64 tests)
  4. frontend-typecheck → npx tsc --noEmit
  5. frontend-tests     → npm run test:run (13 Vitest tests)
```

**Docker images used:**
- `rust:1.88-slim` — Rust steps (minimum for cookie_store + time + darling)
- `node:22-alpine` — Frontend steps

**Pipeline YAML format (Woodpecker 2.x — steps list format):**
```yaml
clone:
  git:
    image: woodpeckerci/plugin-git
    network_mode: gogs_default     # requires repo_trusted=1
    environment:
      - CI_REPO_CLONE_URL=http://gitea_app:3000/sarman/tftsr-devops_investigation.git

steps:
  - name: step-name                 # LIST format (- name:)
    image: rust:1.88-slim
    commands:
      - cargo test
```

> ⚠️ Woodpecker 2.x uses the `steps:` list format. The legacy `pipeline:` map format from
> Woodpecker 0.15.4 is no longer supported.

---

## Release Pipeline (`.woodpecker/release.yml`)

**Triggers:** Git tags matching `v*`

```
Pipeline steps:
  1. clone (amd64 workspace)  → alpine/git with explicit tag fetch + checkout
  2. build-linux-amd64        → cargo tauri build (x86_64-unknown-linux-gnu)
                                 → artifacts/linux-amd64/{.deb, .rpm, .AppImage}
  3. build-windows-amd64      → cargo tauri build (x86_64-pc-windows-gnu)
                                 → artifacts/windows-amd64/{.exe, .msi}
  4. build-linux-arm64        → cargo tauri build (aarch64-unknown-linux-gnu)
                                 → artifacts/linux-arm64/{.deb, .rpm, .AppImage}
                                 → uploads arm64 artifacts inline to Gitea release
  5. upload-release            → Create Gitea release + upload amd64 + windows artifacts
```

**Per-step agent routing (Woodpecker 2.x labels):**

```yaml
steps:
  - name: build-linux-amd64
    labels:
      platform: linux/amd64   # → woodpecker_agent on 172.0.0.29

  - name: build-linux-arm64
    labels:
      platform: linux/arm64   # → woodpecker-agent.service on local arm64 machine
```

**Multi-agent workspace isolation:**

Steps routed to different agents do **not** share a workspace. The arm64 step clones
the repo directly within its commands (using `http://172.0.0.29:3000`, accessible from
the local machine) and uploads its artifacts inline. The `upload-release` step (amd64)
handles amd64 + windows artifacts only.

**Clone override (release.yml — amd64 workspace):**

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

From the arm64 agent (local machine), use `http://172.0.0.29:3000/api/v1` instead.

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

1. Log in at `http://172.0.0.29:8085` via Gitea OAuth2
2. Add repo `sarman/tftsr-devops_investigation`
3. Woodpecker creates webhook in Gitea automatically

---

## Branch Protection

Master branch is protected: all changes require a PR.

```sql
-- Gitea branch protection (via psql on gogs_postgres_db container)
-- Check protection
SELECT name, protected, require_pull_request FROM protect_branch WHERE repo_id=42;

-- Temporarily disable for urgent fixes (restore immediately after!)
UPDATE protect_branch SET protected=false WHERE repo_id=42 AND name='master';
-- ... push ...
UPDATE protect_branch SET protected=true, require_pull_request=true WHERE repo_id=42 AND name='master';
```

---

## Known Issues & Fixes

### Step Containers Cannot Reach `gitea_app`
Default Docker bridge containers cannot resolve `gitea_app` or reach `172.0.0.29:3000`
(host firewall). Fix: use `network_mode: gogs_default` in any step that needs Gitea
access. Requires `repo_trusted=1`.

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

```bash
docker exec gogs_postgres_db psql -U gogs -d gogsdb -c "SELECT id, lower_name FROM repository;"
```

> Database name is `gogsdb` (unchanged from Gogs migration).

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
