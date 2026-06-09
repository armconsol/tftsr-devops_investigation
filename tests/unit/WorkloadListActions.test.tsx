import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { DeploymentList } from "@/components/Kubernetes/DeploymentList";
import { StatefulSetList } from "@/components/Kubernetes/StatefulSetList";
import { DaemonSetList } from "@/components/Kubernetes/DaemonSetList";
import { ReplicaSetList } from "@/components/Kubernetes/ReplicaSetList";
import { JobList } from "@/components/Kubernetes/JobList";
import { CronJobList } from "@/components/Kubernetes/CronJobList";
import type {
  DeploymentInfo,
  StatefulSetInfo,
  DaemonSetInfo,
  ReplicaSetInfo,
  JobInfo,
  CronJobInfo,
} from "@/lib/tauriCommands";

type MockedInvoke = typeof invoke & {
  mockResolvedValue: (v: unknown) => void;
  mockImplementation: (fn: (cmd: string) => Promise<unknown>) => void;
};

const mockInvoke = invoke as MockedInvoke;

// Helper: open the action menu for the first Actions button, then click a menu item by label
async function openMenuAndClick(label: string) {
  const btn = screen.getAllByRole("button", { name: /actions/i })[0];
  fireEvent.click(btn);
  const item = await screen.findByRole("button", { name: new RegExp(label, "i") });
  fireEvent.click(item);
}

// ─── DeploymentList ──────────────────────────────────────────────────────────

describe("DeploymentList — actions use item.namespace not filter prop", () => {
  const deployment: DeploymentInfo = {
    name: "nginx",
    namespace: "kube-system",
    ready: "1/1",
    up_to_date: "1",
    available: "1",
    replicas: 1,
    age: "1d",
    labels: {},
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue("apiVersion: apps/v1");
  });

  it("openEdit calls getResourceYamlCmd with item.namespace, not 'all'", async () => {
    render(
      <DeploymentList
        deployments={[deployment]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Edit");

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "deployments",
        namespace: "kube-system",
        resourceName: "nginx",
      });
    });
  });

  it("handleDelete calls deleteResourceCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "delete_resource") return undefined;
      return "yaml";
    });

    render(
      <DeploymentList
        deployments={[deployment]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /delete|confirm/i });
    fireEvent.click(confirmBtn);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "deployments",
        namespace: "kube-system",
        resourceName: "nginx",
      });
    });
  });

  it("ScaleModal onScale calls scaleDeploymentCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);

    render(
      <DeploymentList
        deployments={[deployment]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Scale");
    const scaleBtn = await screen.findByRole("button", { name: /scale/i });
    fireEvent.click(scaleBtn);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("scale_deployment", {
        clusterId: "c1",
        namespace: "kube-system",
        deploymentName: "nginx",
        replicas: expect.any(Number),
      });
    });
  });

  it("handleRestart calls restartDeploymentCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "restart_deployment") return undefined;
      return "yaml";
    });

    render(
      <DeploymentList
        deployments={[deployment]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Restart");
    const confirmBtn = await screen.findByRole("button", { name: /delete|confirm|restart/i });
    fireEvent.click(confirmBtn);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("restart_deployment", {
        clusterId: "c1",
        namespace: "kube-system",
        deploymentName: "nginx",
      });
    });
  });

  it("handleRollback calls rollbackDeploymentCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "rollback_deployment") return undefined;
      return "yaml";
    });

    render(
      <DeploymentList
        deployments={[deployment]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Rollback");
    const confirmBtn = await screen.findByRole("button", { name: /delete|confirm|rollback/i });
    fireEvent.click(confirmBtn);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("rollback_deployment", {
        clusterId: "c1",
        namespace: "kube-system",
        deploymentName: "nginx",
      });
    });
  });
});

// ─── StatefulSetList ─────────────────────────────────────────────────────────

describe("StatefulSetList — actions use item.namespace not filter prop", () => {
  const ss: StatefulSetInfo = {
    name: "postgres",
    namespace: "kube-system",
    ready: "1/1",
    replicas: 1,
    age: "2d",
    labels: {},
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue("apiVersion: apps/v1");
  });

  it("openEdit calls getResourceYamlCmd with item.namespace, not 'all'", async () => {
    render(
      <StatefulSetList
        statefulsets={[ss]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Edit");

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "statefulsets",
        namespace: "kube-system",
        resourceName: "postgres",
      });
    });
  });

  it("handleDelete calls deleteResourceCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "delete_resource") return undefined;
      return "yaml";
    });

    render(
      <StatefulSetList
        statefulsets={[ss]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /delete|confirm/i });
    fireEvent.click(confirmBtn);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "statefulsets",
        namespace: "kube-system",
        resourceName: "postgres",
      });
    });
  });

  it("ScaleModal onScale calls scaleStatefulsetCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);

    render(
      <StatefulSetList
        statefulsets={[ss]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Scale");
    const scaleBtn = await screen.findByRole("button", { name: /scale/i });
    fireEvent.click(scaleBtn);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("scale_statefulset", {
        clusterId: "c1",
        namespace: "kube-system",
        name: "postgres",
        replicas: expect.any(Number),
      });
    });
  });

  it("handleRestart calls restartStatefulsetCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "restart_statefulset") return undefined;
      return "yaml";
    });

    render(
      <StatefulSetList
        statefulsets={[ss]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Restart");
    const confirmBtn = await screen.findByRole("button", { name: /delete|confirm|restart/i });
    fireEvent.click(confirmBtn);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("restart_statefulset", {
        clusterId: "c1",
        namespace: "kube-system",
        name: "postgres",
      });
    });
  });
});

// ─── DaemonSetList ───────────────────────────────────────────────────────────

