# Troubleshooting

## CI/CD

### Builds Not Triggering After Push

**Cause:** Woodpecker 0.15.4 `token.ParseRequest()` does not read `?token=` URL params.

**Fix:** Webhook URL must use `?access_token=<JWT>` (not `?token=`).

Regenerate the JWT if it's stale (see [CICD-Pipeline → Webhook Configuration](wiki/CICD-Pipeline)):
```python
# JWT payload: {"text":"sarman/tftsr-devops_investigation","type":"hook"}
# Signed HS256 with repo_hash = dK8zFWtAu67qfKd3Et6N8LptqTmedumJ
```

---

### Build Stuck in "Pending" or "Running" with No Containers

**Cause:** After a Woodpecker server restart, the agent enters a loop trying to clean up orphaned containers from the previous session and stops processing new tasks.

**Fix:**
```bash
# 1. Kill any orphan step containers and volumes
ssh sarman@172.0.0.29
docker rm -f $(docker ps -aq --filter 'name=0_')
docker volume rm $(docker volume ls -q | grep '0_')

# 2. Cancel stuck builds in Woodpecker DB
python3 -c "
import sqlite3; conn = sqlite3.connect('/docker_mounts/woodpecker/data/woodpecker.sqlite')
conn.execute(\"UPDATE builds SET build_status='cancelled' WHERE build_status IN ('pending','running')\")
conn.execute('DELETE FROM tasks')
conn.commit()
"

# 3. Restart agent
docker restart woodpecker_agent
```

---

### Pipeline Step Can't Reach Gogs (`gogs_app` not found)

**Cause:** Step containers run on the default Docker bridge. They cannot resolve `gogs_app` hostname or reach `172.0.0.29:3000` (blocked by host firewall).

**Fix:** Add `network_mode: gogs_default` to any step needing Gogs access. Requires `repo_trusted=1`:
```bash
python3 -c "
import sqlite3; conn = sqlite3.connect('/docker_mounts/woodpecker/data/woodpecker.sqlite')
conn.execute(\"UPDATE repos SET repo_trusted=1 WHERE repo_full_name='sarman/tftsr-devops_investigation'\")
conn.commit()
"
```

---

### `CI=woodpecker` Rejected by `cargo tauri build`

**Cause:** Woodpecker sets `CI=woodpecker` (string). Tauri CLI's `--ci` flag expects `true`/`false`.

**Fix:** Prefix the build command:
```yaml
commands:
  - CI=true cargo tauri build --target $TARGET
```

---

### Release Artifacts Not Uploaded (Upload Step Silent Failure)

**Cause 1:** Upload container on default bridge can't reach Gogs API at `http://172.0.0.29:3000`.
**Fix:** Add `network_mode: gogs_default` to the upload step and use `gogs_app:3000` in the API URL.

**Cause 2:** Artifacts written to `/artifacts/` (absolute path) not visible to the upload step.
**Fix:** Write artifacts to the workspace using relative paths (`artifacts/linux-amd64/`). Only the workspace directory is shared between pipeline steps.

**Cause 3:** `GOGS_TOKEN` secret not set or has wrong value.
**Check:**
```bash
python3 -c "
import sqlite3; conn = sqlite3.connect('/docker_mounts/woodpecker/data/woodpecker.sqlite')
s = conn.execute(\"SELECT secret_value FROM secrets WHERE secret_name='GOGS_TOKEN'\").fetchone()
print('Token length:', len(s[0]) if s else 'NOT SET')
"
# Test token (replace 172.0.0.29:3000 with gogs_app:3000 from inside gogs_default network)
curl -H "Authorization: token <token_value>" http://172.0.0.29:3000/api/v1/user
```

---

### `git switch refs/tags/v*` Fails in Release Build

**Cause:** `woodpeckerci/plugin-git:latest` uses `git switch` which doesn't support detached HEAD for tag refs.

**Fix:** Override the clone section with `alpine/git` and explicit commands:
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

---

### Woodpecker Login Fails

**Cause:** Gogs 0.14 SPA login form uses `login=` field; backend reads `username=`.

**Fix:** Use the nginx proxy at `http://172.0.0.29:8085/login` which serves a corrected login form.

---

### Empty Clone URL in Pipeline (Push Events)

**Cause:** Woodpecker 0.15.4 `go-gogs-client` `PayloadRepo` struct is missing `CloneURL`.

**Fix:** Override with `CI_REPO_CLONE_URL` environment variable in the clone section:
```yaml
clone:
  git:
    environment:
      - CI_REPO_CLONE_URL=http://gogs_app:3000/sarman/tftsr-devops_investigation.git
```

---

## Rust Compilation

### `MutexGuard` Not `Send` Across Await

```
error[E0277]: `MutexGuard<'_, Connection>` cannot be sent between threads safely
```

**Fix:** Release the mutex lock before any `.await` point:
```rust
// ✅ Correct
let result = {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.query_row(...)?
};  // lock dropped here
async_fn().await?;
```

---

### Clippy Lints Fail in CI

Common lint fixes required by `-D warnings` (especially on Rust 1.88+):

```rust
// uninlined_format_args
format!("{}", x)  →  format!("{x}")

// range::contains
x >= a && x < b  →  (a..b).contains(&x)

// push_str single char
s.push_str("a")  →  s.push('a')
```

Run locally: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings -W clippy::uninlined_format_args`

Auto-fix: `cargo clippy --manifest-path src-tauri/Cargo.toml --fix --allow-dirty -- -D warnings -W clippy::uninlined_format_args`

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

**Check:**
1. `TFTSR_DB_KEY` env var is set
2. Key matches what was used when DB was created
3. File isn't corrupted: `file tftsr.db` should say `SQLite 3.x database`

**Warning:** Changing the key requires re-encrypting the database:
```bash
sqlite3 tftsr.db "ATTACH 'new.db' AS newdb KEY 'new-key'; \
  SELECT sqlcipher_export('newdb'); DETACH DATABASE newdb;"
```

---

### Migration Fails to Run

Check which migrations have been applied:
```sql
SELECT name, applied_at FROM _migrations ORDER BY id;
```

If a migration is partially applied, the DB may be inconsistent. Restore from backup or recreate.

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
// ✅ Correct
const title = detail.issue.title;

// ❌ Wrong
const title = detail.title;  // field doesn't exist at top level
```

---

### Vitest Tests Fail

```bash
npm run test:run
```

Common causes:
- Mocked `invoke()` return type doesn't match updated command signature
- `sessionStore` state not reset between tests (call `store.reset()` in `beforeEach`)

---

## Gogs

### Token Authentication

The `sha1` field from the Gogs token create API **is** the bearer token — use it directly:
```bash
curl -H "Authorization: token <sha1_value>" https://gogs.tftsr.com/api/v1/user
```

Do not confuse with the `sha1` column in the `access_token` table, which stores `sha1(token)[:40]` (a hash of the token, not the token itself).

### PostgreSQL Access

```bash
docker exec gogs_postgres_db psql -U gogs -d gogsdb -c "SELECT id, lower_name, is_private FROM repository;"
```

Database is named `gogsdb`, not `gogs`.
