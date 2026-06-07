import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { RbacViewer } from "@/components/Kubernetes/RbacViewer";

type MockedInvoke = typeof invoke & {
  mockResolvedValue: (v: unknown) => void;
  mockRejectedValue: (e: Error) => void;
  mockImplementation: (fn: (cmd: string, args?: unknown) => Promise<unknown>) => void;
};

const mockInvoke = invoke as MockedInvoke;

const MOCK_ROLES = [
  { name: "pod-reader", namespace: "default", age: "5d" },
  { name: "secret-viewer", namespace: "default", age: "3d" },
];

const MOCK_CLUSTER_ROLES = [
  { name: "admin", age: "100d" },
  { name: "view", age: "100d" },
];

const MOCK_ROLE_BINDINGS = [
  { name: "pod-reader-binding", namespace: "default", role: "pod-reader", age: "4d" },
  { name: "view-binding", namespace: "default", role: "view", age: "2d" },
];

const MOCK_CLUSTER_ROLE_BINDINGS = [
  { name: "admin-binding", cluster_role: "admin", age: "100d" },
  { name: "view-global-binding", cluster_role: "view", age: "50d" },
];

describe("RbacViewer", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows loading state initially", () => {
    mockInvoke.mockImplementation(() => new Promise(() => {}));
    render(<RbacViewer clusterId="cluster-1" namespace="default" />);
    expect(screen.getByTestId("rbac-loading")).toBeInTheDocument();
  });

  it("renders roles from listRolesCmd response", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_roles") return Promise.resolve(MOCK_ROLES);
      if (cmd === "list_clusterroles") return Promise.resolve(MOCK_CLUSTER_ROLES);
      if (cmd === "list_rolebindings") return Promise.resolve(MOCK_ROLE_BINDINGS);
      if (cmd === "list_clusterrolebindings") return Promise.resolve(MOCK_CLUSTER_ROLE_BINDINGS);
      return Promise.resolve([]);
    });

    render(<RbacViewer clusterId="cluster-1" namespace="default" />);

    await waitFor(() => {
      expect(screen.getByText("pod-reader")).toBeInTheDocument();
      expect(screen.getByText("secret-viewer")).toBeInTheDocument();
    });
  });

  it("renders cluster roles from listClusterrolesCmd response", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_roles") return Promise.resolve(MOCK_ROLES);
      if (cmd === "list_clusterroles") return Promise.resolve(MOCK_CLUSTER_ROLES);
      if (cmd === "list_rolebindings") return Promise.resolve(MOCK_ROLE_BINDINGS);
      if (cmd === "list_clusterrolebindings") return Promise.resolve(MOCK_CLUSTER_ROLE_BINDINGS);
      return Promise.resolve([]);
    });

    render(<RbacViewer clusterId="cluster-1" namespace="default" />);

    // Navigate to ClusterRoles tab
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /clusterroles/i })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: /clusterroles/i }));

    await waitFor(() => {
      expect(screen.getByText("admin")).toBeInTheDocument();
      expect(screen.getByText("view")).toBeInTheDocument();
    });
  });

  it("renders role bindings from listRolebindingsCmd response", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_roles") return Promise.resolve(MOCK_ROLES);
      if (cmd === "list_clusterroles") return Promise.resolve(MOCK_CLUSTER_ROLES);
      if (cmd === "list_rolebindings") return Promise.resolve(MOCK_ROLE_BINDINGS);
      if (cmd === "list_clusterrolebindings") return Promise.resolve(MOCK_CLUSTER_ROLE_BINDINGS);
      return Promise.resolve([]);
    });

    render(<RbacViewer clusterId="cluster-1" namespace="default" />);

    // Navigate to RoleBindings tab
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "RoleBindings" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "RoleBindings" }));

    await waitFor(() => {
      expect(screen.getByText("pod-reader-binding")).toBeInTheDocument();
      expect(screen.getByText("view-binding")).toBeInTheDocument();
    });
  });

  it("renders cluster role bindings from listClusterrolebindingsCmd response", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_roles") return Promise.resolve(MOCK_ROLES);
      if (cmd === "list_clusterroles") return Promise.resolve(MOCK_CLUSTER_ROLES);
      if (cmd === "list_rolebindings") return Promise.resolve(MOCK_ROLE_BINDINGS);
      if (cmd === "list_clusterrolebindings") return Promise.resolve(MOCK_CLUSTER_ROLE_BINDINGS);
      return Promise.resolve([]);
    });

    render(<RbacViewer clusterId="cluster-1" namespace="default" />);

    // Navigate to ClusterRoleBindings tab
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /clusterrolebindings/i })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: /clusterrolebindings/i }));

    await waitFor(() => {
      expect(screen.getByText("admin-binding")).toBeInTheDocument();
      expect(screen.getByText("view-global-binding")).toBeInTheDocument();
    });
  });

  it("shows error state when fetch fails", async () => {
    mockInvoke.mockImplementation(() =>
      Promise.reject(new Error("Connection refused"))
    );

    render(<RbacViewer clusterId="cluster-1" namespace="default" />);

    await waitFor(() => {
      expect(screen.getByTestId("rbac-error")).toBeInTheDocument();
    });
  });

  it("shows retry button on error state", async () => {
    mockInvoke.mockImplementation(() =>
      Promise.reject(new Error("Connection refused"))
    );

    render(<RbacViewer clusterId="cluster-1" namespace="default" />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /retry/i })).toBeInTheDocument();
    });
  });

  it("Create Role button is present", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_roles") return Promise.resolve(MOCK_ROLES);
      if (cmd === "list_clusterroles") return Promise.resolve(MOCK_CLUSTER_ROLES);
      if (cmd === "list_rolebindings") return Promise.resolve(MOCK_ROLE_BINDINGS);
      if (cmd === "list_clusterrolebindings") return Promise.resolve(MOCK_CLUSTER_ROLE_BINDINGS);
      return Promise.resolve([]);
    });

    render(<RbacViewer clusterId="cluster-1" namespace="default" />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /create role/i })).toBeInTheDocument();
    });
  });

  it("Delete button calls deleteResourceCmd for a role", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_roles") return Promise.resolve(MOCK_ROLES);
      if (cmd === "list_clusterroles") return Promise.resolve(MOCK_CLUSTER_ROLES);
      if (cmd === "list_rolebindings") return Promise.resolve(MOCK_ROLE_BINDINGS);
      if (cmd === "list_clusterrolebindings") return Promise.resolve(MOCK_CLUSTER_ROLE_BINDINGS);
      if (cmd === "delete_resource") return Promise.resolve(undefined);
      return Promise.resolve([]);
    });

    render(<RbacViewer clusterId="cluster-1" namespace="default" />);

    await waitFor(() => {
      expect(screen.getByText("pod-reader")).toBeInTheDocument();
    });

    // Click the first Delete button in the Roles tab
    const deleteButtons = screen.getAllByRole("button", { name: /delete/i });
    fireEvent.click(deleteButtons[0]);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "cluster-1",
        resourceType: "roles",
        namespace: "default",
        resourceName: "pod-reader",
      });
    });
  });
});
