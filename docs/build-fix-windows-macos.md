# Fix Windows amd64 & macOS arm64 build failures

## Description
The release build failed to compile on two platforms:

- **Windows amd64** — `ssh2::Session::userauth_pubkey_memory` did not exist. The
  default Windows crypto backend of libssh2 does not expose in-memory public-key
  authentication, which `remote/ssh_tunnel.rs::authenticate_with_key_data` relies on.
- **macOS arm64** — `secure_storage.rs` referenced the `security_framework` crate,
  which was never declared in `Cargo.toml` (18 compile errors), plus a shadowed
  `_account` parameter and several type-inference failures.

While fixing macOS, a latent defect was found: the `keyring` crate (3.x) enables
**no backend by default**, so the OS keychain silently used an in-memory mock on
all platforms, and the backend cached a single `keyring::Entry` — collapsing every
credential into one slot regardless of the requested account.

## Acceptance Criteria
- Windows and macOS targets compile cleanly.
- macOS keychain uses a real native backend, not the in-memory mock.
- Credentials are isolated per account (no cross-account collision/leak).
- `cargo fmt --check`, `cargo clippy -D warnings`, and the full test suite pass.

## Work Implemented
- **`src-tauri/Cargo.toml`**
  - `ssh2` now built with the `vendored-openssl` feature so `userauth_pubkey_memory`
    is available on every platform.
  - Added target-specific `keyring` backends: `apple-native` (macOS),
    `windows-native` (Windows), `sync-secret-service` (Linux).
- **`src-tauri/src/secure_storage.rs`**
  - Removed the non-compiling custom `security_framework` implementation.
  - Collapsed the platform enum variants into a single `Keyring` marker; `store`,
    `get`, and `delete` build a fresh `keyring::Entry::new(service_name, account)`
    per call, fixing the per-account collision across all platforms.
- **`src-tauri/src/remote/ssh_tunnel.rs`** — added a unit test covering the
  in-memory key-data auth wiring.
- **`docs/wiki/Troubleshooting.md`** — documented both build failures and fixes.

## Testing Needed
- CI build/test on Windows amd64, macOS arm64/Intel, and Linux amd64/arm64
  (confirm `sync-secret-service` links cleanly in the Linux builder image).
- Manual verification that SSH credentials persist to the OS keychain and that
  distinct hosts/accounts do not overwrite one another.

### Testing Performed (local, macOS arm64)
- `cargo fmt --check` — clean
- `cargo clippy -- -D warnings` — clean
- `cargo test -- --test-threads=1` — 669 passed, 0 failed, 7 ignored
- New `test_macos_keychain_roundtrip` exercises the real Keychain and asserts
  per-account isolation; `test_ssh_tunnel_with_key_data` covers the in-memory path.
