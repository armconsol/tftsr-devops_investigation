import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ClusterOverview } from "@/components/Kubernetes/ClusterOverview";

type MockedInvoke = typeof invoke & {
  mockResolvedValue: (v: unknown) => void;
  mockRejectedValue: (e: Error) => void;
  mockImplementation: (fn: (cmd: string) => Promise<unknown>) => void;
};

const mockInvoke = invoke as MockedInvoke;

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
  {
    name: "node-3",
    status: "NotReady",
    roles: "worker",
    version: "v1.28.4",
    internal_ip: "10.0.0.3",
    os_image: "Ubuntu 22.04",
    kernel_version: "5.15.0",
    kubelet_version: "v1.28.4",
    age: "1d",
  },
];

const mockPods = [
  { name: "nginx-1", status: "Running", ready: "1/1", age: "2d", containers: ["nginx"] },
  { name: "nginx-2", status: "Running", ready: "1/1", age: "2d", containers: ["nginx"] },
  { name: "crash-loop", status: "CrashLoopBackOff", ready: "0/1", age: "1h", containers: ["app"] },
];

const mockDeployments = [
  { name: "nginx", namespace: "default", ready: "2/2", up_to_date: "2", available: "2", age: "2d", replicas: 2, labels: {} },
  { name: "api", namespace: "kube-system", ready: "1/1", up_to_date: "1", available: "1", age: "5d", replicas: 1, labels: {} },
];

const mockNamespaces = [
  { name: "default", status: "Active", age: "30d" },
  { name: "kube-system", status: "Active", age: "30d" },
  { name: "monitoring", status: "Active", age: "10d" },
];

describe("ClusterOverview", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows loading spinner initially", () => {
    mockInvoke.mockImplementation(() => new Promise(() => {}));
    render(<ClusterOverview clusterId="cluster-1" />);
    expect(screen.getByTestId("overview-loading")).toBeInTheDocument();
  });

  it("renders node count from listNodesCmd response", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_nodes") return Promise.resolve(mockNodes);
      if (cmd === "list_pods") return Promise.resolve(mockPods);
      if (cmd === "list_deployments") return Promise.resolve(mockDeployments);
      if (cmd === "list_namespaces") return Promise.resolve(mockNamespaces);
      return Promise.resolve([]);
    });

    render(<ClusterOverview clusterId="cluster-1" />);

    await waitFor(() => {
      expect(screen.getByTestId("node-count")).toHaveTextContent("3");
    });
  });

  it("renders pod count from listPodsCmd response", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_nodes") return Promise.resolve(mockNodes);
      if (cmd === "list_pods") return Promise.resolve(mockPods);
      if (cmd === "list_deployments") return Promise.resolve(mockDeployments);
      if (cmd === "list_namespaces") return Promise.resolve(mockNamespaces);
      return Promise.resolve([]);
    });

    render(<ClusterOverview clusterId="cluster-1" />);

    await waitFor(() => {
      expect(screen.getByTestId("pod-count")).toHaveTextContent("3");
    });
  });

  it("renders namespace count from listNamespacesCmd response", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_nodes") return Promise.resolve(mockNodes);
      if (cmd === "list_pods") return Promise.resolve(mockPods);
      if (cmd === "list_deployments") return Promise.resolve(mockDeployments);
      if (cmd === "list_namespaces") return Promise.resolve(mockNamespaces);
      return Promise.resolve([]);
    });

    render(<ClusterOverview clusterId="cluster-1" />);

    await waitFor(() => {
      expect(screen.getByTestId("namespace-count")).toHaveTextContent("3");
    });
  });

  it("shows deployment count from listDeploymentsCmd response", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_nodes") return Promise.resolve(mockNodes);
      if (cmd === "list_pods") return Promise.resolve(mockPods);
      if (cmd === "list_deployments") return Promise.resolve(mockDeployments);
      if (cmd === "list_namespaces") return Promise.resolve(mockNamespaces);
      return Promise.resolve([]);
    });

    render(<ClusterOverview clusterId="cluster-1" />);

    await waitFor(() => {
      expect(screen.getByTestId("deployment-count")).toHaveTextContent("2");
    });
  });

  it("shows error state when IPC fails", async () => {
    mockInvoke.mockImplementation(() =>
      Promise.reject(new Error("Connection refused"))
    );

    render(<ClusterOverview clusterId="cluster-1" />);

    await waitFor(() => {
      expect(screen.getByTestId("overview-error")).toBeInTheDocument();
    });
  });

  it("shows Ready: X/Y node status", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_nodes") return Promise.resolve(mockNodes);
      if (cmd === "list_pods") return Promise.resolve(mockPods);
      if (cmd === "list_deployments") return Promise.resolve(mockDeployments);
      if (cmd === "list_namespaces") return Promise.resolve(mockNamespaces);
      return Promise.resolve([]);
    });

    render(<ClusterOverview clusterId="cluster-1" />);

    await waitFor(() => {
      expect(screen.getByTestId("node-ready-status")).toHaveTextContent("Ready: 2/3");
    });
  });

  it("displays clusterName prop in header instead of raw GUID", async () => {
    mockInvoke.mockImplementation(() => Promise.resolve([]));

    render(<ClusterOverview clusterId="019e9ff0-b6a4-78e1-a566-7a0c05e32577" clusterName="devops1-mgmt" />);

    await waitFor(() => {
      expect(screen.getByTestId("cluster-name-header")).toHaveTextContent("devops1-mgmt");
      expect(screen.queryByText("019e9ff0-b6a4-78e1-a566-7a0c05e32577")).not.toBeInTheDocument();
    });
  });

  it("falls back gracefully when clusterName prop is not provided", async () => {
    mockInvoke.mockImplementation(() => Promise.resolve([]));

    render(<ClusterOverview clusterId="cluster-1" />);

    await waitFor(() => {
      const header = screen.getByTestId("cluster-name-header");
      expect(header).toBeInTheDocument();
    });
  });
});
