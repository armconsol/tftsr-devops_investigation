import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import type { PiiDetectionResult } from "@/lib/tauriCommands";

describe("PII Detection Commands", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("detects IPv4 addresses in log text", async () => {
    const mockResult: PiiDetectionResult = {
      log_file_id: "log-001",
      detections: [
        {
          id: "span-1",
          pii_type: "IPv4",
          start: 13,
          end: 24,
          original: "192.168.1.1",
          replacement: "[IPv4]",
        },
      ],
      total_pii_found: 1,
    };
    vi.mocked(invoke).mockResolvedValueOnce(mockResult);

    const { detectPiiCmd } = await import("@/lib/tauriCommands");
    const result = await detectPiiCmd("log-001");

    expect(invoke).toHaveBeenCalledWith("detect_pii", { logFileId: "log-001" });
    expect(result.detections).toHaveLength(1);
    expect(result.detections[0].pii_type).toBe("IPv4");
    expect(result.detections[0].original).toBe("192.168.1.1");
    expect(result.detections[0].replacement).toBe("[IPv4]");
  });

  it("detects email addresses", async () => {
    const mockResult: PiiDetectionResult = {
      log_file_id: "log-002",
      detections: [
        {
          id: "span-2",
          pii_type: "Email",
          start: 5,
          end: 22,
          original: "admin@example.com",
          replacement: "[EMAIL]",
        },
      ],
      total_pii_found: 1,
    };
    vi.mocked(invoke).mockResolvedValueOnce(mockResult);

    const { detectPiiCmd } = await import("@/lib/tauriCommands");
    const result = await detectPiiCmd("log-002");

    expect(result.detections[0].pii_type).toBe("Email");
  });

  it("detects Bearer tokens", async () => {
    const mockResult: PiiDetectionResult = {
      log_file_id: "log-003",
      detections: [
        {
          id: "span-3",
          pii_type: "Bearer",
          start: 15,
          end: 60,
          original: "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9",
          replacement: "[BEARER_TOKEN]",
        },
      ],
      total_pii_found: 1,
    };
    vi.mocked(invoke).mockResolvedValueOnce(mockResult);

    const { detectPiiCmd } = await import("@/lib/tauriCommands");
    const result = await detectPiiCmd("log-003");

    expect(result.detections[0].pii_type).toBe("Bearer");
  });

  it("returns empty detections for clean logs", async () => {
    const mockResult: PiiDetectionResult = {
      log_file_id: "log-004",
      detections: [],
      total_pii_found: 0,
    };
    vi.mocked(invoke).mockResolvedValueOnce(mockResult);

    const { detectPiiCmd } = await import("@/lib/tauriCommands");
    const result = await detectPiiCmd("log-004");

    expect(result.detections).toHaveLength(0);
    expect(result.total_pii_found).toBe(0);
  });
});
