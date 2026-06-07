import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { PodDetail } from "@/components/Kubernetes/PodDetail";
import type { PodInfo } from "@/lib/tauriCommands";

vi.mock("@tauri-apps/api/core");

type MockedInvoke = typeof invoke & {
  mockResolvedValue: (v: unknown) => void;
  mockRejectedValue: (e: Error) => void;
  mockImplementation: (fn: (cmd: string) => Promise<unknown>) => void;
};

const mockInvoke = invoke as MockedInvoke;

const mockPod: PodInfo = {
  name: "nginx-abc123",
  status: "Running",
  ready: "2/2",
  age: "3h",
  containers: ["nginx", "sidecar"],
};

describe("PodDetail", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing when given a PodInfo prop", () => {
    render(
      <PodDetail
        clusterId="cluster-1"
        namespace="default"
        pod={mockPod}
        onClose={() => {}}
      />
    );
    // heading shows pod name
    expect(screen.getByRole("heading", { name: /pod: nginx-abc123/i })).toBeDefined();
  });

  it("shows pod name in heading", () => {
    render(
      <PodDetail
        clusterId="cluster-1"
        namespace="default"
        pod={mockPod}
        onClose={() => {}}
      />
    );
    expect(screen.getByRole("heading", { name: /pod: nginx-abc123/i })).toBeDefined();
  });

  it("shows pod namespace in metadata", () => {
    render(
      <PodDetail
        clusterId="cluster-1"
        namespace="default"
        pod={mockPod}
        onClose={() => {}}
      />
    );
    // badge shows namespace
    const badges = screen.getAllByText("default");
    expect(badges.length).toBeGreaterThan(0);
  });

  it("renders container names from pod.containers", () => {
    render(
      <PodDetail
        clusterId="cluster-1"
        namespace="default"
        pod={mockPod}
        onClose={() => {}}
      />
    );
    // containers appear in the Containers table (and possibly in the dropdown)
    const nginxCells = screen.getAllByText("nginx");
    expect(nginxCells.length).toBeGreaterThan(0);
    const sidecarCells = screen.getAllByText("sidecar");
    expect(sidecarCells.length).toBeGreaterThan(0);
  });

  it("shows loading state during log fetch when logs tab clicked", async () => {
    mockInvoke.mockImplementation(() => new Promise(() => {}));

    render(
      <PodDetail
        clusterId="cluster-1"
        namespace="default"
        pod={mockPod}
        onClose={() => {}}
      />
    );

    const logsTab = screen.getByRole("button", { name: /^logs$/i });
    fireEvent.click(logsTab);

    await waitFor(() => {
      expect(screen.getByTestId("logs-loading")).toBeDefined();
    });
  });

  it("fetches logs via get_pod_logs IPC when logs tab clicked", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_pod_logs") {
        return Promise.resolve({ logs: "INFO Starting up\nINFO Ready" });
      }
      return Promise.resolve(undefined);
    });

    render(
      <PodDetail
        clusterId="cluster-1"
        namespace="default"
        pod={mockPod}
        onClose={() => {}}
      />
    );

    const logsTab = screen.getByRole("button", { name: /^logs$/i });
    fireEvent.click(logsTab);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("get_pod_logs", {
        clusterId: "cluster-1",
        namespace: "default",
        podName: "nginx-abc123",
        containerName: "nginx",
      });
    });
  });

  it("shows error when log fetch fails", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_pod_logs") {
        return Promise.reject(new Error("Connection refused"));
      }
      return Promise.resolve(undefined);
    });

    render(
      <PodDetail
        clusterId="cluster-1"
        namespace="default"
        pod={mockPod}
        onClose={() => {}}
      />
    );

    const logsTab = screen.getByRole("button", { name: /^logs$/i });
    fireEvent.click(logsTab);

    await waitFor(() => {
      expect(screen.getByTestId("logs-error")).toBeDefined();
    });
  });
});
