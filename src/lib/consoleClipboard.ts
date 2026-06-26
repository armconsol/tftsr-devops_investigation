/**
 * Pure keyboard-shortcut predicates shared by the Proxmox console components
 * (noVNC graphical consoles and the xterm.js terminal).
 *
 * Consoles capture most key events for the guest/remote session, so copy and
 * paste are bound to the terminal-style chords `Ctrl/Cmd+Shift+C` and
 * `Ctrl/Cmd+Shift+V` rather than the bare `Ctrl+C`/`Ctrl+V` (which must reach
 * the guest). These predicates are intentionally free of DOM/RFB types so they
 * can be unit-tested in isolation.
 */

interface ShortcutEventLike {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
}

/** True when the event is the console copy chord (Ctrl/Cmd+Shift+C). */
export function isCopyShortcut(e: ShortcutEventLike): boolean {
  return (e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "c";
}

/** True when the event is the console paste chord (Ctrl/Cmd+Shift+V). */
export function isPasteShortcut(e: ShortcutEventLike): boolean {
  return (e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "v";
}
