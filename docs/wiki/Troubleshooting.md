# Troubleshooting

## CI/CD — Gitea Actions

### Build Not Triggering After Push

**Check:**
1. Verify the workflow file exists in `.gitea/workflows/` on the pushed branch
2. Check the Actions tab at `http://172.0.0.29:3000/sarman/tftsr-devops_investigation/actions`
3. Confirm the act_runner is online: `docker logs gitea_act_runner_amd64 --since 5m`

---

### Job Container Can't Reach Gitea (`172.0.0.29:3000` blocked)

**Cause:** act_runner creates an isolated Docker network per job (when `container:` is specified). Traffic from the job container to `172.0.0.29:3000` is blocked by the host firewall.

**Fix:** Ensure `container.network: host` is set in the act_runner config AND that `CONFIG_FILE=/data/config.yaml` is in the container's environment:

```yaml
# /docker_mounts/gitea/runner/amd64/config.yaml
container:
  network: "host"
```

```yaml
# docker-compose.yml for act-runner-amd64
environment:
  - CONFIG_FILE=/data/config.yaml
```

Also set `capacity: 1` — with capacity > 1, concurrent jobs may not get host networking:

```yaml
runner:
  capacity: 1
```

Restart runner: `docker restart gitea_act_runner_amd64`

---

### `Unable to locate package git` in Rust Job

**Cause:** `rust:1.88-slim` has an empty apt package cache.

**Fix:** Always run `apt-get update` before `apt-get install`:
```yaml
- name: Checkout
  run: |
    apt-get update -qq && apt-get install -y -qq git
    git init
    git remote add origin http://172.0.0.29:3000/sarman/tftsr-devops_investigation.git
    git fetch --depth=1 origin $GITHUB_SHA
    git checkout FETCH_HEAD
```

---

### `exec: "node": executable file not found in $PATH`

**Cause:** `actions/checkout@v4` is a Node.js action. `rust:1.88-slim` and similar slim images don't have Node.

**Fix:** Don't use `actions/checkout@v4` — use direct git commands instead (see above).

---

### Job Skipped (status 6) on Tag Push

**Cause:** Pattern matching issue with `on: push: tags:`. Use unquoted glob in the workflow:

```yaml
# Correct
on:
  push:
    tags:
      - v*
```

Also add `workflow_dispatch` for manual triggering during testing:
```yaml
on:
  push:
    tags:
      - v*
  workflow_dispatch:
    inputs:
      tag:
        description: 'Release tag'
        required: true
```

---

### `CI=woodpecker` Rejected by `cargo tauri build`

**Cause:** CI runners set `CI=woodpecker` (string). Tauri CLI expects `true`/`false`.

**Fix:** Prefix the build command:
```yaml
- run: CI=true cargo tauri build --target $TARGET
```

---

### Release Artifacts Not Uploaded

**Cause 1:** `RELEASE_TOKEN` secret not set or expired.
```bash
# Recreate via admin CLI:
docker exec -u git gitea_app gitea admin user generate-access-token \
  --username sarman --token-name gitea-ci-token --raw \
  --scopes 'write:repository,read:user'
# Add the token as RELEASE_TOKEN in repo Settings > Actions > Secrets
```

**Cause 2:** Each build job uploads its own artifacts independently. All jobs require host network on the runner (see above).

---

## Rust Compilation

### `MutexGuard` Not `Send` Across Await

```
error[E0277]: `MutexGuard<'_, Connection>` cannot be sent between threads safely
```

**Fix:** Release the mutex lock before any `.await` point:
```rust
let result = {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.query_row(...)?
};  // lock dropped here
async_fn().await?;
```

---

### Clippy Lints Fail in CI

Common lint fixes required by `-D warnings` (Rust 1.88+):

```rust
format!("{}", x)  →  format!("{x}")
x >= a && x < b  →  (a..b).contains(&x)
s.push_str("a")  →  s.push('a')
```

Run locally: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`

Auto-fix: `cargo clippy --manifest-path src-tauri/Cargo.toml --fix --allow-dirty -- -D warnings`

---

### `cargo tauri dev` Fails — Missing System Libraries

**Fix (Fedora/RHEL):**
```bash
sudo dnf install -y glib2-devel gtk3-devel webkit2gtk4.1-devel \
  libsoup3-devel openssl-devel librsvg2-devel
```

**Fix (Debian/Ubuntu):**
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf pkg-config
```

---

## Database

### DB Won't Open in Production

**Symptom:** App fails to start with SQLCipher error.

1. `TRCAA_DB_KEY` (or legacy `TRCAA_DB_KEY`) env var is set
2. Key matches what was used when DB was created
3. File isn't corrupted: `file tftsr.db` should say `SQLite 3.x database`

---

### Migration Fails to Run

Check which migrations have been applied:
```sql
SELECT name, applied_at FROM _migrations ORDER BY id;
```

---

## Frontend

### TypeScript Errors After Pulling

```bash
npx tsc --noEmit
```

Ensure `tauriCommands.ts` matches Rust command signatures exactly (especially `IssueDetail` nesting).

---

### `IssueDetail` Field Access Errors

`get_issue()` returns a **nested** struct:
```typescript
// Correct
const title = detail.issue.title;

// Wrong — field doesn't exist at top level
const title = detail.title;
```

---

### Vitest Tests Fail

Common causes:
- Mocked `invoke()` return type doesn't match updated command signature
- `sessionStore` state not reset between tests (call `store.reset()` in `beforeEach`)

---

## Gitea

### API Token Authentication

```bash
curl -H "Authorization: token <token_value>" http://172.0.0.29:3000/api/v1/user
```

Create tokens in Gitea Settings > Applications > Access Tokens, or via admin CLI:
```bash
docker exec -u git gitea_app gitea admin user generate-access-token \
  --username sarman --token-name mytoken --raw --scopes 'read:user,write:repository'
```

### PostgreSQL Access

```bash
docker exec gogs_postgres_db psql -U gogs -d gogsdb -c "SELECT id, lower_name, is_private FROM repository;"
```

Database is named `gogsdb`. The PostgreSQL instance uses SCRAM-SHA-256 auth (MD5 also configured for the `gogs` user for compatibility with older clients).
