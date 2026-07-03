# Description
`ConnectionConfig` gained a new `ssh_tunnel_config` field, but several constructors were not updated, causing Rust builds and tests to fail.

# Acceptance Criteria
- Every `ConnectionConfig` literal compiles with the new `ssh_tunnel_config` field.
- `cargo fmt`, `cargo clippy`, and `cargo test` pass for `src-tauri`.
- Changes are committed on a new branch and pushed to Gitea.

# Work Implemented
- Added `ssh_tunnel_config: None` to the missing `ConnectionConfig` initializers in the database loader and affected driver tests.
- Kept the fix scoped to the compile break rather than refactoring unrelated database code.

# Testing Needed
- Re-run `cargo fmt --manifest-path src-tauri/Cargo.toml`
- Re-run `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
- Re-run `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1`
