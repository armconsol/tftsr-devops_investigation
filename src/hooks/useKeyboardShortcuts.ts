import { useEffect, useCallback, useRef } from "react";

export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  alt?: boolean;
  shift?: boolean;
  meta?: boolean;
  callback: () => void;
  description: string;
  enabled?: boolean;
}

export function useKeyboardShortcuts(shortcuts: KeyboardShortcut[]): void {
  const shortcutsRef = useRef(shortcuts);
  shortcutsRef.current = shortcuts;

  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    for (const shortcut of shortcutsRef.current) {
      if (shortcut.enabled === false) continue;

      const ctrlMatch = shortcut.ctrl ? event.ctrlKey || event.metaKey : !event.ctrlKey && !event.metaKey;
      const altMatch = shortcut.alt ? event.altKey : !event.altKey;
      const shiftMatch = shortcut.shift ? event.shiftKey : !event.shiftKey;
      const metaMatch = shortcut.meta ? event.metaKey : !event.metaKey;

      if (
        event.key.toLowerCase() === shortcut.key.toLowerCase() &&
        ctrlMatch &&
        altMatch &&
        shiftMatch &&
        metaMatch
      ) {
        event.preventDefault();
        shortcut.callback();
        break;
      }
    }
  }, []);

  useEffect(() => {
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}

export const GLOBAL_SHORTCUTS = {
  COMMAND_PALETTE: { key: "k", ctrl: true, description: "Open command palette" },
  REFRESH: { key: "r", ctrl: true, description: "Refresh current view" },
  SEARCH: { key: "f", ctrl: true, description: "Focus search" },
  HELP: { key: "?", shift: true, description: "Show keyboard shortcuts" },
  ESCAPE: { key: "Escape", description: "Close modal/dialog" },
  NAVIGATE_UP: { key: "ArrowUp", ctrl: true, description: "Navigate up" },
  NAVIGATE_DOWN: { key: "ArrowDown", ctrl: true, description: "Navigate down" },
} as const;
