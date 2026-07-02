# Ticket Summary — RDP Black Screen (WebSocket lifecycle)

## Description
Backend disk logs showed a repeatable black-screen path during RDP connect:
- WebSocket connected, then reset immediately.
- RDP decode loop continued producing frames.
- Frame forwarding then failed with repeated `Session not found` warnings.

Root cause was a WebSocket lifecycle/routing mismatch: connection teardown removed frame-routing state while the RDP session stayed active.

## Acceptance Criteria
- RDP frame routing is not torn down by transient WebSocket disconnects.
- WebSocket handshake requires the expected `binary` subprotocol.
- Session attach requires an unguessable per-session token.
- Only one active controlling WebSocket client can attach to a session.
- Frontend requests the required subprotocol and uses tokenized session URL.
- Existing frontend checks pass; Rust formatting passes.

## Work Implemented
- Updated `src-tauri/src/remote/websocket_server.rs`:
  - Switched frame routing to per-session `broadcast` channels to preserve routing across reconnect windows.
  - Added per-session auth token generation and token validation on connect.
  - Enforced `binary` subprotocol at handshake (reject mismatches).
  - Enforced single active controller per session.
  - Reduced frame channel depth to limit buffering pressure.
  - Added tests for subprotocol parsing, token extraction, and disconnect/routing persistence.
- Updated `src-tauri/src/remote/rdp.rs`:
  - Stored and propagated per-session WebSocket token.
  - Returned tokenized WebSocket URLs for active sessions.
- Updated `src/pages/Remote/RemoteDesktopPage.tsx`:
  - WebSocket now explicitly requests `binary` subprotocol.

## Testing Needed
- Rust:
  - `cargo fmt --manifest-path src-tauri/Cargo.toml --check` ✅
  - `cargo test --manifest-path src-tauri/Cargo.toml websocket_server -- --test-threads=1` ⛔ blocked (missing `/tmp/ironrdp-patch/crates/ironrdp`)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` ⛔ blocked (same dependency path)
- Frontend:
  - `npx tsc --noEmit` ✅
  - `npx eslint src/ tests/ --quiet` ✅
  - `npm run test:run -- tests/unit/pages/Remote/RemoteDesktopPage.test.tsx` ✅
