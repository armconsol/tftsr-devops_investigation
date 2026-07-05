import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { confirm } from "@tauri-apps/plugin-dialog";
import { ProxmoxAdminPage } from "@/pages/Proxmox/AdminPage";
import { useProxmoxStore } from "@/stores/proxmoxStore";

vi.mock("@tauri-apps/api/core");
vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), info: vi.fn() },
  Toaster: () => null,
}));

const navigateMock = vi.fn();
vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual<typeof import("react-router-dom")>("react-router-dom");
  return { ...actual, useNavigate: () => navigateMock };
});

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
      case "get_node_status":
        return Promise.resolve({ cpu: 0.1, memory: {}, swap: {}, disk: {}, uptime: 100 });
      case "list_apt_updates":
        return Promise.resolve([
          { package: "curl", version: "7.0", newVersion: "7.1", description: "" },
        ]);
      case "refresh_apt_cache":
        return Promise.resolve("UPID:vmhost1:00001111:aaa:aptupdate:root@pam:");
      default:
        return Promise.resolve(undefined);
    }
  });
}

beforeEach(() => {
  mockInvoke.mockReset();
  mockConfirm.mockReset();
  mockConfirm.mockResolvedValue(false);
  navigateMock.mockReset();
  localStorage.clear();
  useProxmoxStore.setState({ selectedClusterId: "", selectedNodeByCluster: {} });
  setupInvoke();
});

function renderAdmin() {
  return render(
    <MemoryRouter>
      <ProxmoxAdminPage />
    </MemoryRouter>
  );
}

describe("AdminPage — Updates tab actions", () => {
  it("renders a Refresh APT Cache button that calls refresh_apt_cache", async () => {
    const user = userEvent.setup();
    renderAdmin();
    await user.click(screen.getByRole("button", { name: "Updates" }));
    await waitFor(() => expect(screen.getByText("curl")).toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: /refresh apt cache/i }));
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("refresh_apt_cache", {
        clusterId: "cluster-1",
        node: "vmhost1",
      })
    );
  });

  it("does not navigate to the upgrade shell without confirmation", async () => {
    const user = userEvent.setup();
    renderAdmin();
    await user.click(screen.getByRole("button", { name: "Updates" }));
    await waitFor(() => expect(screen.getByText("curl")).toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: /upgrade node/i }));
    expect(mockConfirm).toHaveBeenCalled();
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it("navigates to the node shell with ?cmd=upgrade after confirmation", async () => {
    mockConfirm.mockResolvedValue(true);
    const user = userEvent.setup();
    renderAdmin();
    await user.click(screen.getByRole("button", { name: "Updates" }));
    await waitFor(() => expect(screen.getByText("curl")).toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: /upgrade node/i }));
    await waitFor(() =>
      expect(navigateMock).toHaveBeenCalledWith("/proxmox/shell/cluster-1/vmhost1?cmd=upgrade")
    );
  });
});
