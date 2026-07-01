# Fix: Prevent RDP Black Screen from Premature Session Stop

## Description
Fixes the black screen issue when connecting to RDP sessions by preventing premature session termination.

## Root Cause Analysis
The RDP session was being stopped when the WebSocket component unmounted, before the WebSocket client could properly connect and start receiving frames.

**Timeline from debug logs:**
```
20:33:53.814  INFO  RDP session stopped: 019f1f63-5e8b-7a22-9700-dfb1a83af559
20:33:53.827  INFO  WebSocket client connected for session: 019f1f63-5e8b-7a22-9700-dfb1a83af559
20:33:53.828  ERROR WebSocket error: WebSocket protocol error: Connection reset without closing handshake
```

The session was unregistered at `.814`, but the WebSocket tried to connect at `.827` - the session was already gone!

## Solution
Removed the `stopRdpSession()` call from the WebSocket cleanup function in `RemoteDesktopPage.tsx`. The RDP session should only be stopped when the user explicitly disconnects, not when the WebSocket component unmounts or reconnects.

## Changes

### Frontend
**File:** `src/pages/Remote/RemoteDesktopPage.tsx`

Removed premature session stop:
```typescript
return () => {
  ws.close();
  // Do NOT stop the RDP session here - it should only be stopped when
  // the user explicitly disconnects. The WebSocket might reconnect,
  // and stopping the session prematurely causes the black screen issue.
};
```

### Backend
**File:** `src-tauri/src/remote/websocket_server.rs`

1. Fixed corrupted syntax (replaced `antml::<parameter>` tags with proper `anyhow::anyhow!`)
2. Added debug logging for WebSocket handshake
3. Added regression test `test_frame_broadcast_after_registration()`

## Testing
- ✅ All existing tests pass (8 tests in websocket_server module)
- ✅ New regression test added and passing
- ✅ `cargo fmt --check`: pass
- ✅ `cargo clippy`: pass (0 errors)
- ✅ TypeScript type check: pass

## Impact
- RDP connections now display properly without black screen
- WebSocket reconnections work without restarting the RDP session
- No breaking changes to API or behavior

## Related
- Debug logs at: `/tmp/rdp_debug_*.log`
- Issue: RDP black screen with "Session not found" errors
