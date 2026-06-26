import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { VMList } from "@/components/Proxmox/VMList";

vi.mock("@tauri-apps/api/core");
vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
  },
  Toaster: () => null,
}));

type MockedInvoke = typeof invoke & {
  mockResolvedValue: (v: unknown) => void;
  mockRejectedValue: (e: unknown) => void;
  mockResolvedValueOnce: (v: unknown) => void;
};

const mockInvoke = invoke as MockedInvoke;

const stoppedVm = {
  id: 101,
  name: "nginx",
  node: "vmhost2",
  status: "stopped",
  cpu: 0,
  mem: 0,
  max_mem: 2 * 1024 * 1024 * 1024,
  disk: 0,
  max_disk: 20 * 1024 * 1024 * 1024,
  uptime: 0,
};

const runningVm = {
  id: 102,
  name: "docker-01",
  node: "vmhost2",
  status: "running",
  cpu: 0.35,
  mem: 512 * 1024 * 1024,
  max_mem: 2 * 1024 * 1024 * 1024,
  disk: 0,
  max_disk: 20 * 1024 * 1024 * 1024,
  uptime: 3600,
};

const pausedVm = {
  id: 103,
  name: "test-vm",
  node: "vmhost2",
  status: "paused",
  cpu: 0,
  mem: 0,
  max_mem: 1024 * 1024 * 1024,
  disk: 0,
  max_disk: 10 * 1024 * 1024 * 1024,
  uptime: 0,
};

const mockClusters = [{ id: "cluster-1", name: "TFTSR" }];

function renderVMList(vms = [stoppedVm], clusterId = "cluster-1") {
  const onRefresh = vi.fn();
  return {
    onRefresh,
    ...render(
      <MemoryRouter>
        <VMList
          vms={vms}
          clusterId={clusterId}
          clusters={mockClusters as never}
          onRefresh={onRefresh}
        />
      </MemoryRouter>
    ),
  };
}

describe("VMList — column rendering", () => {
  it("renders VM name, VMID, node, status, CPU, memory and uptime columns", () => {
    renderVMList([stoppedVm]);
    expect(screen.getByText("Name")).toBeDefined();
    expect(screen.getByText("VM ID")).toBeDefined();
    expect(screen.getByText("Node")).toBeDefined();
    expect(screen.getByText("Status")).toBeDefined();
    expect(screen.getByText("CPU")).toBeDefined();
    expect(screen.getByText("Memory")).toBeDefined();
    expect(screen.getByText("Uptime")).toBeDefined();
  });

  it("does NOT render the Disk column", () => {
    renderVMList([stoppedVm]);
    expect(screen.queryByText("Disk")).toBeNull();
  });

  it("displays VM name in the list", () => {
    renderVMList([stoppedVm]);
    expect(screen.getByText("nginx")).toBeDefined();
  });

  it("displays the correct VMID", () => {
    renderVMList([stoppedVm]);
    expect(screen.getByText("101")).toBeDefined();
  });

  it("displays status badge for stopped VM", () => {
    renderVMList([stoppedVm]);
    expect(screen.getByText("stopped")).toBeDefined();
  });

  it("displays status badge for running VM", () => {
    renderVMList([runningVm]);
    expect(screen.getByText("running")).toBeDefined();
  });
});

describe("VMList — action menu for stopped VM", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
  });

  it("shows Start action for stopped VM", async () => {
    renderVMList([stoppedVm]);
    const menuBtn = screen.getAllByRole("button").find(
      (b) => b.querySelector("svg")
    );
    fireEvent.click(menuBtn!);
    expect(screen.getByText("Start")).toBeDefined();
  });

  it("does NOT show Stop/Reboot/Shutdown for stopped VM", async () => {
    renderVMList([stoppedVm]);
    const menuBtn = screen.getAllByRole("button").find(
      (b) => b.querySelector("svg")
    );
    fireEvent.click(menuBtn!);
    expect(screen.queryByText("Stop")).toBeNull();
    expect(screen.queryByText("Reboot")).toBeNull();
    expect(screen.queryByText("Shutdown")).toBeNull();
  });

  it("calls start_proxmox_vm when Start is clicked", async () => {
    renderVMList([stoppedVm]);
    const menuBtn = screen.getAllByRole("button").find(
      (b) => b.querySelector("svg")
    );
    fireEvent.click(menuBtn!);
    fireEvent.click(screen.getByText("Start"));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("start_proxmox_vm", {
        clusterId: "cluster-1",
        nodeId: "vmhost2",
        vmId: 101,
      });
    });
  });
});

