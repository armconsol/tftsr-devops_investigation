# Remote Desktop — Edit Dialog Closing & Black RDP Window

## Description
Two defects were reported in the Remote Desktop feature (`/remote-desktop`,
`src/pages/Remote/RemoteDesktopPage.tsx`):

1. **Edit dialog closes when clicking the SSH (or Display/Resolution) tab.**
   The connection editor is a tabbed form. Clicking the **SSH Tunnel** or **Display**
   tab caused the whole edit dialog to close instead of switching tabs.

   *Root cause:* `TabsTrigger` in `src/components/ui/index.tsx` rendered a `<button>`
   with no explicit `type`. Inside an HTML `<form>`, a button without a `type` defaults
   to `type="submit"`. Clicking a tab therefore submitted the form. In **edit** mode all
   required fields are pre-populated and the password is optional, so validation passed,
   `onSave` ran, and the dialog closed. (The Dialog close "X" button had the same latent
   missing-`type` issue.)

2. **Connecting to an RDP session shows only a black window**, even though the endpoint
   is valid (verified with the native Windows RDP client).

   *Root cause:* `src-tauri/src/remote/websocket_server.rs` serialized each RDP frame as
   **JSON** (`serde_json::to_vec`) and sent it as a binary WebSocket message. The frontend
   canvas decoder expects a **raw binary** layout: `[u32 LE width][u32 LE height][RGBA bytes]`
   (width at byte 0, height at byte 4, pixels from byte 8). The JSON bytes produced garbage
   width/height values and a length-mismatched `ImageData`, so `putImageData` failed and the
   canvas was never painted — a black window.

## Acceptance Criteria
- Clicking the **SSH Tunnel** or **Display** tab in the connection editor switches tabs
  without submitting the form or closing the dialog, in both add and edit modes.
- The Dialog close button does not submit the surrounding form.
- A connected RDP session renders the live desktop on the canvas instead of a black window.
- RDP frames are transmitted in the binary format the frontend decoder expects.
- No regressions in existing Rust or frontend test suites; lint/format/type checks pass.

## Work Implemented
- `src/components/ui/index.tsx`
  - Added `type="button"` to the `TabsTrigger` `<button>`.
  - Added `type="button"` to the `DialogContent` close `<button>`.
- `src-tauri/src/remote/websocket_server.rs`
  - Added `encode_frame(&RdpFrame) -> Vec<u8>` producing `width.to_le_bytes()` ++
    `height.to_le_bytes()` ++ `data` (the binary layout the browser decodes).
  - Replaced the JSON serialization in the frame send task with `encode_frame`.
- Tests (TDD — written failing first, then made to pass):
  - `tests/unit/RemoteDesktopForm.test.tsx`: edit-mode test asserting that clicking the
    SSH/Display tab triggers does **not** call `onSave` (form is not submitted).
  - `websocket_server.rs`: `test_encode_frame_binary_layout` asserting the byte offsets,
    little-endian width/height, pixel payload, and total length.

## Testing Needed
- **Automated (run and passing):**
  - `cargo fmt --check`, `cargo clippy -- -D warnings` — clean.
  - `cargo test` (remote/websocket module) — pass, including new `test_encode_frame_binary_layout`.
  - `npx tsc --noEmit` — clean.
  - `npm run test:run` — 467 passed / 1 skipped, including the new edit-mode tab test.
- **Manual verification (recommended):**
  - Open the Remote Desktop page, edit an existing connection, click through the
    Connection → SSH Tunnel → Display tabs; confirm the dialog stays open and edits persist.
  - Connect to a real RDP endpoint and confirm the remote desktop renders (not black),
    including after a window resize, and that mouse/keyboard input works.
  - Repeat the RDP connection over an SSH tunnel to confirm the binary stream path is
    unaffected by the tunnel.
