# Ticket Summary — Usable RDP Client (Fix Black Screen + Wire Input)

## Description

Connecting to a saved RDP session rendered only a **black screen**, and
keyboard/mouse input was completely non-functional. The goal of this feature set
is a **usable** in-app RDP client: live video plus working keyboard and mouse.

Using a live `#[ignore]` integration test driven against a real RDP host
(reachable from the build environment), the black screen was traced to a chain of
backend defects, each masking the next, plus a final frame-delivery bug and a
fully unwired input path.

### Root causes fixed

1. **Blocking socket handed to tokio** — `from_std` on a socket with a read
   timeout panicked.
2. **No rustls `CryptoProvider`** — rustls 0.23 panicked mid-handshake because
   both `ring` and `aws-lc-rs` are in the tree (now installed once via
   `ensure_crypto_provider`).
3. **Wrong IronRDP handshake ordering** — the `ShouldUpgrade` token is now
   threaded from `connect_begin` into `mark_as_upgraded`; the async-TLS +
   `block_on` path was replaced with a synchronous rustls handshake.
4. **Slow-path graphics unsupported** — xrdp-style `ShareDataPdu::Update` frames
   aborted the session; fixed by upgrading to the ironrdp 0.16 generation
   (ironrdp-session ≥ 0.10). MSRV rises to Rust 1.89.
5. **Frames produced but never delivered (the persistent black screen)** — the
   frame-forwarding task awaited `frame_rx.recv()`, but channel wakeups land on
   the blocking `connect()` worker's local run-queue, which it never drains while
   parked in a socket read, so `recv().await` never woke. Fixed by polling
   `try_recv()` with a 5 ms async sleep.
6. **Input dead** — the webview sent input JSON over the WebSocket, but the server
   only logged inbound messages. Input is now decoded, routed per-session, and
   emitted as IronRDP fastpath input.

## Acceptance Criteria

- [x] Connecting to a saved RDP session renders the real desktop (non-black
      frames), not a black screen.
- [x] Keyboard input reaches the remote session (scancode set 1; extended keys
      handled).
- [x] Mouse movement, buttons (left/middle/right), and wheel work; coordinates
      scale correctly from the stretched canvas to the RDP resolution.
- [x] The session is responsive (input is serviced promptly even when the server
      sends no graphics).
- [x] Release builds do not break after the dependency/toolchain bump (Rust
      1.89): release profile compiles cleanly; CI builder images and release
      workflows updated to `rust1.89-node22`.
- [x] Full quality gate passes (fmt, clippy `-D warnings`, Rust tests, tsc,
      eslint, vitest).
- [x] New input-handling code reviewed for security; findings remediated.

## Work Implemented

**Backend (`src-tauri/src/remote/`)**
- `rdp_client.rs`: synchronous handshake fixes; frame forwarder switched to a
  polling `try_recv` + 5 ms sleep loop; read-timeout duality (30 s during
  handshake → 50 ms after `connect_finalize`); full input emission
  (`handle_input`, `send_keyboard_input`, `send_mouse_move`, `send_mouse_input`,
  `send_mouse_wheel`).
- `input.rs` (**new**): `RawInputEvent` wire type, JS `KeyboardEvent.code` → RDP
  scancode map, coordinate clamping, 12 unit tests.
- `websocket_server.rs`: per-session input channel + `register_input_sender`;
  decode inbound JSON input frames; 4 KiB WebSocket message/frame cap.
- `rdp.rs`: Arc the session, create the input channel, spawn the input-dispatch
  task.
- `mod.rs`: register the new `input` module.

**Frontend**
- `src/pages/Remote/RemoteDesktopPage.tsx`: coordinate scaling
  (`toRemoteCoords`), always-on mouse-move, wheel handler, context-menu
  prevention, canvas focus on mousedown.

**Dependencies / toolchain**
- ironrdp 0.14 → 0.16 generation; sspi bump; new `connector::Config` fields
  defaulted. MSRV → Rust 1.89.
- CI: `.docker/Dockerfile.{linux-amd64,linux-arm64,windows-cross}` and
  `auto-tag.yml` / `release-beta.yml` / `build-images.yml` bumped to Rust 1.89 /
  `rust1.89-node22`. PR gate (`test.yml`, nightly) unaffected.

**Security remediation** (from review of the new input path)
- `send_mouse_wheel`: `saturating_neg()` to avoid an `i32::MIN` negation panic
  from attacker-controlled wheel delta.
- WebSocket: 4 KiB `max_message_size` / `max_frame_size` to prevent large-frame
  memory pressure (also bounds the keyboard `code` string allocation).
- Confirmed sound: session-id validation, coordinate clamping, bounded input
  channel with `try_send` drop-on-saturation.

**Docs**
- `docs/wiki/Architecture.md`: new “Remote Desktop (RDP)” section (pipeline +
  implementation notes).
- `docs/wiki/Troubleshooting.md`: entries for the frame-delivery black screen and
  dead input.

## Testing Needed

- **Automated (done):** cargo fmt clean; clippy `-D warnings` clean; 686 Rust
  tests pass; tsc clean; eslint clean; vitest 468 pass; release build compiles.
- **Live backend e2e (done):** `tests/rdp_live.rs` against the real host —
  `connected=true`, non-black frames, input injected, completes in ~0.45 s.
- **Manual GUI confirmation (recommended):** run `cargo tauri dev`, connect to a
  saved session, and verify the desktop renders and that keyboard, mouse buttons,
  movement, and wheel behave correctly. Visually confirm color order (RGBA vs
  BGRA) on the rendered desktop.
- **Cross-target release builds (CI):** confirm linux amd64/arm64 and windows
  cross builds succeed on the new `rust1.89-node22` builder images after
  `build-images.yml` rebuilds them.

### Known limitations / out of scope
- The SSH-tunnel path does not perform TLS-over-SSH (unchanged; out of scope).
- On connect failure the WebSocket stays up with no frames; surfacing a richer
  disconnected state to the UI is deferred.
