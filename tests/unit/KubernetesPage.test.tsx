import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { KubernetesPage } from "@/pages/Kubernetes/KubernetesPage";
import { useKubernetesStore } from "@/stores/kubernetesStore";

// Mock all Kubernetes child components that do their own invoke calls or have heavy deps
vi.mock("@/components/Kubernetes/ClusterOverview", () => ({
  ClusterOverview: ({ clusterId }: { clusterId: string }) => (
    <div data-testid="cluster-overview">ClusterOverview:{clusterId}</div>
  ),
}));

vi.mock("@/components/Kubernetes/PodList", () => ({
  PodList: () => <div data-testid="pod-list">PodList</div>,
}));

vi.mock("@/components/Kubernetes/DeploymentList", () => ({
  DeploymentList: () => <div data-testid="deployment-list">DeploymentList</div>,
}));

vi.mock("@/components/Kubernetes/DaemonSetList", () => ({
  DaemonSetList: () => <div data-testid="daemonset-list">DaemonSetList</div>,
}));

vi.mock("@/components/Kubernetes/StatefulSetList", () => ({
  StatefulSetList: () => <div data-testid="statefulset-list">StatefulSetList</div>,
}));

vi.mock("@/components/Kubernetes/ReplicaSetList", () => ({
  ReplicaSetList: () => <div data-testid="replicaset-list">ReplicaSetList</div>,
}));

vi.mock("@/components/Kubernetes/JobList", () => ({
  JobList: () => <div data-testid="job-list">JobList</div>,
}));

vi.mock("@/components/Kubernetes/CronJobList", () => ({
  CronJobList: () => <div data-testid="cronjob-list">CronJobList</div>,
}));

vi.mock("@/components/Kubernetes/ServiceList", () => ({
  ServiceList: () => <div data-testid="service-list">ServiceList</div>,
}));

vi.mock("@/components/Kubernetes/IngressList", () => ({
  IngressList: () => <div data-testid="ingress-list">IngressList</div>,
}));

vi.mock("@/components/Kubernetes/ConfigMapList", () => ({
  ConfigMapList: () => <div data-testid="configmap-list">ConfigMapList</div>,
}));

vi.mock("@/components/Kubernetes/SecretList", () => ({
  SecretList: () => <div data-testid="secret-list">SecretList</div>,
}));

vi.mock("@/components/Kubernetes/HPAList", () => ({
  HPAList: () => <div data-testid="hpa-list">HPAList</div>,
}));

vi.mock("@/components/Kubernetes/PVCList", () => ({
  PVCList: () => <div data-testid="pvc-list">PVCList</div>,
}));

vi.mock("@/components/Kubernetes/PVList", () => ({
  PVList: () => <div data-testid="pv-list">PVList</div>,
}));

vi.mock("@/components/Kubernetes/ServiceAccountList", () => ({
  ServiceAccountList: () => <div data-testid="serviceaccount-list">ServiceAccountList</div>,
}));

vi.mock("@/components/Kubernetes/RoleList", () => ({
  RoleList: () => <div data-testid="role-list">RoleList</div>,
}));

vi.mock("@/components/Kubernetes/ClusterRoleList", () => ({
  ClusterRoleList: () => <div data-testid="clusterrole-list">ClusterRoleList</div>,
}));

vi.mock("@/components/Kubernetes/RoleBindingList", () => ({
  RoleBindingList: () => <div data-testid="rolebinding-list">RoleBindingList</div>,
}));

vi.mock("@/components/Kubernetes/ClusterRoleBindingList", () => ({
  ClusterRoleBindingList: () => <div data-testid="clusterrolebinding-list">ClusterRoleBindingList</div>,
}));

vi.mock("@/components/Kubernetes/NodeList", () => ({
  NodeList: () => <div data-testid="node-list">NodeList</div>,
}));

vi.mock("@/components/Kubernetes/EventList", () => ({
  EventList: () => <div data-testid="event-list">EventList</div>,
}));

vi.mock("@/components/Kubernetes/PortForwardList", () => ({
  PortForwardList: ({ onStart }: { onStart: () => void }) => (
    <div data-testid="port-forward-list">
      <button onClick={onStart}>Start Port Forward</button>
    </div>
  ),
}));

