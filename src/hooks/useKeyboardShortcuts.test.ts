import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { useKeyboardShortcuts } from "./useKeyboardShortcuts";

describe("useKeyboardShortcuts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("triggers callback on matching shortcut", () => {
    const callback = vi.fn();
    renderHook(() =>
      useKeyboardShortcuts([
        {
          key: "k",
          ctrl: true,
          callback,
          description: "Test shortcut",
        },
      ])
    );

    const event = new KeyboardEvent("keydown", { key: "k", ctrlKey: true });
    document.dispatchEvent(event);

    expect(callback).toHaveBeenCalledTimes(1);
  });

  it("does not trigger on non-matching shortcut", () => {
    const callback = vi.fn();
    renderHook(() =>
      useKeyboardShortcuts([
        {
          key: "k",
          ctrl: true,
          callback,
          description: "Test shortcut",
        },
      ])
    );

    const event = new KeyboardEvent("keydown", { key: "j", ctrlKey: true });
    document.dispatchEvent(event);

    expect(callback).not.toHaveBeenCalled();
  });

  it("respects modifier key requirements", () => {
    const callback = vi.fn();
    renderHook(() =>
      useKeyboardShortcuts([
        {
          key: "k",
          ctrl: true,
          shift: true,
          callback,
          description: "Test shortcut",
        },
      ])
    );

    // Without shift
    let event = new KeyboardEvent("keydown", { key: "k", ctrlKey: true });
    document.dispatchEvent(event);
    expect(callback).not.toHaveBeenCalled();

    // With shift
    event = new KeyboardEvent("keydown", {
      key: "k",
      ctrlKey: true,
      shiftKey: true,
    });
    document.dispatchEvent(event);
    expect(callback).toHaveBeenCalledTimes(1);
  });

  it("handles alt modifier", () => {
    const callback = vi.fn();
    renderHook(() =>
      useKeyboardShortcuts([
        {
          key: "k",
          alt: true,
          callback,
          description: "Test shortcut",
        },
      ])
    );

    const event = new KeyboardEvent("keydown", { key: "k", altKey: true });
    document.dispatchEvent(event);

    expect(callback).toHaveBeenCalledTimes(1);
  });

  it("skips disabled shortcuts", () => {
    const callback = vi.fn();
    renderHook(() =>
      useKeyboardShortcuts([
        {
          key: "k",
          ctrl: true,
          callback,
          description: "Test shortcut",
          enabled: false,
        },
      ])
    );

    const event = new KeyboardEvent("keydown", { key: "k", ctrlKey: true });
    document.dispatchEvent(event);

    expect(callback).not.toHaveBeenCalled();
  });

  it("handles multiple shortcuts", () => {
    const callback1 = vi.fn();
    const callback2 = vi.fn();
    renderHook(() =>
      useKeyboardShortcuts([
        {
          key: "k",
          ctrl: true,
          callback: callback1,
          description: "Shortcut 1",
        },
        {
          key: "r",
          ctrl: true,
          callback: callback2,
          description: "Shortcut 2",
        },
      ])
    );

    let event = new KeyboardEvent("keydown", { key: "k", ctrlKey: true });
    document.dispatchEvent(event);
    expect(callback1).toHaveBeenCalledTimes(1);
    expect(callback2).not.toHaveBeenCalled();

    event = new KeyboardEvent("keydown", { key: "r", ctrlKey: true });
    document.dispatchEvent(event);
    expect(callback2).toHaveBeenCalledTimes(1);
  });

  it("prevents default on matched shortcuts", () => {
    const callback = vi.fn();
    renderHook(() =>
      useKeyboardShortcuts([
        {
          key: "k",
          ctrl: true,
          callback,
          description: "Test shortcut",
        },
      ])
    );

    const event = new KeyboardEvent("keydown", { key: "k", ctrlKey: true });
    const preventDefaultSpy = vi.spyOn(event, "preventDefault");
    document.dispatchEvent(event);

    expect(preventDefaultSpy).toHaveBeenCalled();
  });

  it("handles meta key as ctrl on macOS", () => {
    const callback = vi.fn();
    renderHook(() =>
      useKeyboardShortcuts([
        {
          key: "k",
          ctrl: true,
          callback,
          description: "Test shortcut",
        },
      ])
    );

    const event = new KeyboardEvent("keydown", { key: "k", metaKey: true });
    document.dispatchEvent(event);

    expect(callback).toHaveBeenCalledTimes(1);
  });

  it("cleans up event listener on unmount", () => {
    const callback = vi.fn();
    const { unmount } = renderHook(() =>
      useKeyboardShortcuts([
        {
          key: "k",
          ctrl: true,
          callback,
          description: "Test shortcut",
        },
      ])
    );

    unmount();

    const event = new KeyboardEvent("keydown", { key: "k", ctrlKey: true });
    document.dispatchEvent(event);

    expect(callback).not.toHaveBeenCalled();
  });
});
