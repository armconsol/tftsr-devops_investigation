# Description
`ConnectionConfig` gained a new `ssh_tunnel_config` field, and the database loader also needed to restore persisted SSH tunnel metadata.

# Acceptance Criteria
- Every `ConnectionConfig` literal compiles with the new `ssh_tunnel_config` field.
- `cargo fmt`, `cargo clippy`, and `cargo test` pass for `src-tauri`.
- Changes are committed on a new branch and pushed to Gitea.

# Work Implemented
- Added `ssh_tunnel_config` to the missing `ConnectionConfig` initializers in the database loader and affected driver tests.
- Restored SSH tunnel metadata when loading saved database connections so the field is no longer always empty.
- Kept the fix scoped to the compile break and its direct follow-on bug rather than refactoring unrelated database code.

# Testing Needed
- Re-run `cargo fmt --manifest-path src-tauri/Cargo.toml`
- Re-run `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
- Re-run `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1`
