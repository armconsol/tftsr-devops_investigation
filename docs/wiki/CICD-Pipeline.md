# CI/CD Pipeline

## Infrastructure

| Component | URL | Notes |
|-----------|-----|-------|
| Gogs | `http://172.0.0.29:3000` / `https://gogs.tftsr.com` | Git server, version 0.14 |
| Woodpecker CI (direct) | `http://172.0.0.29:8084` | v0.15.4 |
| Woodpecker CI (proxy) | `http://172.0.0.29:8085` | nginx with custom login page |
| PostgreSQL (Gogs DB) | Container: `gogs_postgres_db` | DB: `gogsdb`, User: `gogs` |

---

## Test Pipeline (`.woodpecker/test.yml`)

**Triggers:** Every push and pull request to any branch.

```
Pipeline steps:
  1. rust-fmt-check     → cargo fmt --check
  2. rust-clippy        → cargo clippy -- -D warnings
  3. rust-tests         → cargo test
  4. frontend-typecheck → npx tsc --noEmit
  5. frontend-tests     → npm run test:run (Vitest)
```

**Docker images used:**
- `rust:1.88-slim` — Rust steps (minimum for cookie_store + time + darling)
- `node:22-alpine` — Frontend steps

**System dependencies installed in CI (Rust steps):**
```
libwebkit2gtk-4.1-dev, libssl-dev, libgtk-3-dev, libsoup-3.0-dev,
librsvg2-dev, libglib2.0-dev
```

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

> ⚠️ **Do NOT** use the newer `steps:` list format — Woodpecker 0.15.4 uses the Drone-legacy map format.

---

## Release Pipeline (`.woodpecker/release.yml`)

**Triggers:** Git tags matching `v*`

```
Pipeline steps:
  1. build-linux-amd64   → cargo tauri build (x86_64-unknown-linux-gnu)
  2. build-linux-arm64   → cargo tauri build (aarch64-unknown-linux-gnu, cross-compile)
  3. upload-release      → Create Gogs release + upload artifacts via API
```

**Artifacts per platform:**
- Linux amd64: `.deb`, `.rpm`, `.AppImage`
- Linux arm64: `.deb`, `.AppImage`

**Gogs Release API:**
```bash
# Create release
POST $API/repos/sarman/tftsr-devops_investigation/releases
Authorization: token $GOGS_TOKEN

# Upload artifact
POST $API/repos/sarman/tftsr-devops_investigation/releases/{id}/assets
```

The `GOGS_TOKEN` is stored as a Woodpecker secret.

---

## Webhook Configuration

**Hook ID:** 6 (in Gogs)
**Events:** `create`, `push`, `pull_request`
**URL:** `http://172.0.0.29:8084/hook?access_token=<JWT>`

**JWT signing:**
- Algorithm: HS256
- Secret: `repo_hash` value from Woodpecker DB (`dK8zFWtAu67qfKd3Et6N8LptqTmedumJ`)
- Payload: `{"text":"sarman/tftsr-devops_investigation","type":"hook"}`

> ⚠️ JWT has an `iat` claim. If it's stale, regenerate it.

---

## Woodpecker DB State

SQLite at `/docker_mounts/woodpecker/data/woodpecker.sqlite` (on host `172.0.0.29`).

Key values:
```sql
-- User
SELECT user_token FROM users WHERE user_login='sarman';
-- Should be: [REDACTED-ROTATED]

-- Repo
SELECT repo_active, repo_trusted, repo_config_path, repo_hash
FROM repos WHERE repo_full_name='sarman/tftsr-devops_investigation';
-- repo_active=1, repo_trusted=1
-- repo_config_path='.woodpecker/test.yml'
-- repo_hash='dK8zFWtAu67qfKd3Et6N8LptqTmedumJ'
```

---

## Known Issues & Fixes

### Webhook JWT Must Use `?access_token=`
`token.ParseRequest()` in Woodpecker 0.15.4 does **not** read `?token=` URL params. Use `?access_token=<JWT>` instead.

### JWT Signed with `repo_hash` (Not User Hash)
Hook JWT must be signed with the `repo_hash` value, not the user's hash.

### Directory-Based Config Not Supported
Woodpecker 0.15.4 only supports a **single config file**. Set `repo_config_path = .woodpecker/test.yml` in the Woodpecker DB. The `.woodpecker/` directory approach requires v2.x+.

### Step Containers Network Isolation
Pipeline step containers run on the default Docker bridge and cannot resolve `gogs_app` hostname. Fix: set `network_mode: gogs_default` in the clone section (requires `repo_trusted=1`).

### Empty Clone URL Bug
Woodpecker 0.15.4's `go-gogs-client` `PayloadRepo` struct lacks `CloneURL`/`SSHURL` fields, so `build_remote` is always empty from Gogs push payloads. Fix: override the clone URL via `CI_REPO_CLONE_URL` environment variable.

### Gogs Token Authentication
The `sha1` field in Gogs token create API response **is** the actual bearer token (not a hash). Use it directly:
```
Authorization: token <sha1_from_create_response>
```

### Gogs SPA Login Field Mismatch
Gogs 0.14 SPA login form uses `login=` field; the Gogs backend reads `username=`. A custom login page is served by nginx at `/login`.

### Gogs OAuth2 Limitation
Gogs 0.14 has no OAuth2 provider support, blocking upgrade to Woodpecker 2.x.

---

## Gogs PostgreSQL Access

```bash
docker exec gogs_postgres_db psql -U gogs -d gogsdb -c "SELECT id, lower_name FROM repository;"
```

> Database name is `gogsdb`, not `gogs`.
