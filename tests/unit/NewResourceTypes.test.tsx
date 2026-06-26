import React from "react";
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { StorageClassList } from "@/components/Kubernetes/StorageClassList";
import { NetworkPolicyList } from "@/components/Kubernetes/NetworkPolicyList";
import { ResourceQuotaList } from "@/components/Kubernetes/ResourceQuotaList";
import { LimitRangeList } from "@/components/Kubernetes/LimitRangeList";
import type {
  StorageClassInfo,
  NetworkPolicyInfo,
  ResourceQuotaInfo,
  LimitRangeInfo,
} from "@/lib/tauriCommands";

// ─── StorageClassList ─────────────────────────────────────────────────────────

describe("StorageClassList", () => {
  const mockStorageClasses: StorageClassInfo[] = [
    {
      name: "standard",
      provisioner: "kubernetes.io/no-provisioner",
      reclaim_policy: "Retain",
      volume_binding_mode: "WaitForFirstConsumer",
      allow_volume_expansion: true,
      age: "10d",
    },
    {
      name: "fast-ssd",
      provisioner: "csi.vsphere.vmware.com",
      reclaim_policy: "Delete",
      volume_binding_mode: "Immediate",
      allow_volume_expansion: false,
      age: "5d",
    },
  ];

  it("renders storage class names", () => {
    render(
      <StorageClassList
        storageclasses={mockStorageClasses}
        clusterId="cluster-1"
        namespace=""
      />
    );
    expect(screen.getByText("standard")).toBeDefined();
    expect(screen.getByText("fast-ssd")).toBeDefined();
  });

  it("shows provisioner", () => {
    render(
      <StorageClassList
        storageclasses={mockStorageClasses}
        clusterId="cluster-1"
        namespace=""
      />
    );
    expect(screen.getByText("kubernetes.io/no-provisioner")).toBeDefined();
    expect(screen.getByText("csi.vsphere.vmware.com")).toBeDefined();
  });

  it("shows reclaim policy", () => {
    render(
      <StorageClassList
        storageclasses={mockStorageClasses}
        clusterId="cluster-1"
        namespace=""
      />
    );
    expect(screen.getByText("Retain")).toBeDefined();
    expect(screen.getByText("Delete")).toBeDefined();
  });

  it("shows volume expansion status", () => {
    render(
      <StorageClassList
        storageclasses={mockStorageClasses}
        clusterId="cluster-1"
        namespace=""
      />
    );
    expect(screen.getByText("Yes")).toBeDefined();
    expect(screen.getByText("No")).toBeDefined();
  });

  it("shows empty state when no items", () => {
    render(
      <StorageClassList storageclasses={[]} clusterId="cluster-1" namespace="" />
    );
    expect(screen.getByText("No storage classes found")).toBeDefined();
  });

  it("renders column headers", () => {
    render(
      <StorageClassList storageclasses={[]} clusterId="cluster-1" namespace="" />
    );
    expect(screen.getByText("Name")).toBeDefined();
    expect(screen.getByText("Provisioner")).toBeDefined();
    expect(screen.getByText("Reclaim Policy")).toBeDefined();
    expect(screen.getByText("Volume Binding Mode")).toBeDefined();
    expect(screen.getByText("Expand")).toBeDefined();
    expect(screen.getByText("Age")).toBeDefined();
  });
});

// ─── NetworkPolicyList ────────────────────────────────────────────────────────

describe("NetworkPolicyList", () => {
  const mockNetworkPolicies: NetworkPolicyInfo[] = [
    {
      name: "deny-all",
      namespace: "production",
      pod_selector: "{}",
      policy_types: ["Ingress", "Egress"],
      age: "3d",
    },
    {
      name: "allow-frontend",
      namespace: "production",
      pod_selector: '{"matchLabels":{"app":"frontend"}}',
      policy_types: ["Ingress"],
      age: "1d",
    },
  ];

  it("renders network policy names", () => {
    render(
      <NetworkPolicyList
        networkpolicies={mockNetworkPolicies}
        clusterId="cluster-1"
        namespace="production"
      />
    );
    expect(screen.getByText("deny-all")).toBeDefined();
    expect(screen.getByText("allow-frontend")).toBeDefined();
  });

  it("shows namespace", () => {
    render(
      <NetworkPolicyList
        networkpolicies={mockNetworkPolicies}
        clusterId="cluster-1"
        namespace="production"
      />
    );
    const cells = screen.getAllByText("production");
    expect(cells.length).toBeGreaterThan(0);
  });

  it("shows policy types joined by comma", () => {
    render(
      <NetworkPolicyList
        networkpolicies={mockNetworkPolicies}
        clusterId="cluster-1"
        namespace="production"
      />
    );
    expect(screen.getByText("Ingress, Egress")).toBeDefined();
    expect(screen.getByText("Ingress")).toBeDefined();
  });

  it("shows empty state when no items", () => {
    render(
      <NetworkPolicyList
        networkpolicies={[]}
        clusterId="cluster-1"
        namespace="production"
      />
    );
    expect(screen.getByText("No network policies found")).toBeDefined();
  });

  it("renders column headers", () => {
    render(
      <NetworkPolicyList networkpolicies={[]} clusterId="cluster-1" namespace="" />
    );
    expect(screen.getByText("Name")).toBeDefined();
    expect(screen.getByText("Namespace")).toBeDefined();
    expect(screen.getByText("Pod Selector")).toBeDefined();
    expect(screen.getByText("Policy Types")).toBeDefined();
    expect(screen.getByText("Age")).toBeDefined();
  });
});

