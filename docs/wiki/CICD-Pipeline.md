# CI/CD Pipeline

## Infrastructure

| Component | URL | Notes |
|-----------|-----|-------|
| Gogs | `https://gogs.tftsr.com` / `http://172.0.0.29:3000` | Git server, version 0.14 |
| Woodpecker CI (direct) | `http://172.0.0.29:8084` | v0.15.4 |
| Woodpecker CI (proxy) | `http://172.0.0.29:8085` | nginx with custom login page |
| PostgreSQL (Gogs DB) | Container: `gogs_postgres_db` | DB: `gogsdb`, User: `gogs` |

### CI Agents

| Container | Platform | Purpose |
|-----------|----------|---------|
| `woodpecker_agent` | `linux/amd64` | Native x86_64 — all test builds + amd64/windows release |
| `woodpecker_agent_arm64` | `linux/arm64` | QEMU emulation on x86_64 host — arm64 release builds |

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

**Pipeline YAML format (Woodpecker 0.15.4 — legacy MAP format):**
```yaml
clone:
  git:
    image: woodpeckerci/plugin-git
    network_mode: gogs_default     # requires repo_trusted=1
    environment:
      - CI_REPO_CLONE_URL=http://gogs_app:3000/sarman/tftsr-devops_investigation.git

pipeline:
  step-name:                        # KEY = step name (MAP, not list!)
    image: rust:1.88-slim
    commands:
      - cargo test
```

> ⚠️ **Do NOT** use the newer `steps:` list format — Woodpecker 0.15.4 uses the Drone-legacy map format. Using `steps:` causes "Invalid or missing pipeline section" error.

---

## Release Pipeline (`.woodpecker/release.yml`)

**Triggers:** Git tags matching `v*`

**Active config path:** Woodpecker DB must have `repo_config_path = .woodpecker/release.yml` when the tag is pushed. Switch back to `test.yml` after tagging to restore PR/push CI.

```
Pipeline steps:
  1. clone                → alpine/git with explicit tag fetch + checkout
  2. build-linux-amd64   → cargo tauri build (x86_64-unknown-linux-gnu)
                            → artifacts/linux-amd64/{.deb, .rpm, .AppImage}
  3. build-windows-amd64 → cargo tauri build (x86_64-pc-windows-gnu via mingw-w64)
                            → artifacts/windows-amd64/{.exe, .msi}
  4. upload-release       → Create Gogs release + upload all artifacts
```

**Clone override (release.yml):**

Release builds use `alpine/git` with explicit commands because `woodpeckerci/plugin-git:latest` uses `git switch` which fails on tag refs:

```yaml
clone:
  git:
    image: alpine/git
    network_mode: gogs_default
    commands:
      - git init -b master
      - git remote add origin http://gogs_app:3000/sarman/tftsr-devops_investigation.git
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
- Linux arm64: `.deb`, `.AppImage` (requires arm64 agent or QEMU)

**Important:** Artifacts must be written to the **workspace** (relative paths like `artifacts/linux-amd64/`), not to absolute paths like `/artifacts/`. Only the workspace is shared between pipeline steps via Docker volume.

**Upload step (requires gogs_default network):**
```yaml
upload-release:
  image: curlimages/curl:latest
  network_mode: gogs_default    # host firewall blocks default bridge from reaching Gogs API
  secrets: [GOGS_TOKEN]
```

The `GOGS_TOKEN` Woodpecker secret is inserted into the DB:
```python
conn.execute("""
  INSERT INTO secrets (secret_repo_id, secret_name, secret_value, secret_images, secret_events, secret_skip_verify, secret_conceal)
  VALUES (1, 'GOGS_TOKEN', '<bearer_token>', '', 'tag', 0, 1)
""")
```

**Gogs Release API:**
```bash
# Create release
POST http://gogs_app:3000/api/v1/repos/sarman/tftsr-devops_investigation/releases
Authorization: token $GOGS_TOKEN

# Upload artifact
POST http://gogs_app:3000/api/v1/repos/sarman/tftsr-devops_investigation/releases/{id}/assets
```

---

## Switching Between Test and Release Config

Woodpecker 0.15.4 supports only **one config file per repo**. The workflow:

```bash
# For regular pushes/PRs — use test pipeline
python3 -c "conn.execute(\"UPDATE repos SET repo_config_path='.woodpecker/test.yml'\")"

# Before pushing a release tag — switch to release pipeline
python3 -c "conn.execute(\"UPDATE repos SET repo_config_path='.woodpecker/release.yml'\")"
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
# → Switch back to test.yml after build starts
```

---

## Webhook Configuration

**Hook ID:** 9 (in Gogs, `http://gogs.tftsr.com`)
**Events:** `create`, `push`, `pull_request`
**URL:** `http://172.0.0.29:8084/hook?access_token=<JWT>`

**JWT signing:**
- Algorithm: HS256
- Secret: `repo_hash` from Woodpecker DB (`dK8zFWtAu67qfKd3Et6N8LptqTmedumJ`)
- Payload: `{"text":"sarman/tftsr-devops_investigation","type":"hook","iat":<timestamp>}`

