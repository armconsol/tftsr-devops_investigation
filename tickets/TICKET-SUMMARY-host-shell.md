# Ticket Summary — Host Shell Console on Proxmox | Remotes

## Description
Add a **Console (Shell)** action to the **Proxmox | Remotes** page so users can
open a host (node) shell for a stored remote, reusing the in-app noVNC console
infrastructure. Because the two Proxmox products expose different shell APIs,
two code paths are implemented:

- **PVE** remotes use `POST /nodes/{node}/vncshell?websocket=1` and render a
  graphical shell with **noVNC** (reusing the existing local WebSocket proxy and
  RFB renderer).
- **PBS** remotes use `POST /nodes/{node}/termproxy` and render a text terminal
  with **xterm.js** (PBS has no `vncshell`).

A node picker lists the remote's nodes (auto-skipped when a remote has a single
node). The local proxy injects the correct auth cookie (`PVEAuthCookie` vs
`PBSAuthCookie`) per product and accepts the node's self-signed TLS certificate.

## Acceptance Criteria
- A "Console (Shell)" item appears in the Remotes actions menu for both PVE and
  PBS remotes.
- Selecting it shows a node picker; choosing a node opens the shell. Single-node
  remotes open directly.
- PVE opens a graphical noVNC shell using the separate `password` returned by
  `vncshell` as the RFB password.
- PBS opens an xterm.js terminal that authenticates and exchanges framed term
  data per the Proxmox term-proxy protocol.
- All new and pre-existing tests pass; fmt/clippy/tsc/eslint/build are clean.

## Work Implemented
**Backend (`src-tauri`):**
- `proxmox/console.rs`: added optional `password` to `VncProxyInfo` (+ parsing);
  `build_node_vncwebsocket_url`; made `build_auth_cookie` cookie-name aware;
  added `vncshell_node()` and `termproxy_node()`; `start_vnc_proxy` now takes a
  cookie name.
- `commands/proxmox.rs`: new `open_node_shell` command returning a tagged
  `NodeShellSession { kind, localUrl, ticket, localPort, password, user }`,
  selecting `vncshell`/noVNC for PVE and `termproxy`/xterm for PBS by
  `cluster_type`.
- `lib.rs`: registered `open_node_shell`.

**Frontend (`src`):**
- `lib/proxmoxClient.ts`: `openNodeShell` wrapper + `NodeShellSession` type.
- `lib/termproxy.ts`: pure term-proxy framing helpers (login line, data, resize,
  ping).
- `components/Proxmox/XtermConsole.tsx`: xterm.js terminal bound to the term
  proxy websocket.
- `components/Proxmox/NodeShellConsole.tsx`: fetches the session and renders
  noVNC or xterm by `kind`.
- `pages/Proxmox/ShellPage.tsx` + route `/proxmox/shell/:clusterId/:node`.
- `components/Proxmox/RemotesList.tsx`: "Console (Shell)" action + `onShell`.
- `pages/Proxmox/RemotesPage.tsx`: node-picker dialog + navigation.
- Added `@xterm/xterm` + `@xterm/addon-fit` dependencies.

**Docs:** `docs/wiki/IPC-Commands.md` — `open_node_shell` entry.

## Testing Needed
- Automated (passing): Rust unit tests for `vncshell`/`termproxy` parsing
  (incl. password), node-ws URL, cookie builder; frontend unit tests for the
  term-proxy framing and the RemotesList "Console (Shell)" action.
- Manual against live infrastructure:
  - PVE remote → graphical host shell renders and is interactive.
  - PBS remote → xterm terminal authenticates (`user@pam` ticket) and is
    interactive; confirm `PBSAuthCookie` behavior.
  - Multi-node remote shows the picker; single-node opens directly.
