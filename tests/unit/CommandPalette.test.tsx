import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { CommandPalette } from "@/components/Kubernetes/CommandPalette";

describe("CommandPalette", () => {
  const defaultProps = {
    isOpen: true,
    onClose: vi.fn(),
    onNavigate: vi.fn(),
    clusterId: "cluster-1",
    namespace: "default",
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders nothing when isOpen is false", () => {
    render(<CommandPalette {...defaultProps} isOpen={false} />);
    expect(screen.queryByPlaceholderText(/command/i)).not.toBeInTheDocument();
  });

  it("renders search input when open", () => {
    render(<CommandPalette {...defaultProps} />);
    expect(screen.getByPlaceholderText(/type a command or search/i)).toBeInTheDocument();
  });

  it("shows the full command list when query is empty", () => {
    render(<CommandPalette {...defaultProps} />);
    expect(screen.getByText("Go to Pods")).toBeInTheDocument();
    expect(screen.getByText("Go to Deployments")).toBeInTheDocument();
    expect(screen.getByText("Go to Services")).toBeInTheDocument();
  });

  it("includes all expected navigation commands", () => {
    render(<CommandPalette {...defaultProps} />);
    const expectedLabels = [
      "Go to Overview",
      "Go to Pods",
      "Go to Deployments",
      "Go to Services",
      "Go to Nodes",
      "Go to Events",
      "Go to Config Maps",
      "Go to Secrets",
      "Go to PVCs",
      "Go to Ingresses",
      "Go to Port Forwarding",
      "Go to Roles",
    ];
    for (const label of expectedLabels) {
      expect(screen.getByText(label)).toBeInTheDocument();
    }
  });

  it("filters commands by query text", () => {
    render(<CommandPalette {...defaultProps} />);
    const input = screen.getByPlaceholderText(/type a command or search/i);
    fireEvent.change(input, { target: { value: "pods" } });
    expect(screen.getByText("Go to Pods")).toBeInTheDocument();
    expect(screen.queryByText("Go to Services")).not.toBeInTheDocument();
  });

  it("shows 'No commands found' when filter matches nothing", () => {
    render(<CommandPalette {...defaultProps} />);
    const input = screen.getByPlaceholderText(/type a command or search/i);
    fireEvent.change(input, { target: { value: "xyznonexistent" } });
    expect(screen.getByText(/no commands found/i)).toBeInTheDocument();
  });

  it("calls onNavigate with the correct target when a navigation command is clicked", async () => {
    const onNavigate = vi.fn();
    const onClose = vi.fn();
    render(<CommandPalette {...defaultProps} onNavigate={onNavigate} onClose={onClose} />);

    fireEvent.click(screen.getByText("Go to Pods"));

    await waitFor(() => {
      expect(onNavigate).toHaveBeenCalledWith("pods");
      expect(onClose).toHaveBeenCalled();
    });
  });

  it("calls onNavigate with 'deployments' when 'Go to Deployments' is clicked", async () => {
    const onNavigate = vi.fn();
    render(<CommandPalette {...defaultProps} onNavigate={onNavigate} />);

    fireEvent.click(screen.getByText("Go to Deployments"));

    await waitFor(() => {
      expect(onNavigate).toHaveBeenCalledWith("deployments");
    });
  });

  it("Close button calls onClose", () => {
    const onClose = vi.fn();
    render(<CommandPalette {...defaultProps} onClose={onClose} />);
    // The X button in the header
    const closeButtons = screen.getAllByRole("button");
    const xButton = closeButtons.find((btn) => btn.querySelector("svg"));
    expect(xButton).toBeDefined();
    fireEvent.click(xButton!);
    expect(onClose).toHaveBeenCalled();
  });

  it("pressing Escape calls onClose", () => {
    const onClose = vi.fn();
    render(<CommandPalette {...defaultProps} onClose={onClose} />);
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
  });

  it("pressing Enter on the first result invokes onNavigate with its target", async () => {
    const onNavigate = vi.fn();
    const onClose = vi.fn();
    render(<CommandPalette {...defaultProps} onNavigate={onNavigate} onClose={onClose} />);

    const input = screen.getByPlaceholderText(/type a command or search/i);
    // Filter to a single result
    fireEvent.change(input, { target: { value: "overview" } });

    await waitFor(() => {
      expect(screen.getByText("Go to Overview")).toBeInTheDocument();
    });

    fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() => {
      expect(onNavigate).toHaveBeenCalledWith("overview");
      expect(onClose).toHaveBeenCalled();
    });
  });

  it("shows category badge for Navigate commands", () => {
    render(<CommandPalette {...defaultProps} />);
    const badges = screen.getAllByText("Navigate");
    expect(badges.length).toBeGreaterThan(0);
  });
});
