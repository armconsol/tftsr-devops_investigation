import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ClusterDetails } from "@/components/Kubernetes/ClusterDetails";

type MockedInvoke = typeof invoke & {
  mockResolvedValue: (v: unknown) => void;
  mockRejectedValue: (e: Error) => void;
  mockImplementation: (fn: (cmd: string) => Promise<unknown>) => void;
};

const mockInvoke = invoke as MockedInvoke;

const mockKubeconfigs = [
  {
    id: "cluster-1",
    name: "production-k8s",
    context: "prod-context",
    cluster_url: "https://k8s.example.com:6443",
    is_active: true,
  },
  {
    id: "cluster-2",
    name: "staging-k8s",
    context: "staging-context",
    cluster_url: "https://staging.example.com:6443",
    is_active: false,
  },
];

const mockNodes = [
  {
    name: "node-1",
    status: "Ready",
    roles: "control-plane",
    version: "v1.28.4",
    internal_ip: "10.0.0.1",
    os_image: "Ubuntu 22.04",
    kernel_version: "5.15.0",
    kubelet_version: "v1.28.4",
    age: "30d",
  },
  {
    name: "node-2",
    status: "Ready",
    roles: "worker",
    version: "v1.28.4",
    internal_ip: "10.0.0.2",
    os_image: "Ubuntu 22.04",
    kernel_version: "5.15.0",
    kubelet_version: "v1.28.4",
    age: "30d",
  },
];

describe("ClusterDetails", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders cluster name from kubeconfig", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_kubeconfigs") return Promise.resolve(mockKubeconfigs);
      if (cmd === "list_nodes") return Promise.resolve(mockNodes);
      return Promise.resolve([]);
    });

    render(<ClusterDetails clusterId="cluster-1" />);

    await waitFor(() => {
      expect(screen.getByTestId("cluster-name")).toHaveTextContent("production-k8s");
    });
  });

  it("renders API server URL from kubeconfig", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_kubeconfigs") return Promise.resolve(mockKubeconfigs);
      if (cmd === "list_nodes") return Promise.resolve(mockNodes);
      return Promise.resolve([]);
    });

    render(<ClusterDetails clusterId="cluster-1" />);

    await waitFor(() => {
      expect(screen.getByTestId("cluster-api-server")).toHaveTextContent(
        "https://k8s.example.com:6443"
      );
    });
  });

  it("renders context name", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_kubeconfigs") return Promise.resolve(mockKubeconfigs);
      if (cmd === "list_nodes") return Promise.resolve(mockNodes);
      return Promise.resolve([]);
    });

    render(<ClusterDetails clusterId="cluster-1" />);

    await waitFor(() => {
      expect(screen.getByTestId("cluster-context")).toHaveTextContent("prod-context");
    });
  });

  it("shows node information from listNodesCmd", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_kubeconfigs") return Promise.resolve(mockKubeconfigs);
      if (cmd === "list_nodes") return Promise.resolve(mockNodes);
      return Promise.resolve([]);
    });

    render(<ClusterDetails clusterId="cluster-1" />);

    await waitFor(() => {
      expect(screen.getByText("node-1")).toBeInTheDocument();
      expect(screen.getByText("node-2")).toBeInTheDocument();
    });
  });

  it("shows 'No data' message when cluster info unavailable", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_kubeconfigs") return Promise.resolve([]);
      if (cmd === "list_nodes") return Promise.resolve([]);
      return Promise.resolve([]);
    });

    render(<ClusterDetails clusterId="cluster-unknown" />);

    await waitFor(() => {
      expect(screen.getByTestId("cluster-no-data")).toBeInTheDocument();
    });
  });

  it("shows context name in header instead of raw GUID", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_kubeconfigs") return Promise.resolve(mockKubeconfigs);
      if (cmd === "list_nodes") return Promise.resolve(mockNodes);
      return Promise.resolve([]);
    });

    render(<ClusterDetails clusterId="cluster-1" />);

    await waitFor(() => {
      expect(screen.getByTestId("cluster-context-header")).toHaveTextContent("prod-context");
      expect(screen.queryByText("cluster-1")).not.toBeInTheDocument();
    });
  });
});
