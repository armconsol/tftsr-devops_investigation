import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { save } from "@tauri-apps/plugin-dialog";
import { LogsTab } from "@/components/dock/LogsTab";

type PodLogLineHandler = (event: { payload: { stream_id: string; line: string } }) => void;

// jsdom does not implement scrollIntoView; the component calls it in a
// follow-mode effect whenever new lines arrive.
Element.prototype.scrollIntoView = vi.fn();

let lastListenHandler: PodLogLineHandler | null = null;

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((_event: string, handler: PodLogLineHandler) => {
    lastListenHandler = handler;
    return Promise.resolve(() => {});
  }),
}));

vi.mock("@/lib/tauriCommands", () => ({
  streamPodLogsCmd: vi.fn().mockResolvedValue("stream-1"),
  stopLogStreamCmd: vi.fn().mockResolvedValue(undefined),
  listPodsCmd: vi.fn().mockResolvedValue([]),
  getPodLogsCmd: vi.fn().mockResolvedValue({ logs: "" }),
  saveLogFileCmd: vi.fn().mockResolvedValue(undefined),
}));

import {
  streamPodLogsCmd,
  getPodLogsCmd,
  saveLogFileCmd,
} from "@/lib/tauriCommands";

const mockSave = save as unknown as ReturnType<typeof vi.fn>;
const mockGetPodLogsCmd = getPodLogsCmd as unknown as ReturnType<typeof vi.fn>;
const mockSaveLogFileCmd = saveLogFileCmd as unknown as ReturnType<typeof vi.fn>;

async function startStreamAndEmit(lines: string[]) {
  fireEvent.click(screen.getByRole("button", { name: /^stream$/i }));
  await waitFor(() => expect(lastListenHandler).not.toBeNull());
  act(() => {
    for (const line of lines) {
      lastListenHandler!({ payload: { stream_id: "stream-1", line } });
    }
  });
}

const singlePodData = {
  clusterId: "c1",
  namespace: "default",
  podName: "test-pod",
  containers: ["app"],
};

describe("LogsTab — Download Visible", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    lastListenHandler = null;
    mockSave.mockResolvedValue(null);
  });

  it("is disabled when no lines have streamed in", () => {
    render(<LogsTab data={singlePodData} />);
    expect(screen.getByRole("button", { name: /download visible/i })).toBeDisabled();
  });

  it("saves only the filtered (visible) lines, not the full buffer", async () => {
    mockSave.mockResolvedValueOnce("/tmp/visible-logs.txt");
    render(<LogsTab data={singlePodData} />);

    await startStreamAndEmit(["error: boom", "info: fine", "error: again"]);

    fireEvent.change(screen.getByPlaceholderText(/filter log lines/i), {
      target: { value: "error" },
    });

    await waitFor(() =>
      expect(screen.getByRole("button", { name: /download visible/i })).not.toBeDisabled()
    );
    fireEvent.click(screen.getByRole("button", { name: /download visible/i }));

    await waitFor(() => expect(mockSaveLogFileCmd).toHaveBeenCalledTimes(1));
    expect(mockSave).toHaveBeenCalledWith(
      expect.objectContaining({ defaultPath: expect.stringContaining("visible-logs") })
    );
    expect(mockSaveLogFileCmd).toHaveBeenCalledWith(
      "/tmp/visible-logs.txt",
      "error: boom\nerror: again",
      "c1",
      "default",
      "test-pod",
      "app"
    );
  });

  it("does not save when the user cancels the save dialog", async () => {
    mockSave.mockResolvedValueOnce(null);
    render(<LogsTab data={singlePodData} />);

    await startStreamAndEmit(["line one"]);

    await waitFor(() =>
      expect(screen.getByRole("button", { name: /download visible/i })).not.toBeDisabled()
    );
    fireEvent.click(screen.getByRole("button", { name: /download visible/i }));

    await waitFor(() => expect(mockSave).toHaveBeenCalledTimes(1));
    expect(mockSaveLogFileCmd).not.toHaveBeenCalled();
  });

  it("surfaces an error banner when the save write fails", async () => {
    mockSave.mockResolvedValueOnce("/tmp/visible-logs.txt");
    mockSaveLogFileCmd.mockRejectedValueOnce(new Error("disk full"));
    render(<LogsTab data={singlePodData} />);

    await startStreamAndEmit(["line one"]);

    await waitFor(() =>
      expect(screen.getByRole("button", { name: /download visible/i })).not.toBeDisabled()
    );
    fireEvent.click(screen.getByRole("button", { name: /download visible/i }));

    await waitFor(() => expect(screen.getByText(/disk full/i)).toBeInTheDocument());
  });
});

describe("LogsTab — Download All", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    lastListenHandler = null;
    mockSave.mockResolvedValue(null);
  });

  it("is disabled when no pod/container is selected", () => {
    render(
      <LogsTab
        data={{
          clusterId: "c1",
          namespace: "default",
          workloadName: "my-deploy",
          workloadType: "Deployment",
        }}
      />
    );
    expect(screen.getByRole("button", { name: /download all/i })).toBeDisabled();
  });

  it("fetches the complete unfiltered log from the backend and saves it, ignoring the active filter", async () => {
    mockSave.mockResolvedValueOnce("/tmp/all-logs.txt");
    mockGetPodLogsCmd.mockResolvedValueOnce({
      logs: "line 1\nline 2\nline 3 (never streamed)",
    });
    render(<LogsTab data={singlePodData} />);

    await startStreamAndEmit(["line 1"]);

    fireEvent.change(screen.getByPlaceholderText(/filter log lines/i), {
      target: { value: "line 2" },
    });

    fireEvent.click(screen.getByRole("button", { name: /download all/i }));

    await waitFor(() => expect(mockGetPodLogsCmd).toHaveBeenCalledWith("c1", "default", "test-pod", "app"));
    await waitFor(() => expect(mockSaveLogFileCmd).toHaveBeenCalledTimes(1));
    expect(mockSave).toHaveBeenCalledWith(
      expect.objectContaining({ defaultPath: expect.stringContaining("all-logs") })
    );
    expect(mockSaveLogFileCmd).toHaveBeenCalledWith(
      "/tmp/all-logs.txt",
      "line 1\nline 2\nline 3 (never streamed)",
      "c1",
      "default",
      "test-pod",
      "app"
    );
  });

  it("does not save when the user cancels the save dialog", async () => {
    mockSave.mockResolvedValueOnce(null);
    render(<LogsTab data={singlePodData} />);

    fireEvent.click(screen.getByRole("button", { name: /download all/i }));

    await waitFor(() => expect(mockSave).toHaveBeenCalledTimes(1));
    expect(mockGetPodLogsCmd).not.toHaveBeenCalled();
    expect(mockSaveLogFileCmd).not.toHaveBeenCalled();
  });
});

describe("LogsTab — streamPodLogsCmd sanity", () => {
  it("still starts a stream normally (regression guard)", async () => {
    render(<LogsTab data={singlePodData} />);
    fireEvent.click(screen.getByRole("button", { name: /^stream$/i }));
    await waitFor(() => expect(streamPodLogsCmd).toHaveBeenCalledTimes(1));
  });
});