**Regenerate JWT when stale:**
```python
import base64, hmac, hashlib, json, time

def b64url(data):
    if isinstance(data, str): data = data.encode()
    return base64.urlsafe_b64encode(data).rstrip(b'=').decode()

header  = b64url(json.dumps({'alg':'HS256','typ':'JWT'}, separators=(',',':')))
payload = b64url(json.dumps({'text':'sarman/tftsr-devops_investigation','type':'hook','iat':int(time.time())}, separators=(',',':')))
msg     = f'{header}.{payload}'
sig     = hmac.new(b'dK8zFWtAu67qfKd3Et6N8LptqTmedumJ', msg.encode(), hashlib.sha256).digest()
print(f'{msg}.{b64url(sig)}')
```

Then update the webhook in Gogs via API:
```bash
curl -X DELETE http://172.0.0.29:3000/api/v1/repos/sarman/tftsr-devops_investigation/hooks/<old_id>
curl -X POST http://172.0.0.29:3000/api/v1/repos/sarman/tftsr-devops_investigation/hooks \
  -H "Authorization: token <bearer_token>" \
  -H "Content-Type: application/json" \
  -d '{"type":"gogs","config":{"url":"http://172.0.0.29:8084/hook?access_token=<NEW_JWT>","content_type":"json","secret":"af5dc60e0984f2680d0969f4a087e7100a4ece7e"},"events":["push","pull_request","create"],"active":true}'
```

---

## Woodpecker DB State

SQLite at `/docker_mounts/woodpecker/data/woodpecker.sqlite` (on host `172.0.0.29`).

```sql
-- Verify config
SELECT user_token IS NOT NULL AND user_token != '' AS token_set FROM users WHERE user_login='sarman';

SELECT repo_active, repo_trusted, repo_config_path, repo_hash
FROM repos WHERE repo_full_name='sarman/tftsr-devops_investigation';
-- repo_active=1, repo_trusted=1
-- repo_config_path='.woodpecker/test.yml'  (or release.yml during release)
-- repo_hash='dK8zFWtAu67qfKd3Et6N8LptqTmedumJ'
```

---

## Branch Protection

Master branch is protected: all changes require a PR. Direct pushes are blocked.

```sql
-- Check protection
SELECT name, protected, require_pull_request FROM protect_branch WHERE repo_id=42;

-- Temporarily disable for urgent fixes (restore immediately after!)
UPDATE protect_branch SET protected=false WHERE repo_id=42 AND name='master';
-- ... push ...
UPDATE protect_branch SET protected=true, require_pull_request=true WHERE repo_id=42 AND name='master';
```

> Gogs 0.14 does **not** enforce required CI status checks before merging. Only `require_pull_request=true` is supported.

---

## Known Issues & Fixes

### Webhook JWT Must Use `?access_token=`
`token.ParseRequest()` in Woodpecker 0.15.4 does **not** read `?token=` URL params. Use `?access_token=<JWT>` instead.

### Directory-Based Config Not Supported
Woodpecker 0.15.4 only supports a **single config file**. Multi-file pipelines require v2.x+.

### Empty Clone URL in Push Events
Woodpecker 0.15.4's `go-gogs-client` `PayloadRepo` struct lacks `CloneURL`, so `build_remote` is always empty. Fix: set `CI_REPO_CLONE_URL` in the clone step environment.

### Step Containers Cannot Reach `gogs_app`
Default Docker bridge containers cannot resolve `gogs_app` or reach `172.0.0.29:3000` (host firewall). Fix: use `network_mode: gogs_default` in any step that needs Gogs access. Requires `repo_trusted=1`.

### `CI=woodpecker` Rejected by Tauri CLI
Woodpecker sets `CI=woodpecker`; `cargo tauri build` expects a boolean. Fix: prefix with `CI=true cargo tauri build`.

### Agent Stalls After Server Restart
After restarting the Woodpecker server, the agent may enter a loop cleaning up orphaned containers and stop picking up new builds. Fix:
```bash
# Kill orphan containers and volumes
docker rm -f $(docker ps -aq --filter 'name=0_')
docker volume rm $(docker volume ls -q | grep '0_')
# Restart agent
docker restart woodpecker_agent
```

### Windows DLL Export Ordinal Too Large
`/usr/bin/x86_64-w64-mingw32-ld: error: export ordinal too large: 106290`

MinGW's `ld` auto-exports ALL public Rust symbols into the DLL export table. With a
large dependency tree (~106k symbols), this exceeds the 65,535 PE ordinal limit.

Fix: `src-tauri/.cargo/config.toml` tells `ld` to suppress auto-export:
```toml
[target.x86_64-pc-windows-gnu]
rustflags = ["-C", "link-arg=-Wl,--exclude-all-symbols"]
```
The desktop `main.exe` links against `rlib` (static), so the cdylib export table is
unused at runtime. An empty export table is valid for a DLL.

### Gogs OAuth2 Limitation
Gogs 0.14 has no OAuth2 provider support, blocking upgrade to Woodpecker 2.x.

---

## Gogs PostgreSQL Access

```bash
docker exec gogs_postgres_db psql -U gogs -d gogsdb -c "SELECT id, lower_name FROM repository;"
```

> Database name is `gogsdb`, not `gogs`.
