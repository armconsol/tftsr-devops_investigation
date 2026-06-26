/**
 * TDD tests: action IPC calls in network/config/storage/access-control list
 * components must use the item's own .namespace, never the filter prop (which
 * can be "all" when the user is viewing all namespaces).
 */

import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";

import { ServiceList } from "@/components/Kubernetes/ServiceList";
import { IngressList } from "@/components/Kubernetes/IngressList";
import { ConfigMapList } from "@/components/Kubernetes/ConfigMapList";
import { SecretList } from "@/components/Kubernetes/SecretList";
import { HPAList } from "@/components/Kubernetes/HPAList";
import { PVCList } from "@/components/Kubernetes/PVCList";
import { ServiceAccountList } from "@/components/Kubernetes/ServiceAccountList";
import { RoleList } from "@/components/Kubernetes/RoleList";
import { RoleBindingList } from "@/components/Kubernetes/RoleBindingList";
import { NetworkPolicyList } from "@/components/Kubernetes/NetworkPolicyList";
import { ResourceQuotaList } from "@/components/Kubernetes/ResourceQuotaList";
import { LimitRangeList } from "@/components/Kubernetes/LimitRangeList";

import type {
  ServiceInfo,
  IngressInfo,
  ConfigMapInfo,
  SecretInfo,
  HorizontalPodAutoscalerInfo,
  PersistentVolumeClaimInfo,
  ServiceAccountInfo,
  RoleInfo,
  RoleBindingInfo,
  NetworkPolicyInfo,
  ResourceQuotaInfo,
  LimitRangeInfo,
} from "@/lib/tauriCommands";

type MockedInvoke = typeof invoke & {
  mockResolvedValue: (v: unknown) => void;
  mockImplementation: (fn: (cmd: string, args?: unknown) => Promise<unknown>) => void;
};

const mockInvoke = invoke as MockedInvoke;

// ─── helpers ─────────────────────────────────────────────────────────────────

/** Open the action menu for the first row whose name cell matches `name`. */
function openActionMenu(name: string) {
  const cell = screen.getByText(name);
  const row = cell.closest("tr")!;
  const btn = row.querySelector("button")!;
  fireEvent.click(btn);
}

/** Click the first menu item that contains `label`. */
function clickMenuItem(label: string) {
  const item = screen.getByText(label);
  fireEvent.click(item);
}

// ─── ServiceList ─────────────────────────────────────────────────────────────

describe("ServiceList – action IPC uses item.namespace", () => {
  const svc: ServiceInfo = {
    name: "my-svc",
    namespace: "production",
    type: "ClusterIP",
    cluster_ip: "10.0.0.1",
    ports: [],
    age: "1d",
    selector: {},
  };

  beforeEach(() => vi.clearAllMocks());

  it("getResourceYamlCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue("yaml: content");
    render(
      <ServiceList services={[svc]} clusterId="c1" namespace="all" />
    );
    openActionMenu("my-svc");
    clickMenuItem("Edit");
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "services",
        namespace: "production",
        resourceName: "my-svc",
      })
    );
  });

  it("deleteResourceCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);
    render(
      <ServiceList services={[svc]} clusterId="c1" namespace="all" onRefresh={() => {}} />
    );
    openActionMenu("my-svc");
    clickMenuItem("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /confirm|delete/i });
    fireEvent.click(confirmBtn);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "services",
        namespace: "production",
        resourceName: "my-svc",
      })
    );
  });
});

// ─── IngressList ─────────────────────────────────────────────────────────────

