import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useProxmoxClusters } from "@/hooks/useProxmoxClusters";
import { useProxmoxStore } from "@/stores/proxmoxStore";

vi.mock("@tauri-apps/api/core");

type MockedInvoke = typeof invoke & {
  mockResolvedValue: (v: unknown) => void;
  mockRejectedValue: (e: unknown) => void;
  mockReset: () => void;
};
const mockInvoke = invoke as MockedInvoke;

const cluster = (id: string, clusterType: "ve" | "pbs" = "ve") => ({
  id,
  name: id,
  clusterType,
  url: "https://example.com",
  port: 8006,
  username: "root@pam",
  createdAt: "",
  updatedAt: "",
});

beforeEach(() => {
  mockInvoke.mockReset();
  localStorage.clear();
  useProxmoxStore.setState({ selectedClusterId: "", selectedNodeByCluster: {} });
});

describe("useProxmoxClusters", () => {
  it("loads clusters and selects the first one by default", async () => {
    mockInvoke.mockResolvedValue([cluster("cluster-a"), cluster("cluster-b")]);
    const { result } = renderHook(() => useProxmoxClusters());

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.clusters).toHaveLength(2);
    expect(result.current.selectedClusterId).toBe("cluster-a");
  });

  it("restores a previously persisted cluster selection", async () => {
    useProxmoxStore.getState().setSelectedClusterId("cluster-b");
    mockInvoke.mockResolvedValue([cluster("cluster-a"), cluster("cluster-b")]);

    const { result } = renderHook(() => useProxmoxClusters());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.selectedClusterId).toBe("cluster-b");
  });

  it("falls back to the first cluster when the persisted id no longer exists", async () => {
    useProxmoxStore.getState().setSelectedClusterId("gone");
    mockInvoke.mockResolvedValue([cluster("cluster-a"), cluster("cluster-b")]);

    const { result } = renderHook(() => useProxmoxClusters());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.selectedClusterId).toBe("cluster-a");
  });

  it("applies an optional filter (e.g. only PBS clusters)", async () => {
    mockInvoke.mockResolvedValue([cluster("ve-1", "ve"), cluster("pbs-1", "pbs")]);

    const { result } = renderHook(() => useProxmoxClusters((c) => c.clusterType === "pbs"));
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.clusters).toHaveLength(1);
    expect(result.current.selectedClusterId).toBe("pbs-1");
  });

  it("persists a manually selected cluster to the proxmox store", async () => {
    mockInvoke.mockResolvedValue([cluster("cluster-a"), cluster("cluster-b")]);
    const { result } = renderHook(() => useProxmoxClusters());
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.setSelectedClusterId("cluster-b"));
    expect(result.current.selectedClusterId).toBe("cluster-b");
    expect(useProxmoxStore.getState().selectedClusterId).toBe("cluster-b");
  });

  it("captures an error message when cluster loading fails", async () => {
    mockInvoke.mockRejectedValue("boom");
    const { result } = renderHook(() => useProxmoxClusters());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.error).toContain("boom");
    expect(result.current.clusters).toEqual([]);
  });

  it("persists the auto-selected default cluster, not just manual picks", async () => {
    mockInvoke.mockResolvedValue([cluster("cluster-a"), cluster("cluster-b")]);
    const { result } = renderHook(() => useProxmoxClusters());

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.selectedClusterId).toBe("cluster-a");
    // The implicit default (never explicitly picked by the user) must still
    // land in the shared store, or other pages won't see it.
    expect(useProxmoxStore.getState().selectedClusterId).toBe("cluster-a");
  });

  it("does not let a filtered subset's fallback overwrite the shared selection", async () => {
    // Some other (unfiltered) page already selected "ve-1" globally.
    useProxmoxStore.getState().setSelectedClusterId("ve-1");
    mockInvoke.mockResolvedValue([cluster("ve-1", "ve"), cluster("pbs-1", "pbs")]);

    const { result } = renderHook(() => useProxmoxClusters((c) => c.clusterType === "pbs"));
    await waitFor(() => expect(result.current.loading).toBe(false));

    // This PBS-only view falls back locally to pbs-1 (ve-1 isn't in its list)...
    expect(result.current.selectedClusterId).toBe("pbs-1");
    // ...but must not clobber the global selection an unfiltered page relies on.
    expect(useProxmoxStore.getState().selectedClusterId).toBe("ve-1");
  });
});
