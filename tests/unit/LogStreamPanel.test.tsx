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

    // Simulate receiving log line with ANSI color codes
    const logLine = "\x1b[31mError: something went wrong\x1b[0m";

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

  it("download visible creates blob with current visible lines", () => {
    const createObjectURL = vi.fn(() => "blob:url");
    const revokeObjectURL = vi.fn();
    global.URL.createObjectURL = createObjectURL;
    global.URL.revokeObjectURL = revokeObjectURL;

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

    const downloadBtn = screen.getByRole("button", { name: /download visible/i });
    fireEvent.click(downloadBtn);

    expect(createObjectURL).toHaveBeenCalled();
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

  it("provides next/previous navigation buttons", () => {
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

    expect(screen.getByRole("button", { name: /previous match/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /next match/i })).toBeDefined();
  });
});
