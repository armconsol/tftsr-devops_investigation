import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { usePolling } from "@/hooks/usePolling";

describe("usePolling", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("fires immediately when enabled", async () => {
    const fn = vi.fn().mockResolvedValue(undefined);
    renderHook(() => usePolling(fn, 1000, true));

    await vi.waitFor(() => expect(fn).toHaveBeenCalledTimes(1));
  });

  it("does not fire when disabled", async () => {
    const fn = vi.fn().mockResolvedValue(undefined);
    renderHook(() => usePolling(fn, 1000, false));

    await vi.advanceTimersByTimeAsync(5000);
    expect(fn).not.toHaveBeenCalled();
  });

  it("re-fires on the configured interval", async () => {
    const fn = vi.fn().mockResolvedValue(undefined);
    renderHook(() => usePolling(fn, 1000, true));

    await vi.waitFor(() => expect(fn).toHaveBeenCalledTimes(1));
    await vi.advanceTimersByTimeAsync(1000);
    expect(fn).toHaveBeenCalledTimes(2);
    await vi.advanceTimersByTimeAsync(2000);
    expect(fn).toHaveBeenCalledTimes(4);
  });

  it("stops firing after unmount", async () => {
    const fn = vi.fn().mockResolvedValue(undefined);
    const { unmount } = renderHook(() => usePolling(fn, 1000, true));

    await vi.waitFor(() => expect(fn).toHaveBeenCalledTimes(1));
    unmount();
    await vi.advanceTimersByTimeAsync(5000);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it("stops firing when toggled from enabled to disabled", async () => {
    const fn = vi.fn().mockResolvedValue(undefined);
    const { rerender } = renderHook(
      ({ enabled }) => usePolling(fn, 1000, enabled),
      { initialProps: { enabled: true } }
    );

    await vi.waitFor(() => expect(fn).toHaveBeenCalledTimes(1));
    rerender({ enabled: false });
    await vi.advanceTimersByTimeAsync(5000);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it("restarts immediate fire when re-enabled after being disabled", async () => {
    const fn = vi.fn().mockResolvedValue(undefined);
    const { rerender } = renderHook(
      ({ enabled }) => usePolling(fn, 1000, enabled),
      { initialProps: { enabled: false } }
    );

    await vi.advanceTimersByTimeAsync(2000);
    expect(fn).not.toHaveBeenCalled();

    rerender({ enabled: true });
    await vi.waitFor(() => expect(fn).toHaveBeenCalledTimes(1));
  });

  it("does not throw an unhandled rejection when fn rejects, and keeps polling", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    const fn = vi.fn().mockRejectedValue(new Error("boom"));
    renderHook(() => usePolling(fn, 1000, true));

    await vi.waitFor(() => expect(fn).toHaveBeenCalledTimes(1));
    await vi.waitFor(() => expect(consoleError).toHaveBeenCalledWith(
      expect.stringContaining("usePolling"),
      expect.any(Error)
    ));

    await vi.advanceTimersByTimeAsync(1000);
    expect(fn).toHaveBeenCalledTimes(2);

    consoleError.mockRestore();
  });
});