// ─── ResourceQuotaList ────────────────────────────────────────────────────────

describe("ResourceQuotaList", () => {
  const mockResourceQuotas: ResourceQuotaInfo[] = [
    {
      name: "compute-resources",
      namespace: "default",
      request_cpu: "4",
      request_memory: "8Gi",
      limit_cpu: "8",
      limit_memory: "16Gi",
      age: "7d",
    },
    {
      name: "object-counts",
      namespace: "staging",
      request_cpu: "",
      request_memory: "",
      limit_cpu: "",
      limit_memory: "",
      age: "2d",
    },
  ];

  it("renders resource quota names", () => {
    render(
      <ResourceQuotaList
        resourcequotas={mockResourceQuotas}
        clusterId="cluster-1"
        namespace="default"
      />
    );
    expect(screen.getByText("compute-resources")).toBeDefined();
    expect(screen.getByText("object-counts")).toBeDefined();
  });

  it("shows CPU and memory limits", () => {
    render(
      <ResourceQuotaList
        resourcequotas={mockResourceQuotas}
        clusterId="cluster-1"
        namespace="default"
      />
    );
    expect(screen.getByText("4")).toBeDefined();
    expect(screen.getByText("8Gi")).toBeDefined();
    expect(screen.getByText("8")).toBeDefined();
    expect(screen.getByText("16Gi")).toBeDefined();
  });

  it("shows dash for empty fields", () => {
    render(
      <ResourceQuotaList
        resourcequotas={mockResourceQuotas}
        clusterId="cluster-1"
        namespace="default"
      />
    );
    const dashes = screen.getAllByText("—");
    expect(dashes.length).toBeGreaterThan(0);
  });

  it("shows empty state when no items", () => {
    render(
      <ResourceQuotaList
        resourcequotas={[]}
        clusterId="cluster-1"
        namespace="default"
      />
    );
    expect(screen.getByText("No resource quotas found")).toBeDefined();
  });

  it("renders column headers", () => {
    render(
      <ResourceQuotaList resourcequotas={[]} clusterId="cluster-1" namespace="" />
    );
    expect(screen.getByText("Name")).toBeDefined();
    expect(screen.getByText("Namespace")).toBeDefined();
    expect(screen.getByText("CPU Req")).toBeDefined();
    expect(screen.getByText("Mem Req")).toBeDefined();
    expect(screen.getByText("CPU Limit")).toBeDefined();
    expect(screen.getByText("Mem Limit")).toBeDefined();
    expect(screen.getByText("Age")).toBeDefined();
  });
});

// ─── LimitRangeList ───────────────────────────────────────────────────────────

describe("LimitRangeList", () => {
  const mockLimitRanges: LimitRangeInfo[] = [
    {
      name: "cpu-mem-limits",
      namespace: "default",
      limit_count: 3,
      age: "14d",
    },
    {
      name: "container-defaults",
      namespace: "staging",
      limit_count: 1,
      age: "6d",
    },
  ];

  it("renders limit range names", () => {
    render(
      <LimitRangeList
        limitranges={mockLimitRanges}
        clusterId="cluster-1"
        namespace="default"
      />
    );
    expect(screen.getByText("cpu-mem-limits")).toBeDefined();
    expect(screen.getByText("container-defaults")).toBeDefined();
  });

  it("shows limit count", () => {
    render(
      <LimitRangeList
        limitranges={mockLimitRanges}
        clusterId="cluster-1"
        namespace="default"
      />
    );
    expect(screen.getByText("3")).toBeDefined();
    expect(screen.getByText("1")).toBeDefined();
  });

  it("shows namespace", () => {
    render(
      <LimitRangeList
        limitranges={mockLimitRanges}
        clusterId="cluster-1"
        namespace="default"
      />
    );
    expect(screen.getByText("default")).toBeDefined();
    expect(screen.getByText("staging")).toBeDefined();
  });

  it("shows empty state when no items", () => {
    render(
      <LimitRangeList limitranges={[]} clusterId="cluster-1" namespace="default" />
    );
    expect(screen.getByText("No limit ranges found")).toBeDefined();
  });

  it("renders column headers", () => {
    render(
      <LimitRangeList limitranges={[]} clusterId="cluster-1" namespace="" />
    );
    expect(screen.getByText("Name")).toBeDefined();
    expect(screen.getByText("Namespace")).toBeDefined();
    expect(screen.getByText("Limits")).toBeDefined();
    expect(screen.getByText("Age")).toBeDefined();
  });
});
