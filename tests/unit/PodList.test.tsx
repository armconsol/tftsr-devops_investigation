import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { PodList } from "@/components/Kubernetes/PodList";
import { useBottomPanelStore, BottomPanelTabType } from "@/stores/bottomPanelStore";
import type { PodInfo } from "@/lib/tauriCommands";

vi.mock("@tauri-apps/api/core");

// Silence console.error noise from modal portals in jsdom
vi.mock("@/components/Kubernetes/InteractiveShellModal", () => ({
  InteractiveShellModal: ({ namespace }: { namespace: string }) => (
    <div data-testid="shell-modal" data-namespace={namespace} />
  ),
}));
vi.mock("@/components/Kubernetes/InteractiveAttachModal", () => ({
  InteractiveAttachModal: ({ namespace }: { namespace: string }) => (
    <div data-testid="attach-modal" data-namespace={namespace} />
  ),
}));
vi.mock("@/components/Kubernetes/EditResourceModal", () => ({
  EditResourceModal: ({ namespace }: { namespace: string }) => (
    <div data-testid="edit-modal" data-namespace={namespace} />
  ),
}));
vi.mock("@/components/Kubernetes/ConfirmDeleteDialog", () => ({
  ConfirmDeleteDialog: ({
    onConfirm,
    resourceName,
  }: {
    onConfirm: () => void;
    resourceName: string;
  }) => (
    <div data-testid="confirm-delete">
      <span>{resourceName}</span>
      <button onClick={onConfirm}>confirm</button>
    </div>
  ),
}));

type MockedInvoke = typeof invoke & {
  mockResolvedValue: (v: unknown) => void;
  mockRejectedValue: (e: Error) => void;
  mockImplementation: (fn: (cmd: string) => Promise<unknown>) => void;
};

const mockInvoke = invoke as MockedInvoke;

// A pod whose own namespace ("default") differs from the filter prop ("all")
const mockPod: PodInfo = {
  name: "test-pod",
  namespace: "default",
  status: "Running",
  ready: "1/1",
  age: "1h",
  containers: ["app"],
};

function openActionMenu() {
  const trigger = screen.getByRole("button", { name: /actions/i });
  fireEvent.click(trigger);
}

describe("PodList — namespace isolation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('Edit action calls getResourceYamlCmd with pod.namespace ("default"), not filter "all"', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_resource_yaml") {
        return Promise.resolve("apiVersion: v1\nkind: Pod");
      }
      return Promise.resolve(undefined);
    });

    render(
      <PodList
        pods={[mockPod]}
        clusterId="c1"
        namespace="all"
        onRefresh={() => {}}
      />
    );

    openActionMenu();
    fireEvent.click(screen.getByRole("button", { name: /^edit$/i }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("get_resource_yaml", {
        clusterId: "c1",
        namespace: "default",
        resourceType: "pods",
        resourceName: "test-pod",
      });
    });
  });

  it('Delete action calls deleteResourceCmd with pod.namespace ("default"), not filter "all"', async () => {
    mockInvoke.mockResolvedValue(undefined);

    render(
      <PodList
        pods={[mockPod]}
        clusterId="c1"
        namespace="all"
        onRefresh={() => {}}
      />
    );

    openActionMenu();
    fireEvent.click(screen.getByRole("button", { name: /^delete$/i }));

    await waitFor(() => {
      expect(screen.getByTestId("confirm-delete")).toBeDefined();
    });

    fireEvent.click(screen.getByRole("button", { name: /confirm/i }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("delete_resource", {
        clusterId: "c1",
        namespace: "default",
        resourceType: "pods",
        resourceName: "test-pod",
      });
    });
  });

  it('Force Delete action calls forceDeleteResourceCmd with pod.namespace ("default"), not filter "all"', async () => {
    mockInvoke.mockResolvedValue(undefined);

    // Force Delete is only visible when pod is Running or Pending
    render(
      <PodList
        pods={[mockPod]}
        clusterId="c1"
        namespace="all"
        onRefresh={() => {}}
      />
    );

    openActionMenu();
    fireEvent.click(screen.getByRole("button", { name: /^force delete$/i }));

    await waitFor(() => {
      expect(screen.getByTestId("confirm-delete")).toBeDefined();
    });

    fireEvent.click(screen.getByRole("button", { name: /confirm/i }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("force_delete_resource", {
        clusterId: "c1",
        namespace: "default",
        resourceType: "pods",
        resourceName: "test-pod",
      });
    });
  });

  it('Logs action opens a dock tab carrying pod.namespace ("default"), not filter "all"', async () => {
    useBottomPanelStore.setState({ tabs: [], activeTabId: null });

    render(
      <PodList
        pods={[mockPod]}
        clusterId="c1"
        namespace="all"
        onRefresh={() => {}}
      />
    );

    openActionMenu();
    fireEvent.click(screen.getByRole("button", { name: /^logs$/i }));

    await waitFor(() => {
      const tab = useBottomPanelStore
        .getState()
        .tabs.find((t) => t.type === BottomPanelTabType.POD_LOGS);
      expect(tab).toBeDefined();
      expect(tab?.data?.namespace).toBe("default");
      expect(tab?.data?.podName).toBe("test-pod");
    });
  });

  it('Shell modal receives pod.namespace ("default"), not filter "all"', async () => {
    render(
      <PodList
        pods={[mockPod]}
        clusterId="c1"
        namespace="all"
        onRefresh={() => {}}
      />
    );

    openActionMenu();
    fireEvent.click(screen.getByRole("button", { name: /^shell$/i }));

    await waitFor(() => {
      const modal = screen.getByTestId("shell-modal");
      expect(modal.getAttribute("data-namespace")).toBe("default");
    });
  });

  it('Attach modal receives pod.namespace ("default"), not filter "all"', async () => {
    render(
      <PodList
        pods={[mockPod]}
        clusterId="c1"
        namespace="all"
        onRefresh={() => {}}
      />
    );

    openActionMenu();
    fireEvent.click(screen.getByRole("button", { name: /^attach$/i }));

    await waitFor(() => {
      const modal = screen.getByTestId("attach-modal");
      expect(modal.getAttribute("data-namespace")).toBe("default");
    });
  });
});