describe("IngressList – action IPC uses item.namespace", () => {
  const ing: IngressInfo = {
    name: "my-ingress",
    namespace: "staging",
    host: "example.com",
    addresses: [],
    age: "2d",
  };

  beforeEach(() => vi.clearAllMocks());

  it("getResourceYamlCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue("yaml: content");
    render(
      <IngressList ingresses={[ing]} clusterId="c1" namespace="all" />
    );
    openActionMenu("my-ingress");
    clickMenuItem("Edit");
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "ingresses",
        namespace: "staging",
        resourceName: "my-ingress",
      })
    );
  });

  it("deleteResourceCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);
    render(
      <IngressList ingresses={[ing]} clusterId="c1" namespace="all" onRefresh={() => {}} />
    );
    openActionMenu("my-ingress");
    clickMenuItem("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /confirm|delete/i });
    fireEvent.click(confirmBtn);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "ingresses",
        namespace: "staging",
        resourceName: "my-ingress",
      })
    );
  });
});

// ─── ConfigMapList ────────────────────────────────────────────────────────────

describe("ConfigMapList – action IPC uses item.namespace", () => {
  const cm: ConfigMapInfo = {
    name: "app-config",
    namespace: "kube-system",
    data_keys: 3,
    age: "10d",
  };

  beforeEach(() => vi.clearAllMocks());

  it("getResourceYamlCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue("yaml: content");
    render(
      <ConfigMapList configmaps={[cm]} clusterId="c1" namespace="all" />
    );
    openActionMenu("app-config");
    clickMenuItem("Edit");
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "configmaps",
        namespace: "kube-system",
        resourceName: "app-config",
      })
    );
  });

  it("deleteResourceCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);
    render(
      <ConfigMapList configmaps={[cm]} clusterId="c1" namespace="all" onRefresh={() => {}} />
    );
    openActionMenu("app-config");
    clickMenuItem("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /confirm|delete/i });
    fireEvent.click(confirmBtn);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "configmaps",
        namespace: "kube-system",
        resourceName: "app-config",
      })
    );
  });
});

// ─── SecretList ───────────────────────────────────────────────────────────────

describe("SecretList – action IPC uses item.namespace", () => {
  const secret: SecretInfo = {
    name: "db-creds",
    namespace: "production",
    type: "Opaque",
    data_keys: 2,
    age: "5d",
  };

  beforeEach(() => vi.clearAllMocks());

  it("getResourceYamlCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue("yaml: content");
    render(
      <SecretList secrets={[secret]} clusterId="c1" namespace="all" />
    );
    openActionMenu("db-creds");
    clickMenuItem("Edit");
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "secrets",
        namespace: "production",
        resourceName: "db-creds",
      })
    );
  });

  it("deleteResourceCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);
    render(
      <SecretList secrets={[secret]} clusterId="c1" namespace="all" onRefresh={() => {}} />
    );
    openActionMenu("db-creds");
    clickMenuItem("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /confirm|delete/i });
    fireEvent.click(confirmBtn);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "secrets",
        namespace: "production",
        resourceName: "db-creds",
      })
    );
  });
});

// ─── HPAList ──────────────────────────────────────────────────────────────────

describe("HPAList – action IPC uses item.namespace", () => {
  const hpa: HorizontalPodAutoscalerInfo = {
    name: "web-hpa",
    namespace: "default",
    min_replicas: 1,
    max_replicas: 10,
    current_replicas: 3,
    desired_replicas: 3,
    age: "7d",
  };

  beforeEach(() => vi.clearAllMocks());

  it("getResourceYamlCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue("yaml: content");
    render(
      <HPAList hpas={[hpa]} clusterId="c1" namespace="all" />
    );
    openActionMenu("web-hpa");
    clickMenuItem("Edit");
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "horizontalpodautoscalers",
        namespace: "default",
        resourceName: "web-hpa",
      })
    );
  });

  it("deleteResourceCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);
    render(
      <HPAList hpas={[hpa]} clusterId="c1" namespace="all" onRefresh={() => {}} />
    );
    openActionMenu("web-hpa");
    clickMenuItem("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /confirm|delete/i });
    fireEvent.click(confirmBtn);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "horizontalpodautoscalers",
        namespace: "default",
        resourceName: "web-hpa",
      })
    );
  });
});

// ─── PVCList ──────────────────────────────────────────────────────────────────

