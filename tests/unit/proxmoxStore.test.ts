import { describe, it, expect, beforeEach } from "vitest";
import { useProxmoxStore } from "@/stores/proxmoxStore";

describe("useProxmoxStore", () => {
  beforeEach(() => {
    localStorage.clear();
    useProxmoxStore.setState({
      selectedClusterId: "",
      selectedNodeByCluster: {},
    });
  });

  it("starts with no selection", () => {
    const state = useProxmoxStore.getState();
    expect(state.selectedClusterId).toBe("");
    expect(state.selectedNodeByCluster).toEqual({});
  });

  it("stores the selected cluster id", () => {
    useProxmoxStore.getState().setSelectedClusterId("cluster-1");
    expect(useProxmoxStore.getState().selectedClusterId).toBe("cluster-1");
  });

  it("stores the selected node per cluster independently", () => {
    const { setSelectedNode } = useProxmoxStore.getState();
    setSelectedNode("cluster-1", "node-a");
    setSelectedNode("cluster-2", "node-b");

    const state = useProxmoxStore.getState();
    expect(state.selectedNodeByCluster["cluster-1"]).toBe("node-a");
    expect(state.selectedNodeByCluster["cluster-2"]).toBe("node-b");
  });

  it("getSelectedNode returns the stored node for a cluster or empty string", () => {
    const { setSelectedNode, getSelectedNode } = useProxmoxStore.getState();
    setSelectedNode("cluster-1", "node-a");
    expect(getSelectedNode("cluster-1")).toBe("node-a");
    expect(getSelectedNode("cluster-unknown")).toBe("");
  });

  it("persists selection to localStorage under tftsr-proxmox", () => {
    useProxmoxStore.getState().setSelectedClusterId("cluster-1");
    useProxmoxStore.getState().setSelectedNode("cluster-1", "node-a");

    const stored = localStorage.getItem("tftsr-proxmox");
    expect(stored).toBeTruthy();
    const parsed = JSON.parse(stored!);
    expect(parsed.state.selectedClusterId).toBe("cluster-1");
    expect(parsed.state.selectedNodeByCluster["cluster-1"]).toBe("node-a");
  });
});
