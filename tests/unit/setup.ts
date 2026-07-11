import { vi, beforeAll, afterAll } from "vitest";
import "@testing-library/jest-dom/vitest";

// Node 25 ships a native localStorage stub that lacks .clear() unless --localstorage-file is set.
// Replace it with a real in-memory implementation so tests relying on localStorage work correctly.
function makeStorage() {
  let store: Record<string, string> = {};
  return {
    get length() { return Object.keys(store).length; },
    key(i: number) { return Object.keys(store)[i] ?? null; },
    getItem(k: string) { return store[k] ?? null; },
    setItem(k: string, v: string) { store[k] = String(v); },
    removeItem(k: string) { delete store[k]; },
    clear() { store = {}; },
  };
}
Object.defineProperty(globalThis, "localStorage", { value: makeStorage(), writable: true });
Object.defineProperty(globalThis, "sessionStorage", { value: makeStorage(), writable: true });

class ResizeObserverMock {
  observe() {}
  unobserve() {}
  disconnect() {}
}

Object.defineProperty(globalThis, "ResizeObserver", {
  value: ResizeObserverMock,
  writable: true,
});

// Mock Tauri core API
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

// Mock Tauri event API
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(() => Promise.resolve()),
  once: vi.fn(() => Promise.resolve(() => {})),
}));

// Mock Tauri dialog plugin
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(() => Promise.resolve(null)),
  save: vi.fn(() => Promise.resolve(null)),
  message: vi.fn(() => Promise.resolve()),
  ask: vi.fn(() => Promise.resolve(false)),
  confirm: vi.fn(() => Promise.resolve(false)),
}));

// Mock Tauri fs plugin
vi.mock("@tauri-apps/plugin-fs", () => ({
  readTextFile: vi.fn(() => Promise.resolve("")),
  writeTextFile: vi.fn(() => Promise.resolve()),
  readFile: vi.fn(() => Promise.resolve(new Uint8Array())),
  writeFile: vi.fn(() => Promise.resolve()),
  mkdir: vi.fn(() => Promise.resolve()),
  exists: vi.fn(() => Promise.resolve(false)),
}));

// Mock Tauri clipboard-manager plugin (console copy/paste)
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  readText: vi.fn(() => Promise.resolve("")),
  writeText: vi.fn(() => Promise.resolve()),
}));

// Mock console.error to suppress React warnings
const originalError = console.error;
beforeAll(() => {
  console.error = (...args: unknown[]) => {
    if (typeof args[0] === "string" && args[0].includes("Warning:")) return;
    originalError(...args);
  };
});
afterAll(() => {
  console.error = originalError;
});