describe("PVCList – action IPC uses item.namespace", () => {
  const pvc: PersistentVolumeClaimInfo = {
    name: "data-pvc",
    namespace: "staging",
    status: "Bound",
    volume: "pv-001",
    capacity: "10Gi",
    access_modes: ["ReadWriteOnce"],
    age: "3d",
  };

  beforeEach(() => vi.clearAllMocks());

  it("getResourceYamlCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue("yaml: content");
    render(
      <PVCList pvcs={[pvc]} clusterId="c1" namespace="all" />
    );
    openActionMenu("data-pvc");
    clickMenuItem("Edit");
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "persistentvolumeclaims",
        namespace: "staging",
        resourceName: "data-pvc",
      })
    );
  });

  it("deleteResourceCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);
    render(
      <PVCList pvcs={[pvc]} clusterId="c1" namespace="all" onRefresh={() => {}} />
    );
    openActionMenu("data-pvc");
    clickMenuItem("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /confirm|delete/i });
    fireEvent.click(confirmBtn);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "persistentvolumeclaims",
        namespace: "staging",
        resourceName: "data-pvc",
      })
    );
  });
});

// ─── ServiceAccountList ───────────────────────────────────────────────────────

describe("ServiceAccountList – action IPC uses item.namespace", () => {
  const sa: ServiceAccountInfo = {
    name: "app-sa",
    namespace: "production",
    secrets: 1,
    age: "30d",
  };

  beforeEach(() => vi.clearAllMocks());

  it("getResourceYamlCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue("yaml: content");
    render(
      <ServiceAccountList serviceAccounts={[sa]} clusterId="c1" namespace="all" />
    );
    openActionMenu("app-sa");
    clickMenuItem("Edit");
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "serviceaccounts",
        namespace: "production",
        resourceName: "app-sa",
      })
    );
  });

  it("deleteResourceCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);
    render(
      <ServiceAccountList serviceAccounts={[sa]} clusterId="c1" namespace="all" onRefresh={() => {}} />
    );
    openActionMenu("app-sa");
    clickMenuItem("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /confirm|delete/i });
    fireEvent.click(confirmBtn);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "serviceaccounts",
        namespace: "production",
        resourceName: "app-sa",
      })
    );
  });
});

// ─── RoleList ─────────────────────────────────────────────────────────────────

describe("RoleList – action IPC uses item.namespace", () => {
  const role: RoleInfo = {
    name: "pod-reader",
    namespace: "default",
    age: "14d",
  };

  beforeEach(() => vi.clearAllMocks());

  it("getResourceYamlCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue("yaml: content");
    render(
      <RoleList roles={[role]} clusterId="c1" namespace="all" />
    );
    openActionMenu("pod-reader");
    clickMenuItem("Edit");
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "roles",
        namespace: "default",
        resourceName: "pod-reader",
      })
    );
  });

  it("deleteResourceCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);
    render(
      <RoleList roles={[role]} clusterId="c1" namespace="all" onRefresh={() => {}} />
    );
    openActionMenu("pod-reader");
    clickMenuItem("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /confirm|delete/i });
    fireEvent.click(confirmBtn);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "roles",
        namespace: "default",
        resourceName: "pod-reader",
      })
    );
  });
});

// ─── RoleBindingList ──────────────────────────────────────────────────────────

describe("RoleBindingList – action IPC uses item.namespace", () => {
  const rb: RoleBindingInfo = {
    name: "pod-reader-binding",
    namespace: "default",
    role: "pod-reader",
    age: "10d",
  };

  beforeEach(() => vi.clearAllMocks());

  it("getResourceYamlCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue("yaml: content");
    render(
      <RoleBindingList roleBindings={[rb]} clusterId="c1" namespace="all" />
    );
    openActionMenu("pod-reader-binding");
    clickMenuItem("Edit");
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "rolebindings",
        namespace: "default",
        resourceName: "pod-reader-binding",
      })
    );
  });

  it("deleteResourceCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);
    render(
      <RoleBindingList roleBindings={[rb]} clusterId="c1" namespace="all" onRefresh={() => {}} />
    );
    openActionMenu("pod-reader-binding");
    clickMenuItem("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /confirm|delete/i });
    fireEvent.click(confirmBtn);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "rolebindings",
        namespace: "default",
        resourceName: "pod-reader-binding",
      })
    );
  });
});

