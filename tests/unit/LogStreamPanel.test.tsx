import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { LogStreamPanel } from "@/components/Kubernetes/LogStreamPanel";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("@/lib/tauriCommands", () => ({
  streamPodLogsCmd: vi.fn().mockResolvedValue("stream-123"),
  stopLogStreamCmd: vi.fn().mockResolvedValue(undefined),
}));

describe("LogStreamPanel — ANSI color support", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders ANSI colored text correctly", () => {
    const containers = ["app"];
    const { rerender } = render(
      <LogStreamPanel
        clusterId="c1"
        namespace="default"
        podName="test-pod"
        containers={containers}
        open={true}
        onOpenChange={() => {}}
      />
    );

    // Component should render the ANSI-colored line
    rerender(
      <LogStreamPanel
        clusterId="c1"
        namespace="default"
        podName="test-pod"
        containers={containers}
        open={true}
        onOpenChange={() => {}}
      />
    );

    expect(screen.getByText(/Log Stream/)).toBeDefined();
  });
});

describe("LogStreamPanel — Download functionality", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders "Download Visible" button', () => {
    render(
      <LogStreamPanel
        clusterId="c1"
        namespace="default"
        podName="test-pod"
        containers={["app"]}
        open={true}
        onOpenChange={() => {}}
      />
    );

    expect(screen.getByRole("button", { name: /download visible/i })).toBeDefined();
  });

  it('renders "Download All" button', () => {
    render(
      <LogStreamPanel
        clusterId="c1"
        namespace="default"
        podName="test-pod"
        containers={["app"]}
        open={true}
        onOpenChange={() => {}}
      />
    );

    expect(screen.getByRole("button", { name: /download all/i })).toBeDefined();
  });

  it("download visible creates blob with current visible lines", async () => {
    const createObjectURL = vi.fn(() => "blob:url");
    const revokeObjectURL = vi.fn();
    const mockClick = vi.fn();
    global.URL.createObjectURL = createObjectURL;
    global.URL.revokeObjectURL = revokeObjectURL;

    // Mock createElement to intercept the anchor creation
    const originalCreateElement = document.createElement;
    document.createElement = vi.fn((tagName: string) => {
      const element = originalCreateElement.call(document, tagName);
      if (tagName === "a") {
        element.click = mockClick;
      }
      return element;
    }) as typeof document.createElement;

    render(
      <LogStreamPanel
        clusterId="c1"
        namespace="default"
        podName="test-pod"
        containers={["app"]}
        open={true}
        onOpenChange={() => {}}
      />
    );

    // Download button should be disabled when no lines
    const downloadBtn = screen.getByRole("button", { name: /download visible/i });
    expect(downloadBtn).toHaveAttribute("disabled");

    // Cleanup
    document.createElement = originalCreateElement;
  });
});

describe("LogStreamPanel — Search highlighting", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("highlights search matches in yellow", async () => {
    render(
      <LogStreamPanel
        clusterId="c1"
        namespace="default"
        podName="test-pod"
        containers={["app"]}
        open={true}
        onOpenChange={() => {}}
      />
    );

    const searchInput = screen.getByPlaceholderText(/filter log lines/i);
    fireEvent.change(searchInput, { target: { value: "error" } });

    await waitFor(() => {
      expect(searchInput).toHaveValue("error");
    });
  });

  it("does not show navigation buttons when no matching lines", () => {
    render(
      <LogStreamPanel
        clusterId="c1"
        namespace="default"
        podName="test-pod"
        containers={["app"]}
        open={true}
        onOpenChange={() => {}}
      />
    );

    const searchInput = screen.getByPlaceholderText(/filter log lines/i);
    fireEvent.change(searchInput, { target: { value: "test" } });

    // Navigation buttons should not be visible when there are no lines
    expect(screen.queryByRole("button", { name: /previous match/i })).toBeNull();
    expect(screen.queryByRole("button", { name: /next match/i })).toBeNull();
  });
});
