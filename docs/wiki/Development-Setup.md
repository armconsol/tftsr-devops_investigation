# Development Setup

## Prerequisites

### System (Linux/Fedora)

```bash
sudo dnf install -y glib2-devel gtk3-devel webkit2gtk4.1-devel \
  libsoup3-devel openssl-devel librsvg2-devel
```

### Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Minimum required version: **Rust 1.88** (needed by `cookie_store`, `time`, `darling`).

### Node.js

Node **v22** required. Install via nvm or system package manager.

### Project Dependencies

```bash
npm install --legacy-peer-deps
```

---

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `TRCAA_DATA_DIR` (or legacy `TRCAA_DATA_DIR`) | Platform data dir | Override DB location |
| `TRCAA_DB_KEY` (or legacy `TRCAA_DB_KEY`) | _(none)_ | DB encryption key (required in release builds) |
| `TRCAA_ENCRYPTION_KEY` (or legacy `TRCAA_ENCRYPTION_KEY`) | _(none)_ | Credential encryption key (required in release builds) |
| `RUST_LOG` | `info` | Tracing verbosity: `debug`, `info`, `warn`, `error` |

Application data is stored at:
- **Linux:** `~/.local/share/tftsr/`
- **macOS:** `~/Library/Application Support/tftsr/`
- **Windows:** `%APPDATA%\tftsr\`

---

## Development Commands

### Start Full Dev Environment

```bash
source ~/.cargo/env
cargo tauri dev
```

Hot reload: Vite (frontend at `localhost:1420`) + Tauri (Rust recompiles on save).

### Frontend Only

```bash
npm run dev
# → http://localhost:1420
```

---

## Testing

```bash
# Rust unit tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run a single test module
cargo test --manifest-path src-tauri/Cargo.toml pii::detector

# Run a single test by name
cargo test --manifest-path src-tauri/Cargo.toml test_detect_ipv4

# Frontend tests (single run)
npm run test:run

# Frontend tests (watch mode)
npm run test

# Frontend coverage report
npm run test:coverage

# TypeScript type check
npx tsc --noEmit
```

Current test status: **13/13 frontend tests passing**, **64/64 Rust tests passing**.

---

## Linting & Formatting

```bash
# Rust format check
cargo fmt --manifest-path src-tauri/Cargo.toml --check

# Auto-format
cargo fmt --manifest-path src-tauri/Cargo.toml

# Rust lints (all warnings as errors)
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Quick Rust type check (no linking)
cargo check --manifest-path src-tauri/Cargo.toml
```

---

## Production Build

```bash
cargo tauri build
# → src-tauri/target/release/bundle/
# Outputs: .deb, .rpm, .AppImage (Linux)
```

Release builds enforce secure key configuration. Set both `TRCAA_DB_KEY` (or legacy `TRCAA_DB_KEY`) and `TRCAA_ENCRYPTION_KEY` (or legacy `TRCAA_ENCRYPTION_KEY`) before building.

---

## Rust Design Patterns

### Mutex Release Before Await

`MutexGuard` is not `Send`. Always release the lock before any `.await`:

```rust
// ✅ CORRECT — release lock before await
let value = {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.query_row(...)?
};   // ← lock released here
some_async_call().await?;

// ❌ WRONG — compile error: MutexGuard not Send across await
let db = state.db.lock()?;
let result = some_async_call().await?;  // ERROR
```

### Database Queries (Lifetime Issue)

Use `conn.prepare().and_then(...)` pattern:

```rust
// ✅ CORRECT
let rows = conn.prepare("SELECT ...")
    .and_then(|mut stmt| stmt.query_map(params![], |row| { ... })?.collect())?;

// ❌ causes lifetime issues in async context
let mut stmt = conn.prepare("SELECT ...")?;
let rows = stmt.query_map(...)?;
```

### Command Handler Pattern

```rust
#[tauri::command]
pub async fn my_command(
    param: String,
    state: State<'_, AppState>,
) -> Result<ResponseType, String> {
    let result = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.query_row("SELECT ...", params![param], |row| { ... })
            .map_err(|e| e.to_string())?
    };
    Ok(result)
}
```
