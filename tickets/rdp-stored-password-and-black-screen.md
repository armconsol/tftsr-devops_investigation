# Remote Desktop (RDP) — Stop Re-Prompting for Saved Password & Fix Black Screen

## Description
Two defects remained in the Remote Desktop feature (`/remote-desktop`,
`src/pages/Remote/RemoteDesktopPage.tsx`):

1. **Password re-prompt** — Even though the RDP password was saved on the
   connection entry, the user was prompted for the password again before
   connecting.

   *Root cause:* The RDP password is stored encrypted
   (`remote_credentials.rdp_password_encrypted`), but there was **no function to
   retrieve/decrypt it** (only `get_remote_ssh_credentials` existed). The
   `start_rdp_session` Tauri command required a `password` argument, so the
   frontend always showed a "Connect" password dialog.

2. **Black screen** — Connecting showed only a black canvas instead of the
   remote desktop.

   *Root cause (two parts):*
   - `start_rdp_session` called the legacy **sync** `RdpManager::start_session`,
     which never connected via IronRDP and never started a WebSocket server — it
     returned a `ws://` URL pointing at a dead port, so no frames were ever sent.
   - On the async path, the session id used by the frame sender,
     `register_session`, the `ws://.../rdp/{id}` URL, and `handle_client` did not
     match (each generated its own UUID), so frames could not be routed to the
     client.

## Acceptance Criteria
- Connecting to an RDP connection that has a saved password does **not** prompt
  for a password.
- If no password is saved (or the stored one fails), the password dialog appears
  as a fallback and connecting with an entered password works.
- A connected RDP session renders the live desktop on the canvas instead of a
  black window.
- Decrypted passwords are never logged or sent to the frontend.
- No regressions; lint/format/type checks and existing test suites pass.

## Work Implemented
- **`src-tauri/src/remote/connection.rs`** — Added
  `get_remote_rdp_password(conn, connection_id) -> Result<Option<String>, String>`
  that decrypts `rdp_password_encrypted` via `integrations::auth::decrypt_token`.
- **`src-tauri/src/commands/remote.rs`** — Made `start_rdp_session` `async` with
  `password: Option<String>`; falls back to the stored RDP password when the
  argument is omitted/blank; drives the real async connect path
  (`start_session_with_password`). Manager is cloned (Arc) so no `MutexGuard` is
  held across the `await`.
- **`src-tauri/src/remote/rdp.rs`** — `start_session_async` now starts the
  WebSocket listener, calls `register_session` with the canonical session id,
  binds the `RdpClientSession` to that id, and returns a matching
  `ws://127.0.0.1:{port}/rdp/{id}` URL.
- **`src-tauri/src/remote/rdp_client.rs`** — Added `RdpClientSession::new_with_id`
  and `RdpConnectionHandler::create_session_with_id` so the frame-forwarding task
  sends under the canonical id.
- **`src-tauri/src/remote/websocket_server.rs`** — Added `session_id_from_path`;
  `handle_client` now parses the session id from the `/rdp/{id}` request path
  (instead of a fresh UUID) and **rejects connections for unregistered session
  ids** (defense-in-depth).
- **`src/lib/tauriCommands.ts`** — `startRdpSession(connectionId, password?)`
  (password optional).
- **`src/pages/Remote/RemoteDesktopPage.tsx`** — Clicking **Connect** attempts a
  connection using stored credentials with no prompt; the password dialog is
  shown only as a fallback on failure.
- **Docs** — Added an RDP section to `docs/wiki/Troubleshooting.md`.

## Testing Needed
- **Automated (run and passing):**
  - `cargo fmt --check`, `cargo clippy -- -D warnings` — clean.
  - `cargo test` (remote module) — pass, incl. new
    `get_remote_rdp_password` round-trip/missing tests and
    `session_id_from_path`.
  - `npx tsc --noEmit`, `npx eslint` — clean.
  - `npm run test:run` — 468 passed / 1 skipped, incl. the new optional-password
    `startRdpSession` test.
- **Manual verification (recommended):**
  - Connect to an RDP endpoint that has a saved password — confirm **no**
    password prompt and the live desktop renders (not black).
  - Connect to a connection with no saved password — confirm the fallback
    password dialog appears and connecting works.
  - Verify the same over an SSH tunnel.
  - Confirm dynamic resize still renders correctly.
