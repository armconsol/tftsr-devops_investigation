// Proxmox term-proxy (xterm.js) wire protocol helpers.
//
// After the websocket opens, the client authenticates by sending a login line
// `"<user>:<ticket>\n"`. Subsequent client→server messages are framed:
//   data   -> "0:<byte-length>:<data>"
//   resize -> "1:<cols>:<rows>:"
//   ping   -> "2"
// Server→client output is raw terminal bytes written directly to xterm.

/** Build the initial authentication line sent right after the socket opens. */
export function buildLoginLine(user: string, ticket: string): string {
  return `${user}:${ticket}\n`;
}

/** Frame a user keystroke / data payload for transmission to the node. */
export function encodeData(data: string): string {
  // Length is the UTF-8 byte length, not the JS string length.
  const byteLen = new TextEncoder().encode(data).length;
  return `0:${byteLen}:${data}`;
}

/** Frame a terminal resize event. */
export function encodeResize(cols: number, rows: number): string {
  return `1:${cols}:${rows}:`;
}

/** The keep-alive ping frame. */
export function encodePing(): string {
  return "2";
}
