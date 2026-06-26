import { describe, it, expect, beforeEach, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { exportDocumentCmd } from "@/lib/tauriCommands";

vi.mock("@tauri-apps/api/core");

describe("exportDocumentCmd", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("includes title parameter when calling backend", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue("/path/to/doc.pdf");

    const docId = "doc-123";
    const title = "Test Document";
    const contentMd = "# Test Content";
    const format = "pdf";
    const outputDir = ".";

    await exportDocumentCmd(docId, title, contentMd, format, outputDir);

    // Check that invoke was called with the correct parameters
    expect(mockInvoke).toHaveBeenCalledWith(
      "export_document",
      expect.objectContaining({
        title,
        contentMd,
        format,
        outputDir,
      })
    );
  });

  it("handles missing title gracefully", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockRejectedValue("missing required key title");

    const docId = "doc-123";
    const title = "";
    const contentMd = "# Test";
    const format = "pdf";
    const outputDir = ".";

    await expect(exportDocumentCmd(docId, title, contentMd, format, outputDir)).rejects.toMatch(/title/i);
  });
});
