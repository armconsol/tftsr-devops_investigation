import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { EditResourceModal } from "@/components/Kubernetes/EditResourceModal";

vi.mock("@monaco-editor/react", () => ({
  default: ({
    value,
    onChange,
  }: {
    value?: string;
    onChange?: (v: string | undefined) => void;
  }) => (
    <textarea
      data-testid="monaco-editor"
      value={value ?? ""}
      onChange={(e) => onChange?.(e.target.value)}
    />
  ),
}));

type MockedInvoke = typeof invoke & {
  mockResolvedValue: (v: unknown) => void;
  mockRejectedValue: (e: Error) => void;
};

describe("EditResourceModal", () => {
  const defaultProps = {
    isOpen: true,
    clusterId: "cluster-1",
    namespace: "default",
    resourceType: "deployment",
    resourceName: "nginx",
    initialYaml: "apiVersion: apps/v1\nkind: Deployment",
    onClose: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders with initial YAML content in the editor", async () => {
    render(<EditResourceModal {...defaultProps} />);

    // The YAML tab should load with initialYaml
    fireEvent.click(screen.getByRole("button", { name: /^yaml$/i }));

    const editor = await screen.findByTestId("monaco-editor") as HTMLTextAreaElement;
    expect(editor.value).toBe("apiVersion: apps/v1\nkind: Deployment");
  });

  it("Apply button calls editResourceCmd with correct args", async () => {
    (invoke as MockedInvoke).mockResolvedValue(undefined);

    render(<EditResourceModal {...defaultProps} />);

    fireEvent.click(screen.getByRole("button", { name: /^yaml$/i }));
    const editor = await screen.findByTestId("monaco-editor");
    fireEvent.change(editor, {
      target: { value: "apiVersion: apps/v1\nkind: Deployment\nreplicas: 3" },
    });

    fireEvent.click(screen.getByRole("button", { name: /save changes/i }));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("edit_resource", {
        clusterId: "cluster-1",
        namespace: "default",
        resourceType: "deployment",
        resourceName: "nginx",
        yamlContent: "apiVersion: apps/v1\nkind: Deployment\nreplicas: 3",
      });
    });
  });

  it("shows error message when IPC call fails", async () => {
    (invoke as MockedInvoke).mockRejectedValue(new Error("resource not found"));

    render(<EditResourceModal {...defaultProps} />);

    fireEvent.click(screen.getByRole("button", { name: /^yaml$/i }));
    const editor = await screen.findByTestId("monaco-editor");
    fireEvent.change(editor, { target: { value: "kind: Deployment" } });

    fireEvent.click(screen.getByRole("button", { name: /save changes/i }));

    await waitFor(() => {
      expect(screen.getByText(/resource not found/i)).toBeInTheDocument();
    });
  });

  it("calls onClose after successful IPC call", async () => {
    (invoke as MockedInvoke).mockResolvedValue(undefined);
    const onClose = vi.fn();

    render(<EditResourceModal {...defaultProps} onClose={onClose} />);

    fireEvent.click(screen.getByRole("button", { name: /^yaml$/i }));
    const editor = await screen.findByTestId("monaco-editor");
    fireEvent.change(editor, { target: { value: "kind: Deployment" } });

    fireEvent.click(screen.getByRole("button", { name: /save changes/i }));

    await waitFor(() => {
      expect(onClose).toHaveBeenCalledTimes(1);
    });
  });
});
