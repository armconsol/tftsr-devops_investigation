import { describe, it, expect, beforeEach, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useAttachmentStore } from "@/stores/attachmentStore";
import type { LogFileSummary, ImageAttachmentSummary } from "@/lib/tauriCommands";

const mockInvoke = vi.mocked(invoke);

const makeLogFile = (overrides: Partial<LogFileSummary> = {}): LogFileSummary => ({
  id: "lf-001",
  issue_id: "issue-001",
  issue_title: "Disk Full Alert",
  file_name: "syslog.log",
  filePath: "/var/log/syslog",
  file_size: 2048,
  mime_type: "text/plain",
  content_hash: "abc123",
  uploaded_at: new Date().toISOString(),
  redacted: false,
  ...overrides,
});

const makeImage = (overrides: Partial<ImageAttachmentSummary> = {}): ImageAttachmentSummary => ({
  id: "img-001",
  issue_id: "issue-001",
  issue_title: "Disk Full Alert",
  file_name: "screenshot.png",
  filePath: "/tmp/screenshot.png",
  file_size: 51200,
  mime_type: "image/png",
  upload_hash: "def456",
  uploaded_at: new Date().toISOString(),
  pii_warning_acknowledged: true,
  is_paste: false,
  ...overrides,
});

const resetStore = () => {
  useAttachmentStore.setState({
    logFiles: [],
    images: [],
    isLoading: false,
    error: null,
    searchQuery: "",
  });
};

describe("Attachment Store", () => {
  beforeEach(() => {
    resetStore();
    mockInvoke.mockReset();
  });

  // ─── loadAttachments ──────────────────────────────────────────────────────

  it("loadAttachments populates logFiles and images", async () => {
    const logFiles = [makeLogFile()];
    const images = [makeImage()];
    mockInvoke
      .mockResolvedValueOnce(logFiles) // list_all_log_files
      .mockResolvedValueOnce(images); // list_all_image_attachments

    await useAttachmentStore.getState().loadAttachments();

    expect(useAttachmentStore.getState().logFiles).toHaveLength(1);
    expect(useAttachmentStore.getState().images).toHaveLength(1);
    expect(useAttachmentStore.getState().isLoading).toBe(false);
    expect(useAttachmentStore.getState().error).toBeNull();
  });

  it("loadAttachments sets isLoading=true while in flight", async () => {
    let resolveLog!: (v: unknown) => void;
    mockInvoke.mockReturnValueOnce(new Promise((r) => (resolveLog = r)));
    mockInvoke.mockResolvedValueOnce([]);

    const p = useAttachmentStore.getState().loadAttachments();
    expect(useAttachmentStore.getState().isLoading).toBe(true);

    resolveLog([]);
    await p;
    expect(useAttachmentStore.getState().isLoading).toBe(false);
  });

  it("loadAttachments sets error on failure and clears isLoading", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("DB error"));

    await useAttachmentStore.getState().loadAttachments();

    expect(useAttachmentStore.getState().error).toContain("DB error");
    expect(useAttachmentStore.getState().isLoading).toBe(false);
  });

  it("loadAttachments passes issueId filter when provided", async () => {
    mockInvoke.mockResolvedValueOnce([makeLogFile()]).mockResolvedValueOnce([]);

    await useAttachmentStore.getState().loadAttachments({ issueId: "issue-001" });

    // Both calls should have been made with issueId
    const logCall = mockInvoke.mock.calls.find((c) => c[0] === "list_all_log_files");
    expect(logCall).toBeDefined();
    expect(logCall![1]).toMatchObject({ issueId: "issue-001" });
  });

  // ─── searchAttachments ────────────────────────────────────────────────────

  it("searchAttachments passes search query and updates store", async () => {
    const logFiles = [makeLogFile({ file_name: "app.log" })];
    mockInvoke
      .mockResolvedValueOnce(logFiles)
      .mockResolvedValueOnce([]);

    await useAttachmentStore.getState().searchAttachments("app");

    expect(useAttachmentStore.getState().searchQuery).toBe("app");
    expect(useAttachmentStore.getState().logFiles).toHaveLength(1);

    const logCall = mockInvoke.mock.calls.find((c) => c[0] === "list_all_log_files");
    expect(logCall![1]).toMatchObject({ search: "app" });
  });

  it("searchAttachments with empty string clears filter", async () => {
    mockInvoke.mockResolvedValueOnce([]).mockResolvedValueOnce([]);

    await useAttachmentStore.getState().searchAttachments("");

    const logCall = mockInvoke.mock.calls.find((c) => c[0] === "list_all_log_files");
    expect(logCall![1]).toMatchObject({ search: null });
  });

  // ─── setSearchQuery ───────────────────────────────────────────────────────

  it("setSearchQuery updates searchQuery without triggering a fetch", () => {
    useAttachmentStore.getState().setSearchQuery("kernel panic");
    expect(useAttachmentStore.getState().searchQuery).toBe("kernel panic");
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  // ─── data shape ───────────────────────────────────────────────────────────

  it("log file summary includes issue_title", async () => {
    mockInvoke
      .mockResolvedValueOnce([makeLogFile({ issue_title: "Memory Leak Incident" })])
      .mockResolvedValueOnce([]);

    await useAttachmentStore.getState().loadAttachments();

    expect(useAttachmentStore.getState().logFiles[0].issue_title).toBe("Memory Leak Incident");
  });

  it("image summary includes issue_title and is_paste flag", async () => {
    mockInvoke
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([makeImage({ issue_title: "CPU Spike", is_paste: true })]);

    await useAttachmentStore.getState().loadAttachments();

    const img = useAttachmentStore.getState().images[0];
    expect(img.issue_title).toBe("CPU Spike");
    expect(img.is_paste).toBe(true);
  });
});