describe("DaemonSetList — actions use item.namespace not filter prop", () => {
  const ds: DaemonSetInfo = {
    name: "fluentd",
    namespace: "kube-system",
    desired: 3,
    current: 3,
    ready: 3,
    up_to_date: 3,
    available: 3,
    age: "5d",
    labels: {},
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue("apiVersion: apps/v1");
  });

  it("openEdit calls getResourceYamlCmd with item.namespace, not 'all'", async () => {
    render(
      <DaemonSetList
        daemonsets={[ds]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Edit");

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "daemonsets",
        namespace: "kube-system",
        resourceName: "fluentd",
      });
    });
  });

  it("handleDelete calls deleteResourceCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "delete_resource") return undefined;
      return "yaml";
    });

    render(
      <DaemonSetList
        daemonsets={[ds]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /delete|confirm/i });
    fireEvent.click(confirmBtn);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "daemonsets",
        namespace: "kube-system",
        resourceName: "fluentd",
      });
    });
  });

  it("handleRestart calls restartDaemonsetCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "restart_daemonset") return undefined;
      return "yaml";
    });

    render(
      <DaemonSetList
        daemonsets={[ds]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Restart");
    const confirmBtn = await screen.findByRole("button", { name: /delete|confirm|restart/i });
    fireEvent.click(confirmBtn);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("restart_daemonset", {
        clusterId: "c1",
        namespace: "kube-system",
        name: "fluentd",
      });
    });
  });
});

// ─── ReplicaSetList ──────────────────────────────────────────────────────────

describe("ReplicaSetList — actions use item.namespace not filter prop", () => {
  const rs: ReplicaSetInfo = {
    name: "nginx-abc12",
    namespace: "kube-system",
    replicas: 2,
    ready: "2",
    age: "3d",
    labels: { app: "nginx" },
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue("apiVersion: apps/v1");
  });

  it("openEdit calls getResourceYamlCmd with item.namespace, not 'all'", async () => {
    render(
      <ReplicaSetList
        replicaSets={[rs]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Edit");

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "replicasets",
        namespace: "kube-system",
        resourceName: "nginx-abc12",
      });
    });
  });

  it("handleDelete calls deleteResourceCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "delete_resource") return undefined;
      return "yaml";
    });

    render(
      <ReplicaSetList
        replicaSets={[rs]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /delete|confirm/i });
    fireEvent.click(confirmBtn);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "replicasets",
        namespace: "kube-system",
        resourceName: "nginx-abc12",
      });
    });
  });

  it("ScaleModal onScale calls scaleReplicasetCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockResolvedValue(undefined);

    render(
      <ReplicaSetList
        replicaSets={[rs]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Scale");
    const scaleBtn = await screen.findByRole("button", { name: /scale/i });
    fireEvent.click(scaleBtn);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("scale_replicaset", {
        clusterId: "c1",
        namespace: "kube-system",
        name: "nginx-abc12",
        replicas: expect.any(Number),
      });
    });
  });
});

// ─── JobList ─────────────────────────────────────────────────────────────────

describe("JobList — actions use item.namespace not filter prop", () => {
  const job: JobInfo = {
    name: "db-migrate",
    namespace: "kube-system",
    completions: "1/1",
    duration: "45s",
    age: "1d",
    labels: {},
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue("apiVersion: batch/v1");
  });

  it("openEdit calls getResourceYamlCmd with item.namespace, not 'all'", async () => {
    render(
      <JobList
        jobs={[job]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Edit");

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "jobs",
        namespace: "kube-system",
        resourceName: "db-migrate",
      });
    });
  });

  it("handleDelete calls deleteResourceCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "delete_resource") return undefined;
      return "yaml";
    });

    render(
      <JobList
        jobs={[job]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /delete|confirm/i });
    fireEvent.click(confirmBtn);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "jobs",
        namespace: "kube-system",
        resourceName: "db-migrate",
      });
    });
  });
});

// ─── CronJobList ─────────────────────────────────────────────────────────────

describe("CronJobList — actions use item.namespace not filter prop", () => {
  const cj: CronJobInfo = {
    name: "backup",
    namespace: "kube-system",
    schedule: "0 2 * * *",
    active: 0,
    last_schedule: "1h",
    age: "10d",
    labels: {},
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue("apiVersion: batch/v1");
  });

  it("openEdit calls getResourceYamlCmd with item.namespace, not 'all'", async () => {
    render(
      <CronJobList
        cronJobs={[cj]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Edit");

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        resourceType: "cronjobs",
        namespace: "kube-system",
        resourceName: "backup",
      });
    });
  });

  it("handleDelete calls deleteResourceCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "delete_resource") return undefined;
      return "yaml";
    });

    render(
      <CronJobList
        cronJobs={[cj]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Delete");
    const confirmBtn = await screen.findByRole("button", { name: /delete|confirm/i });
    fireEvent.click(confirmBtn);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        resourceType: "cronjobs",
        namespace: "kube-system",
        resourceName: "backup",
      });
    });
  });

  it("handleSuspend calls suspendCronjobCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "suspend_cronjob") return undefined;
      return "yaml";
    });

    render(
      <CronJobList
        cronJobs={[cj]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Suspend");

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("suspend_cronjob", {
        clusterId: "c1",
        namespace: "kube-system",
        name: "backup",
      });
    });
  });

  it("handleTrigger calls triggerCronjobCmd with item.namespace, not 'all'", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "trigger_cronjob") return undefined;
      return "yaml";
    });

    render(
      <CronJobList
        cronJobs={[cj]}
        clusterId="c1"
        namespace="all"
        onRefresh={vi.fn()}
      />
    );

    await openMenuAndClick("Trigger");

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("trigger_cronjob", {
        clusterId: "c1",
        namespace: "kube-system",
        name: "backup",
      });
    });
  });
});
