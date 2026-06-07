import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { CreateResourceModal } from "@/components/Kubernetes/CreateResourceModal";

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
  mockReturnValue: (v: unknown) => void;
};

describe("CreateResourceModal", () => {
  const defaultProps = {
    isOpen: true,
    clusterId: "cluster-1",
    namespace: "default",
    onClose: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the Form tab and YAML tab", () => {
    render(<CreateResourceModal {...defaultProps} />);
    expect(screen.getByRole("button", { name: /^form$/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /^yaml$/i })).toBeInTheDocument();
  });

  it("resource type dropdown has expected options", async () => {
    render(<CreateResourceModal {...defaultProps} />);
    // The Select trigger shows the current value "pod" as its accessible name
    const trigger = screen.getAllByRole("button").find(
      (btn) => btn.textContent?.toLowerCase().includes("pod") && !btn.textContent?.toLowerCase().includes("yaml")
    );
    expect(trigger).toBeDefined();
    fireEvent.click(trigger!);
    await waitFor(() => {
      // After opening the dropdown, all items should be in the document
      expect(screen.getByText("Deployment")).toBeInTheDocument();
      expect(screen.getByText("Service")).toBeInTheDocument();
    });
  });

  it("Apply button calls createResourceCmd with correct args from YAML tab", async () => {
    (invoke as MockedInvoke).mockResolvedValue(undefined);

    render(<CreateResourceModal {...defaultProps} />);

    // Switch to YAML tab
    fireEvent.click(screen.getByRole("button", { name: /^yaml$/i }));

    // Type YAML content
    const editor = await screen.findByTestId("monaco-editor");
    fireEvent.change(editor, { target: { value: "apiVersion: v1\nkind: Pod" } });

    fireEvent.click(screen.getByRole("button", { name: /create resource/i }));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("create_resource", {
        clusterId: "cluster-1",
        namespace: "default",
        resourceType: "pod",
        yamlContent: "apiVersion: v1\nkind: Pod",
      });
    });
  });

  it("shows error message when IPC call fails", async () => {
    (invoke as MockedInvoke).mockRejectedValue(new Error("cluster unreachable"));

    render(<CreateResourceModal {...defaultProps} />);

    // Switch to YAML tab so we skip the "name required" guard
    fireEvent.click(screen.getByRole("button", { name: /^yaml$/i }));
    fireEvent.change(screen.getByTestId("monaco-editor"), {
      target: { value: "kind: Pod" },
    });

    fireEvent.click(screen.getByRole("button", { name: /create resource/i }));

    await waitFor(() => {
      expect(screen.getByText(/cluster unreachable/i)).toBeInTheDocument();
    });
  });

  it("calls onClose after successful IPC call", async () => {
    (invoke as MockedInvoke).mockResolvedValue(undefined);
    const onClose = vi.fn();

    render(<CreateResourceModal {...defaultProps} onClose={onClose} />);

    fireEvent.click(screen.getByRole("button", { name: /^yaml$/i }));
    fireEvent.change(screen.getByTestId("monaco-editor"), {
      target: { value: "kind: Pod" },
    });
    fireEvent.click(screen.getByRole("button", { name: /create resource/i }));

    await waitFor(() => {
      expect(onClose).toHaveBeenCalledTimes(1);
    });
  });

  it("shows loading state during IPC call", async () => {
    let resolveIpc!: () => void;
    (invoke as MockedInvoke).mockReturnValue(
      new Promise<void>((res) => {
        resolveIpc = res;
      })
    );

    render(<CreateResourceModal {...defaultProps} />);

    fireEvent.click(screen.getByRole("button", { name: /^yaml$/i }));
    fireEvent.change(screen.getByTestId("monaco-editor"), {
      target: { value: "kind: Pod" },
    });
    fireEvent.click(screen.getByRole("button", { name: /create resource/i }));

    // Button should be disabled while pending
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: /creating|create resource/i })
      ).toBeDisabled();
    });

    resolveIpc();
  });
});
