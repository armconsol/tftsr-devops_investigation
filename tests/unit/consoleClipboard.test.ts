import { describe, it, expect } from "vitest";
import {
  isCopyShortcut,
  isPasteShortcut,
} from "../../src/lib/consoleClipboard";

// Minimal shape of the KeyboardEvent fields the predicates inspect. Using a
// plain object keeps these pure-logic tests free of jsdom event construction.
function key(
  k: string,
  mods: { ctrl?: boolean; shift?: boolean; meta?: boolean; alt?: boolean } = {}
): KeyboardEvent {
  return {
    key: k,
    ctrlKey: !!mods.ctrl,
    shiftKey: !!mods.shift,
    metaKey: !!mods.meta,
    altKey: !!mods.alt,
  } as KeyboardEvent;
}

describe("console clipboard shortcuts", () => {
  it("treats Ctrl+Shift+V as paste", () => {
    expect(isPasteShortcut(key("V", { ctrl: true, shift: true }))).toBe(true);
    expect(isPasteShortcut(key("v", { ctrl: true, shift: true }))).toBe(true);
  });

  it("treats Cmd+Shift+V (mac) as paste", () => {
    expect(isPasteShortcut(key("v", { meta: true, shift: true }))).toBe(true);
  });

  it("does not treat plain Ctrl+V as the console paste shortcut", () => {
    expect(isPasteShortcut(key("v", { ctrl: true }))).toBe(false);
  });

  it("treats Ctrl+Shift+C as copy", () => {
    expect(isCopyShortcut(key("C", { ctrl: true, shift: true }))).toBe(true);
    expect(isCopyShortcut(key("c", { ctrl: true, shift: true }))).toBe(true);
  });

  it("treats Cmd+Shift+C (mac) as copy", () => {
    expect(isCopyShortcut(key("c", { meta: true, shift: true }))).toBe(true);
  });

  it("does not treat plain Ctrl+C as the console copy shortcut", () => {
    expect(isCopyShortcut(key("c", { ctrl: true }))).toBe(false);
  });

  it("ignores unrelated keys", () => {
    expect(isPasteShortcut(key("x", { ctrl: true, shift: true }))).toBe(false);
    expect(isCopyShortcut(key("x", { ctrl: true, shift: true }))).toBe(false);
  });
});