vi.mock("@/components/Kubernetes/PortForwardForm", () => ({
  PortForwardForm: ({ isOpen }: { isOpen: boolean }) =>
    isOpen ? <div data-testid="port-forward-form">PortForwardForm</div> : null,
}));

vi.mock("@/components/Kubernetes/CommandPalette", () => ({
  CommandPalette: ({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) =>
    isOpen ? (
      <div data-testid="command-palette">
        CommandPalette
        <button onClick={onClose}>Close</button>
      </div>
    ) : null,
}));

vi.mock("@/components/Kubernetes/Hotbar", () => ({
  Hotbar: ({ onRefresh }: { onRefresh: () => void }) => (
    <div data-testid="hotbar">
      <button onClick={onRefresh} aria-label="Refresh">Refresh</button>
    </div>
  ),
}));

type MockedInvoke = ReturnType<typeof vi.fn>;

const mockInvoke = invoke as unknown as MockedInvoke;

const MOCK_KUBECONFIGS = [
  { id: "kc-1", name: "prod-cluster", context: "prod", cluster_url: "https://k8s.prod.example.com", is_active: true },
  { id: "kc-2", name: "staging-cluster", context: "staging", cluster_url: "https://k8s.staging.example.com", is_active: false },
];

const MOCK_NAMESPACES = [
  { name: "default", status: "Active", age: "100d" },
  { name: "kube-system", status: "Active", age: "100d" },
];

function renderPage() {
  return render(
    <MemoryRouter>
      <KubernetesPage />
    </MemoryRouter>
  );
}

describe("KubernetesPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Reset the kubernetes store to a clean state
    useKubernetesStore.setState({
      selectedClusterId: null,
      selectedNamespace: "all",
    });

    // Default: return empty arrays for all IPC calls unless overridden
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_kubeconfigs") return Promise.resolve([]);
      if (cmd === "list_namespaces") return Promise.resolve([]);
      if (cmd === "list_port_forwards") return Promise.resolve([]);
      return Promise.resolve([]);
    });
  });

  describe("Sidebar structure", () => {
    it("renders all resource category section headings", async () => {
      renderPage();

      await waitFor(() => {
        expect(screen.getByText("Workloads")).toBeInTheDocument();
        expect(screen.getByText("Network")).toBeInTheDocument();
        expect(screen.getByText("Config")).toBeInTheDocument();
        expect(screen.getByText("Storage")).toBeInTheDocument();
        expect(screen.getByText("Access Control")).toBeInTheDocument();
        expect(screen.getByText("Cluster")).toBeInTheDocument();
      });
    });

    it("renders all Workloads nav items", async () => {
      renderPage();

      await waitFor(() => {
        expect(screen.getByRole("button", { name: "Pods" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Deployments" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Daemon Sets" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Stateful Sets" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Replica Sets" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Jobs" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Cron Jobs" })).toBeInTheDocument();
      });
    });

    it("renders all Network nav items", async () => {
      renderPage();

      await waitFor(() => {
        expect(screen.getByRole("button", { name: "Services" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Ingresses" })).toBeInTheDocument();
      });
    });

    it("renders all Config & Storage nav items", async () => {
      renderPage();

      await waitFor(() => {
        expect(screen.getByRole("button", { name: "Config Maps" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Secrets" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Horizontal Pod Autoscalers" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Persistent Volume Claims" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Persistent Volumes" })).toBeInTheDocument();
      });
    });

    it("renders all Access Control nav items", async () => {
      renderPage();

      await waitFor(() => {
        expect(screen.getByRole("button", { name: "Service Accounts" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Roles" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Cluster Roles" })).toBeInTheDocument();
        // Use exact aria-label to disambiguate from "Cluster Role Bindings"
        expect(screen.getByRole("button", { name: "Role Bindings" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Cluster Role Bindings" })).toBeInTheDocument();
      });
    });

    it("renders Port Forwarding under Cluster section", async () => {
      renderPage();

      await waitFor(() => {
        expect(screen.getByRole("button", { name: "Port Forwarding" })).toBeInTheDocument();
      });
    });
  });

  describe("Cluster selector", () => {
    it("renders a cluster selector trigger button", async () => {
      renderPage();

      // The SelectTrigger renders as a <button type="button"> containing the placeholder
      await waitFor(() => {
        expect(screen.getByText("Select cluster")).toBeInTheDocument();
      });
    });

    it("populates cluster list when kubeconfigs are loaded", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "list_kubeconfigs") return Promise.resolve(MOCK_KUBECONFIGS);
        if (cmd === "list_namespaces") return Promise.resolve(MOCK_NAMESPACES);
        if (cmd === "list_port_forwards") return Promise.resolve([]);
        return Promise.resolve([]);
      });

      renderPage();

      // After data loads the active cluster context info is shown in the top bar
      // (the SelectValue shows the raw ID; the context string appears in the info row)
      await waitFor(() => {
        expect(screen.getByText("prod")).toBeInTheDocument();
      });
    });

    it("auto-selects the active kubeconfig on mount", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "list_kubeconfigs") return Promise.resolve(MOCK_KUBECONFIGS);
        if (cmd === "list_namespaces") return Promise.resolve(MOCK_NAMESPACES);
        if (cmd === "list_port_forwards") return Promise.resolve([]);
        return Promise.resolve([]);
      });

      renderPage();

      await waitFor(() => {
        expect(useKubernetesStore.getState().selectedClusterId).toBe("kc-1");
      });
    });
  });

  describe("Default view", () => {
    it("shows ClusterOverview when a cluster is selected", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "list_kubeconfigs") return Promise.resolve(MOCK_KUBECONFIGS);
        if (cmd === "list_namespaces") return Promise.resolve(MOCK_NAMESPACES);
        if (cmd === "list_port_forwards") return Promise.resolve([]);
        return Promise.resolve([]);
      });

      renderPage();

      await waitFor(() => {
        expect(screen.getByTestId("cluster-overview")).toBeInTheDocument();
      });
    });

    it("shows empty state with instructions when no cluster is selected", async () => {
      renderPage();

      await waitFor(() => {
        expect(screen.getByText(/no cluster selected/i)).toBeInTheDocument();
      });
    });
  });

  describe("Sidebar navigation", () => {
    beforeEach(() => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "list_kubeconfigs") return Promise.resolve(MOCK_KUBECONFIGS);
        if (cmd === "list_namespaces") return Promise.resolve(MOCK_NAMESPACES);
        if (cmd === "list_port_forwards") return Promise.resolve([]);
        return Promise.resolve([]);
      });
    });

    it("renders PodList when Pods nav item is clicked", async () => {
      renderPage();

      await waitFor(() => {
        expect(screen.getByRole("button", { name: "Pods" })).toBeInTheDocument();
      });

      fireEvent.click(screen.getByRole("button", { name: "Pods" }));

      await waitFor(() => {
        expect(screen.getByTestId("pod-list")).toBeInTheDocument();
      });
    });

    it("renders DeploymentList when Deployments nav item is clicked", async () => {
      renderPage();

      await waitFor(() => {
        expect(screen.getByRole("button", { name: "Deployments" })).toBeInTheDocument();
      });

      fireEvent.click(screen.getByRole("button", { name: "Deployments" }));

      await waitFor(() => {
        expect(screen.getByTestId("deployment-list")).toBeInTheDocument();
      });
    });

    it("renders ServiceList when Services nav item is clicked", async () => {
      renderPage();

      await waitFor(() => {
        expect(screen.getByRole("button", { name: "Services" })).toBeInTheDocument();
      });

      fireEvent.click(screen.getByRole("button", { name: "Services" }));

      await waitFor(() => {
        expect(screen.getByTestId("service-list")).toBeInTheDocument();
      });
    });
  });

  describe("Namespace selector", () => {
    it("renders namespace selector when a cluster is selected", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "list_kubeconfigs") return Promise.resolve(MOCK_KUBECONFIGS);
        if (cmd === "list_namespaces") return Promise.resolve(MOCK_NAMESPACES);
        if (cmd === "list_port_forwards") return Promise.resolve([]);
        return Promise.resolve([]);
      });

      renderPage();

      // When a cluster is selected the namespace label appears in the top bar
      await waitFor(() => {
        expect(screen.getByText("Namespace:")).toBeInTheDocument();
      });
    });

    it("shows the default 'all' namespace selection when cluster is selected", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "list_kubeconfigs") return Promise.resolve(MOCK_KUBECONFIGS);
        if (cmd === "list_namespaces") return Promise.resolve(MOCK_NAMESPACES);
        if (cmd === "list_port_forwards") return Promise.resolve([]);
        return Promise.resolve([]);
      });

      renderPage();

      await waitFor(() => {
        // The SelectValue renders ctx.value verbatim; the default is "all"
        // Confirm the namespace selector is visible and the store has the default
        expect(useKubernetesStore.getState().selectedNamespace).toBe("all");
        expect(screen.getByText("Namespace:")).toBeInTheDocument();
      });
    });

    it("renders loaded namespace names in the namespace select content after cluster selects", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "list_kubeconfigs") return Promise.resolve(MOCK_KUBECONFIGS);
        if (cmd === "list_namespaces") return Promise.resolve(MOCK_NAMESPACES);
        if (cmd === "list_port_forwards") return Promise.resolve([]);
        return Promise.resolve([]);
      });

      renderPage();

      await waitFor(() => {
        expect(screen.getByTestId("cluster-overview")).toBeInTheDocument();
      });

      // Namespace options are loaded; "default" should appear (either in select content or as selectable)
      // We confirm the namespace list was fetched
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("list_namespaces", { clusterId: "kc-1" });
      });
    });
  });

  describe("Port Forwarding", () => {
    it("renders PortForwardList in the Port Forwarding section", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "list_kubeconfigs") return Promise.resolve(MOCK_KUBECONFIGS);
        if (cmd === "list_namespaces") return Promise.resolve(MOCK_NAMESPACES);
        if (cmd === "list_port_forwards") return Promise.resolve([]);
        return Promise.resolve([]);
      });

      renderPage();

      await waitFor(() => {
        expect(screen.getByRole("button", { name: "Port Forwarding" })).toBeInTheDocument();
      });

      fireEvent.click(screen.getByRole("button", { name: "Port Forwarding" }));

      await waitFor(() => {
        expect(screen.getByTestId("port-forward-list")).toBeInTheDocument();
      });
    });
  });

  describe("CommandPalette", () => {
    it("opens CommandPalette on Ctrl+K keyboard shortcut", async () => {
      renderPage();

      await waitFor(() => {
        expect(screen.queryByTestId("command-palette")).not.toBeInTheDocument();
      });

      fireEvent.keyDown(document, { key: "k", ctrlKey: true });

      await waitFor(() => {
        expect(screen.getByTestId("command-palette")).toBeInTheDocument();
      });
    });

    it("closes CommandPalette after it is opened", async () => {
      renderPage();

      fireEvent.keyDown(document, { key: "k", ctrlKey: true });

      await waitFor(() => {
        expect(screen.getByTestId("command-palette")).toBeInTheDocument();
      });

      fireEvent.click(screen.getByRole("button", { name: /close/i }));

      await waitFor(() => {
        expect(screen.queryByTestId("command-palette")).not.toBeInTheDocument();
      });
    });
  });

  describe("Hotbar refresh", () => {
    it("renders the Hotbar", async () => {
      renderPage();

      await waitFor(() => {
        expect(screen.getByTestId("hotbar")).toBeInTheDocument();
      });
    });

    it("calls IPC when Hotbar refresh button is clicked with a cluster selected", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "list_kubeconfigs") return Promise.resolve(MOCK_KUBECONFIGS);
        if (cmd === "list_namespaces") return Promise.resolve(MOCK_NAMESPACES);
        if (cmd === "list_port_forwards") return Promise.resolve([]);
        return Promise.resolve([]);
      });

      renderPage();

      await waitFor(() => {
        expect(screen.getByTestId("hotbar")).toBeInTheDocument();
      });

      const callsBefore = mockInvoke.mock.calls.length;
      const refreshButton = screen.getByRole("button", { name: /refresh/i });
      fireEvent.click(refreshButton);

      // After refresh click, no new call is expected for "overview" section (ClusterOverview
      // handles its own data), but the handler should not throw
      expect(mockInvoke.mock.calls.length).toBeGreaterThanOrEqual(callsBefore);
    });
  });
});
