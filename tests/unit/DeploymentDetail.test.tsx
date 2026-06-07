import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { DeploymentDetail } from "@/components/Kubernetes/DeploymentDetail";
import type { DeploymentInfo } from "@/lib/tauriCommands";

vi.mock("@tauri-apps/api/core");

type MockedInvoke = typeof invoke & {
  mockResolvedValue: (v: unknown) => void;
  mockRejectedValue: (e: Error) => void;
  mockImplementation: (fn: (cmd: string) => Promise<unknown>) => void;
};

const mockInvoke = invoke as MockedInvoke;

const mockDeployment: DeploymentInfo = {
  name: "nginx-deployment",
  namespace: "production",
  ready: "3/3",
  up_to_date: "3",
  available: "3",
  age: "2d",
  replicas: 3,
  labels: { app: "nginx", tier: "frontend" },
};

describe("DeploymentDetail", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders deployment name from DeploymentInfo prop", () => {
    render(
      <DeploymentDetail
        clusterId="cluster-1"
        namespace="production"
        deployment={mockDeployment}
        onClose={() => {}}
      />
    );
    expect(screen.getByRole("heading", { name: /deployment: nginx-deployment/i })).toBeDefined();
  });

  it("shows replica count in ready/total format", () => {
    render(
      <DeploymentDetail
        clusterId="cluster-1"
        namespace="production"
        deployment={mockDeployment}
        onClose={() => {}}
      />
    );
    // ready field "3/3" shown in overview
    expect(screen.getByText("3/3")).toBeDefined();
  });

  it("scale button calls scale_deployment IPC with new replica count", async () => {
    mockInvoke.mockResolvedValue(undefined);

    render(
      <DeploymentDetail
        clusterId="cluster-1"
        namespace="production"
        deployment={mockDeployment}
        onClose={() => {}}
      />
    );

    const actionsTab = screen.getByRole("button", { name: /^actions$/i });
    fireEvent.click(actionsTab);

    await waitFor(() => {
      expect(screen.getByTestId("scale-button")).toBeDefined();
    });

    const scaleButton = screen.getByTestId("scale-button");
    fireEvent.click(scaleButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("scale_deployment", {
        clusterId: "cluster-1",
        namespace: "production",
        deploymentName: "nginx-deployment",
        replicas: 3,
      });
    });
  });

  it("restart button calls restart_deployment IPC", async () => {
    mockInvoke.mockResolvedValue(undefined);

    render(
      <DeploymentDetail
        clusterId="cluster-1"
        namespace="production"
        deployment={mockDeployment}
        onClose={() => {}}
      />
    );

    const actionsTab = screen.getByRole("button", { name: /^actions$/i });
    fireEvent.click(actionsTab);

    await waitFor(() => {
      expect(screen.getByTestId("restart-button")).toBeDefined();
    });

    const restartButton = screen.getByTestId("restart-button");
    fireEvent.click(restartButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("restart_deployment", {
        clusterId: "cluster-1",
        namespace: "production",
        deploymentName: "nginx-deployment",
      });
    });
  });

  it("shows loading state during scale operation", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "scale_deployment") return new Promise(() => {});
      return Promise.resolve(undefined);
    });

    render(
      <DeploymentDetail
        clusterId="cluster-1"
        namespace="production"
        deployment={mockDeployment}
        onClose={() => {}}
      />
    );

    const actionsTab = screen.getByRole("button", { name: /^actions$/i });
    fireEvent.click(actionsTab);

    await waitFor(() => {
      expect(screen.getByTestId("scale-button")).toBeDefined();
    });

    fireEvent.click(screen.getByTestId("scale-button"));

    await waitFor(() => {
      expect(screen.getByTestId("scale-loading")).toBeDefined();
    });
  });

  it("shows error when scale fails", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "scale_deployment") {
        return Promise.reject(new Error("Forbidden"));
      }
      return Promise.resolve(undefined);
    });

    render(
      <DeploymentDetail
        clusterId="cluster-1"
        namespace="production"
        deployment={mockDeployment}
        onClose={() => {}}
      />
    );

    const actionsTab = screen.getByRole("button", { name: /^actions$/i });
    fireEvent.click(actionsTab);

    await waitFor(() => {
      expect(screen.getByTestId("scale-button")).toBeDefined();
    });

    fireEvent.click(screen.getByTestId("scale-button"));

    await waitFor(() => {
      expect(screen.getByTestId("scale-error")).toBeDefined();
    });
  });
});
