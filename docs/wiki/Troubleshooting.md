# Troubleshooting

## CI/CD

### Builds Not Triggering After Push

**Cause:** Woodpecker 0.15.4 `token.ParseRequest()` does not read `?token=` URL params.

**Fix:** Webhook URL must use `?access_token=<JWT>` (not `?token=`).
```
http://172.0.0.29:8084/hook?access_token=<JWT>
```

Regenerate the JWT if it's stale (JWT has an `iat` claim):
```bash
# JWT payload: {"text":"sarman/tftsr-devops_investigation","type":"hook"}
# Signed with: repo_hash (dK8zFWtAu67qfKd3Et6N8LptqTmedumJ)
```

---

### Pipeline Step Can't Reach Gogs

**Cause:** Step containers run on the default Docker bridge, not on `gogs_default` network.

**Fix:** Use `network_mode: gogs_default` in the clone section and ensure `repo_trusted=1`:
```bash
docker exec woodpecker_db sqlite3 /data/woodpecker.sqlite \
  "UPDATE repos SET repo_trusted=1 WHERE repo_full_name='sarman/tftsr-devops_investigation';"
```

---

### Woodpecker Login Fails

**Cause:** Gogs 0.14 SPA login form uses `login=` field; backend reads `username=`.

**Fix:** Use the nginx proxy at `http://172.0.0.29:8085/login` which serves a corrected login form.

---

### Empty Clone URL in Pipeline

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

**Error:**
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

Common lint fixes:

```rust
// uninlined_format_args
format!("{}", x)  →  format!("{x}")

// range::contains
x >= a && x < b  →  (a..b).contains(&x)

// push_str single char
s.push_str("a")  →  s.push('a')
```

Run locally: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`

---

### `cargo tauri dev` Fails — Missing System Libraries

**Fix (Fedora/RHEL):**
```bash
sudo dnf install -y glib2-devel gtk3-devel webkit2gtk4.1-devel \
  libsoup3-devel openssl-devel librsvg2-devel
```

---

## Database

### DB Won't Open in Production

**Symptom:** App fails to start with SQLCipher error.

**Check:**
1. `TFTSR_DB_KEY` env var is set
2. Key matches what was used when DB was created
3. File isn't corrupted (try `file tftsr.db` — should say `SQLite 3.x database`)

**Warning:** Changing the key requires re-encrypting the database:
```bash
sqlite3 tftsr.db "ATTACH 'new.db' AS newdb KEY 'new-key'; \
  SELECT sqlcipher_export('newdb'); DETACH DATABASE newdb;"
```

---

### Migration Fails to Run

Check which migrations have already been applied:
```sql
SELECT name, applied_at FROM _migrations ORDER BY id;
```

If a migration is partially applied, the DB may be in an inconsistent state. Restore from backup or recreate.

---

## Frontend

### TypeScript Errors After Pulling

Run a fresh type check:
```bash
npx tsc --noEmit
```

Ensure `tauriCommands.ts` matches the Rust command signatures exactly (especially `IssueDetail` nesting).

---

### `IssueDetail` Field Access Errors

The `get_issue` command returns a **nested** struct:
```typescript
// ✅ Correct
const title = detail.issue.title;
const severity = detail.issue.severity;

// ❌ Wrong — these fields don't exist at the top level
const title = detail.title;
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

Do not confuse with the `sha1` column in the `access_token` table, which stores `sha1(token)[:40]`.

### PostgreSQL Access

```bash
docker exec gogs_postgres_db psql -U gogs -d gogsdb -c "SELECT id, lower_name, is_private FROM repository;"
```

Database is named `gogsdb`, not `gogs`.
