import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { confirm } from "@tauri-apps/plugin-dialog";
import { ProxmoxCephPage } from "@/pages/Proxmox/CephPage";
import { useProxmoxStore } from "@/stores/proxmoxStore";

vi.mock("@tauri-apps/api/core");
vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), info: vi.fn() },
  Toaster: () => null,
}));

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;
const mockConfirm = confirm as unknown as ReturnType<typeof vi.fn>;

const cluster = { id: "cluster-1", name: "TFTSR", clusterType: "ve" };
const node = { node: "vmhost1", status: "online" };

function setupInvoke() {
  mockInvoke.mockImplementation((cmd: string) => {
    switch (cmd) {
      case "list_proxmox_clusters":
        return Promise.resolve([cluster]);
      case "list_proxmox_nodes":
        return Promise.resolve([node]);
      case "get_ceph_health":
        return Promise.resolve({ status: "HEALTH_OK", summary: "all good", details: [] });
      case "list_ceph_pools":
        return Promise.resolve([]);
      case "list_ceph_osd":
        return Promise.resolve([]);
      case "list_ceph_monitors":
        return Promise.resolve([
          { name: "mon-a", quorum: true, address: "10.0.0.1:6789", version: "19.2.3" },
        ]);
      case "list_ceph_managers":
        return Promise.resolve([{ name: "mgr-a", addr: "10.0.0.2", state: "active" }]);
      case "list_cephfs":
        return Promise.resolve([
          { name: "cephfs", metadataPool: "cephfs_metadata", dataPool: "cephfs_data" },
        ]);
      case "get_ceph_flags":
        return Promise.resolve([
          { name: "noout", value: 0, description: "OSDs will not automatically be marked out." },
        ]);
      case "create_ceph_monitor":
      case "delete_ceph_monitor":
      case "create_ceph_manager":
      case "delete_ceph_manager":
      case "ceph_service_action":
        return Promise.resolve("UPID:vmhost1:00001234:aaa:ceph:root@pam:");
      case "set_ceph_flag":
      case "create_ceph_pool":
        return Promise.resolve(undefined);
      default:
        return Promise.resolve(undefined);
    }
  });
}

beforeEach(() => {
  mockInvoke.mockReset();
  mockConfirm.mockReset();
  mockConfirm.mockResolvedValue(false);
  localStorage.clear();
  useProxmoxStore.setState({ selectedClusterId: "", selectedNodeByCluster: {} });
  setupInvoke();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("CephPage — tab autoload", () => {
  it("autoloads Monitors without a manual Load click", async () => {
    render(<ProxmoxCephPage />);
    await waitFor(() => expect(screen.getByText("mon-a")).toBeInTheDocument());
    expect(mockInvoke).toHaveBeenCalledWith("list_ceph_monitors", {
      clusterId: "cluster-1",
      node: "vmhost1",
    });
  });

  it("autoloads Managers when the tab is selected, with no Load button present", async () => {
    const user = userEvent.setup();
    render(<ProxmoxCephPage />);
    await waitFor(() => expect(screen.getByRole("button", { name: "Managers" })).toBeInTheDocument());

    expect(screen.queryByRole("button", { name: /^load$/i })).toBeNull();

    await user.click(screen.getByRole("button", { name: "Managers" }));
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("list_ceph_managers", {
        clusterId: "cluster-1",
        node: "vmhost1",
      })
    );
  });

  it("autoloads CephFS and renders its data", async () => {
    const user = userEvent.setup();
    render(<ProxmoxCephPage />);
    await waitFor(() => expect(screen.getByRole("button", { name: "CephFS" })).toBeInTheDocument());
    await user.click(screen.getByRole("button", { name: "CephFS" }));

    await waitFor(() => expect(screen.getByText("cephfs")).toBeInTheDocument());
    expect(screen.getByText("cephfs_data")).toBeInTheDocument();
  });

  it("autoloads Flags", async () => {
    const user = userEvent.setup();
    render(<ProxmoxCephPage />);
    await waitFor(() => expect(screen.getByRole("button", { name: "Flags" })).toBeInTheDocument());
    await user.click(screen.getByRole("button", { name: "Flags" }));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_ceph_flags", {
        clusterId: "cluster-1",
        node: "vmhost1",
      })
    );
  });

  it("polls the active tab's data periodically", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    render(<ProxmoxCephPage />);

    await vi.waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("list_ceph_monitors", {
        clusterId: "cluster-1",
        node: "vmhost1",
      })
    );
    const callsBefore = mockInvoke.mock.calls.filter((c) => c[0] === "list_ceph_monitors").length;

    await vi.advanceTimersByTimeAsync(30_000);

    const callsAfter = mockInvoke.mock.calls.filter((c) => c[0] === "list_ceph_monitors").length;
    expect(callsAfter).toBeGreaterThan(callsBefore);
  });
});

