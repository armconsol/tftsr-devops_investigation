import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { ProxmoxVMsPage } from "@/pages/Proxmox/VMsPage";
import { ProxmoxCephPage } from "@/pages/Proxmox/CephPage";
import { useProxmoxStore } from "@/stores/proxmoxStore";

vi.mock("@tauri-apps/api/core");
vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), info: vi.fn() },
  Toaster: () => null,
}));

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

const clusterA = { id: "cluster-a", name: "Datacenter A", clusterType: "ve" };
const clusterB = { id: "cluster-b", name: "Datacenter B", clusterType: "ve" };

function setupInvoke() {
  mockInvoke.mockImplementation((cmd: string) => {
    switch (cmd) {
      case "list_proxmox_clusters":
        return Promise.resolve([clusterA, clusterB]);
      case "list_proxmox_nodes":
        return Promise.resolve([{ node: "vmhost1" }, { node: "vmhost2" }]);
      case "list_proxmox_vms":
        return Promise.resolve([]);
      case "get_ceph_health":
        return Promise.resolve({ status: "HEALTH_OK", summary: "ok", details: [] });
      case "list_ceph_pools":
      case "list_ceph_osd":
      case "list_ceph_monitors":
        return Promise.resolve([]);
      default:
        return Promise.resolve(undefined);
    }
  });
}

beforeEach(() => {
  mockInvoke.mockReset();
  localStorage.clear();
  useProxmoxStore.setState({ selectedClusterId: "", selectedNodeByCluster: {} });
  setupInvoke();
});

describe("Proxmox cluster/node selection persists across pages", () => {
  it("keeps the cluster selected on VMsPage when CephPage mounts afterwards", async () => {
    const user = userEvent.setup();
    const { unmount } = render(
      <MemoryRouter>
        <ProxmoxVMsPage />
      </MemoryRouter>
    );

    await waitFor(() => expect(screen.getByText("Datacenter A")).toBeInTheDocument());
    const select = screen.getByDisplayValue("Datacenter A");
    await user.selectOptions(select, "cluster-b");

    await waitFor(() =>
      expect(useProxmoxStore.getState().selectedClusterId).toBe("cluster-b")
    );

    unmount();
    cleanup();

    render(
      <MemoryRouter>
        <ProxmoxCephPage />
      </MemoryRouter>
    );

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith(
        "get_ceph_health",
        expect.objectContaining({ clusterId: "cluster-b" })
      )
    );
  });

  it("keeps the node selected for a cluster when navigating between node-scoped pages", async () => {
    const user = userEvent.setup();
    const { unmount } = render(
      <MemoryRouter>
        <ProxmoxCephPage />
      </MemoryRouter>
    );

    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("list_proxmox_nodes", { clusterId: "cluster-a" }));
    const nodeTrigger = await screen.findByText("vmhost1");
    await user.click(nodeTrigger);
    await user.click(screen.getByText("vmhost2"));

    await waitFor(() =>
      expect(useProxmoxStore.getState().selectedNodeByCluster["cluster-a"]).toBe("vmhost2")
    );

    unmount();
    cleanup();
    mockInvoke.mockClear();
    setupInvoke();

    render(
      <MemoryRouter>
        <ProxmoxCephPage />
      </MemoryRouter>
    );

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_ceph_health", { clusterId: "cluster-a", node: "vmhost2" })
    );
  });
});