describe("VMList — action menu for running VM", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
  });

  it("shows Stop, Reboot, Shutdown, Suspend actions for running VM", async () => {
    renderVMList([runningVm]);
    const menuBtn = screen.getAllByRole("button").find(
      (b) => b.querySelector("svg")
    );
    fireEvent.click(menuBtn!);
    expect(screen.getByText("Stop")).toBeDefined();
    expect(screen.getByText("Reboot")).toBeDefined();
    expect(screen.getByText("Shutdown")).toBeDefined();
    expect(screen.getByText("Suspend")).toBeDefined();
  });

  it("calls stop_proxmox_vm when Stop is clicked", async () => {
    renderVMList([runningVm]);
    const menuBtn = screen.getAllByRole("button").find(
      (b) => b.querySelector("svg")
    );
    fireEvent.click(menuBtn!);
    fireEvent.click(screen.getByText("Stop"));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("stop_proxmox_vm", {
        clusterId: "cluster-1",
        nodeId: "vmhost2",
        vmId: 102,
      });
    });
  });

  it("calls reboot_proxmox_vm when Reboot is clicked", async () => {
    renderVMList([runningVm]);
    const menuBtn = screen.getAllByRole("button").find(
      (b) => b.querySelector("svg")
    );
    fireEvent.click(menuBtn!);
    fireEvent.click(screen.getByText("Reboot"));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("reboot_proxmox_vm", {
        clusterId: "cluster-1",
        nodeId: "vmhost2",
        vmId: 102,
      });
    });
  });

  it("calls shutdown_proxmox_vm when Shutdown is clicked", async () => {
    renderVMList([runningVm]);
    const menuBtn = screen.getAllByRole("button").find(
      (b) => b.querySelector("svg")
    );
    fireEvent.click(menuBtn!);
    fireEvent.click(screen.getByText("Shutdown"));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("shutdown_proxmox_vm", {
        clusterId: "cluster-1",
        nodeId: "vmhost2",
        vmId: 102,
      });
    });
  });

  it("calls suspend_proxmox_vm when Suspend is clicked", async () => {
    renderVMList([runningVm]);
    const menuBtn = screen.getAllByRole("button").find(
      (b) => b.querySelector("svg")
    );
    fireEvent.click(menuBtn!);
    fireEvent.click(screen.getByText("Suspend"));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("suspend_proxmox_vm", {
        clusterId: "cluster-1",
        nodeId: "vmhost2",
        vmId: 102,
      });
    });
  });
});

describe("VMList — action menu for paused VM", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
  });

  it("shows Resume action for paused VM", async () => {
    renderVMList([pausedVm]);
    const menuBtn = screen.getAllByRole("button").find(
      (b) => b.querySelector("svg")
    );
    fireEvent.click(menuBtn!);
    expect(screen.getByText("Resume")).toBeDefined();
  });

  it("calls resume_proxmox_vm when Resume is clicked", async () => {
    renderVMList([pausedVm]);
    const menuBtn = screen.getAllByRole("button").find(
      (b) => b.querySelector("svg")
    );
    fireEvent.click(menuBtn!);
    fireEvent.click(screen.getByText("Resume"));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("resume_proxmox_vm", {
        clusterId: "cluster-1",
        nodeId: "vmhost2",
        vmId: 103,
      });
    });
  });
});

describe("VMList — migrate action", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue([
      { node: "vmhost1", status: "online" },
      { node: "vmhost3", status: "online" },
    ]);
  });

  it("shows Migrate option in action menu", async () => {
    renderVMList([runningVm]);
    const menuBtn = screen.getAllByRole("button").find(
      (b) => b.querySelector("svg")
    );
    fireEvent.click(menuBtn!);
    expect(screen.getByText("Migrate")).toBeDefined();
  });

  it("opens migration dialog when Migrate is clicked", async () => {
    renderVMList([runningVm]);
    const menuBtn = screen.getAllByRole("button").find(
      (b) => b.querySelector("svg")
    );
    fireEvent.click(menuBtn!);
    fireEvent.click(screen.getByText("Migrate"));
    await waitFor(() => {
      expect(screen.getByText(/Migrate docker-01/i)).toBeDefined();
    });
  });
});

describe("VMList — empty state", () => {
  it("renders empty table body with no VMs", () => {
    renderVMList([]);
    expect(screen.getByText("Virtual Machines")).toBeDefined();
  });
});
