import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useProxmoxNodes } from "@/hooks/useProxmoxNodes";

vi.mock("@tauri-apps/api/core");

type MockedInvoke = typeof invoke & {
  mockResolvedValue: (v: unknown) => void;
  mockRejectedValue: (e: unknown) => void;
  mockReset: () => void;
};
const mockInvoke = invoke as MockedInvoke;

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("useProxmoxNodes", () => {
  it("loads nodes for a cluster and auto-selects the first node by name", async () => {
    mockInvoke.mockResolvedValue([
      { node: "vmhost2", status: "online" },
      { node: "vmhost1", status: "online" },
    ]);

    const { result } = renderHook(() => useProxmoxNodes("cluster-a"));

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.nodeNames).toEqual(["vmhost1", "vmhost2"]);
    // First node alphabetically is auto-selected so data can load without typing.
    expect(result.current.selectedNode).toBe("vmhost1");
    expect(result.current.error).toBeNull();
  });

  it("does not call the backend when clusterId is empty", async () => {
    renderHook(() => useProxmoxNodes(""));
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("captures an error message when node loading fails", async () => {
    mockInvoke.mockRejectedValue("boom");
    const { result } = renderHook(() => useProxmoxNodes("cluster-a"));
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.error).toContain("boom");
    expect(result.current.nodeNames).toEqual([]);
  });

  it("allows manually overriding the selected node", async () => {
    mockInvoke.mockResolvedValue([{ node: "a" }, { node: "b" }]);
    const { result } = renderHook(() => useProxmoxNodes("cluster-a"));
    await waitFor(() => expect(result.current.loading).toBe(false));
    act(() => result.current.setSelectedNode("b"));
    expect(result.current.selectedNode).toBe("b");
  });
});