// ─── NetworkPolicyList ────────────────────────────────────────────────────────

describe("NetworkPolicyList – action IPC uses item.namespace", () => {
  const np: NetworkPolicyInfo = {
    name: "deny-all",
    namespace: "production",
    pod_selector: "{}",
    policy_types: ["Ingress"],
    age: "3d",
  };

  beforeEach(() => vi.clearAllMocks());

  it("getResourceYamlCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue("yaml: content");
    render(
      <NetworkPolicyList networkpolicies={[np]} clusterId="c1" namespace="all" />
    );
    openActionMenu("deny-all");
    clickMenuItem("Edit");
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "networkpolicies",
        namespace: "production",
        resourceName: "deny-all",
      })
    );
  });

  it("deleteResourceCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);
    render(
      <NetworkPolicyList networkpolicies={[np]} clusterId="c1" namespace="all" onRefresh={() => {}} />
    );
    openActionMenu("deny-all");
    clickMenuItem("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /confirm|delete/i });
    fireEvent.click(confirmBtn);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "networkpolicies",
        namespace: "production",
        resourceName: "deny-all",
      })
    );
  });
});

// ─── ResourceQuotaList ────────────────────────────────────────────────────────

describe("ResourceQuotaList – action IPC uses item.namespace", () => {
  const rq: ResourceQuotaInfo = {
    name: "compute-resources",
    namespace: "default",
    request_cpu: "4",
    request_memory: "8Gi",
    limit_cpu: "8",
    limit_memory: "16Gi",
    age: "7d",
  };

  beforeEach(() => vi.clearAllMocks());

  it("getResourceYamlCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue("yaml: content");
    render(
      <ResourceQuotaList resourcequotas={[rq]} clusterId="c1" namespace="all" />
    );
    openActionMenu("compute-resources");
    clickMenuItem("Edit");
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "resourcequotas",
        namespace: "default",
        resourceName: "compute-resources",
      })
    );
  });

  it("deleteResourceCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);
    render(
      <ResourceQuotaList resourcequotas={[rq]} clusterId="c1" namespace="all" onRefresh={() => {}} />
    );
    openActionMenu("compute-resources");
    clickMenuItem("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /confirm|delete/i });
    fireEvent.click(confirmBtn);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "resourcequotas",
        namespace: "default",
        resourceName: "compute-resources",
      })
    );
  });
});

// ─── LimitRangeList ───────────────────────────────────────────────────────────

describe("LimitRangeList – action IPC uses item.namespace", () => {
  const lr: LimitRangeInfo = {
    name: "cpu-mem-limits",
    namespace: "default",
    limit_count: 3,
    age: "14d",
  };

  beforeEach(() => vi.clearAllMocks());

  it("getResourceYamlCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue("yaml: content");
    render(
      <LimitRangeList limitranges={[lr]} clusterId="c1" namespace="all" />
    );
    openActionMenu("cpu-mem-limits");
    clickMenuItem("Edit");
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "limitranges",
        namespace: "default",
        resourceName: "cpu-mem-limits",
      })
    );
  });

  it("deleteResourceCmd receives item.namespace, not filter prop 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);
    render(
      <LimitRangeList limitranges={[lr]} clusterId="c1" namespace="all" onRefresh={() => {}} />
    );
    openActionMenu("cpu-mem-limits");
    clickMenuItem("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /confirm|delete/i });
    fireEvent.click(confirmBtn);
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "limitranges",
        namespace: "default",
        resourceName: "cpu-mem-limits",
      })
    );
  });
});
