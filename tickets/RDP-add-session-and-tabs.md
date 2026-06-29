# Fix: RDP add-session failure and dead SSH/Display tabs

## Description
On the Remote Desktop page, two bugs prevented adding and configuring remote connections:

1. **Add RDP session fails** with `invalid args 'newConn' for command 'create_remote_connection': command create_remote_connection missing required key newConn`. The frontend wrapper passed the argument as `new_conn` (snake_case), but Tauri v2 maps Rust command parameters (`new_conn: NewRemoteConnection`) to **camelCase** (`newConn`) across the JS boundary. This was the only invoke in `tauriCommands.ts` using snake_case.

2. **SSH Tunnel and Display tabs do nothing** when clicked. The `Tabs` component in `ConnectionForm` was rendered as controlled with a hardcoded `value="connection"` and a no-op `onValueChange`, so the active tab could never change.

## Acceptance Criteria
- Adding a remote connection succeeds; `create_remote_connection` is invoked with the `newConn` key.
- Clicking the **SSH Tunnel** and **Display** tabs switches to and reveals their content.
- The **Connection** tab remains the default active tab.
- No regression to existing Remote Desktop unit tests.

## Work Implemented
- `src/lib/tauriCommands.ts`: `addRemoteConnectionCmd` now invokes `create_remote_connection` with `{ newConn: connection }` (was `{ new_conn: connection }`).
- `src/pages/Remote/RemoteDesktopPage.tsx`: `ConnectionForm` now holds `const [activeTab, setActiveTab] = useState('connection')` and the `Tabs` is wired with `value={activeTab} onValueChange={setActiveTab}`. Component exported for testing.
- `tests/unit/remoteDesktop.test.ts`: assertion updated to expect the `newConn` key (RED → GREEN).
- `tests/unit/RemoteDesktopForm.test.tsx` (new): renders `ConnectionForm`, clicks the SSH/Display tab triggers, and asserts the corresponding `TabsContent` wrapper toggles `block`/`hidden`.

No backend (Rust) changes were required — `create_remote_connection` / `NewRemoteConnection` were already correct.

## Testing Needed
- `npm run test:run` — Remote Desktop command + form tests pass (13/13 for the two relevant files).
- Manual: add a new RDP connection and confirm it saves without the `newConn` error.
- Manual: open the add-connection dialog and confirm SSH Tunnel and Display tabs switch and show their settings.

> Note: 3 unrelated test files (`clipboard.test.ts`, `ConsolePage.test.tsx`, `BackupPageActions.test.tsx`) fail locally because `@tauri-apps/plugin-clipboard-manager` is declared in `package.json` but not installed in the local `node_modules`. This is a pre-existing environment gap (CI installs it) and is unrelated to this change.