describe("CephPage — monitor and manager actions", () => {
  it("stops a monitor via cephServiceAction", async () => {
    const user = userEvent.setup();
    render(<ProxmoxCephPage />);
    await waitFor(() => expect(screen.getByText("mon-a")).toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: /stop monitor mon-a/i }));
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("ceph_service_action", {
        clusterId: "cluster-1",
        node: "vmhost1",
        service: "mon.mon-a",
        action: "stop",
      })
    );
  });

  it("requires confirmation before destroying a monitor, and does not call delete when cancelled", async () => {
    const user = userEvent.setup();
    render(<ProxmoxCephPage />);
    await waitFor(() => expect(screen.getByText("mon-a")).toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: /destroy monitor mon-a/i }));
    expect(mockConfirm).toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalledWith("delete_ceph_monitor", expect.anything());
  });

  it("destroys a monitor after confirmation", async () => {
    mockConfirm.mockResolvedValue(true);
    const user = userEvent.setup();
    render(<ProxmoxCephPage />);
    await waitFor(() => expect(screen.getByText("mon-a")).toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: /destroy monitor mon-a/i }));
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("delete_ceph_monitor", {
        clusterId: "cluster-1",
        node: "vmhost1",
        monid: "mon-a",
      })
    );
  });

  it("creates a monitor via the Create Monitor dialog", async () => {
    const user = userEvent.setup();
    render(<ProxmoxCephPage />);
    await waitFor(() => expect(screen.getByText("mon-a")).toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: /create monitor/i }));
    const monidInput = await screen.findByLabelText(/monitor id/i);
    await user.type(monidInput, "mon-b");
    await user.click(screen.getByRole("button", { name: /^create$/i }));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("create_ceph_monitor", {
        clusterId: "cluster-1",
        node: "vmhost1",
        monid: "mon-b",
      })
    );
  });

  it("stops a manager via cephServiceAction", async () => {
    const user = userEvent.setup();
    render(<ProxmoxCephPage />);
    await waitFor(() => expect(screen.getByText("mon-a")).toBeInTheDocument());
    await user.click(screen.getByRole("button", { name: "Managers" }));
    await waitFor(() => expect(screen.getByText("mgr-a")).toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: /stop manager mgr-a/i }));
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("ceph_service_action", {
        clusterId: "cluster-1",
        node: "vmhost1",
        service: "mgr.mgr-a",
        action: "stop",
      })
    );
  });
});

describe("CephPage — flags editing", () => {
  it("toggles a flag via setCephFlag", async () => {
    const user = userEvent.setup();
    render(<ProxmoxCephPage />);
    await waitFor(() => expect(screen.getByRole("button", { name: "Flags" })).toBeInTheDocument());
    await user.click(screen.getByRole("button", { name: "Flags" }));

    const toggle = await screen.findByRole("checkbox", { name: /noout/i });
    await user.click(toggle);

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("set_ceph_flag", {
        clusterId: "cluster-1",
        flag: "noout",
        value: true,
      })
    );
  });
});

describe("CephPage — pool creation", () => {
  it("creates a pool via the New Pool dialog", async () => {
    const user = userEvent.setup();
    render(<ProxmoxCephPage />);
    await waitFor(() => expect(screen.getByRole("button", { name: /new pool/i })).toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: /new pool/i }));
    await user.type(await screen.findByLabelText(/pool name/i), "vm-storage");
    await user.click(screen.getByRole("button", { name: /^create$/i }));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("create_ceph_pool", {
        clusterId: "cluster-1",
        node: "vmhost1",
        pool: "vm-storage",
        pgNum: 128,
      })
    );
  });
});
