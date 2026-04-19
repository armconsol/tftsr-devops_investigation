import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";

const mockInvoke = vi.mocked(invoke);

describe("Timeline Event Commands", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("addTimelineEventCmd calls invoke with correct params", async () => {
    const mockEvent = {
      id: "te-1",
      issue_id: "issue-1",
      event_type: "triage_started",
      description: "Started",
      metadata: "{}",
      created_at: "2025-01-15 10:00:00 UTC",
    };
    mockInvoke.mockResolvedValueOnce(mockEvent as never);

    const { addTimelineEventCmd } = await import("@/lib/tauriCommands");
    const result = await addTimelineEventCmd("issue-1", "triage_started", "Started");
    expect(mockInvoke).toHaveBeenCalledWith("add_timeline_event", {
      issueId: "issue-1",
      eventType: "triage_started",
      description: "Started",
      metadata: null,
    });
    expect(result).toEqual(mockEvent);
  });

  it("addTimelineEventCmd passes metadata when provided", async () => {
    mockInvoke.mockResolvedValueOnce({} as never);

    const { addTimelineEventCmd } = await import("@/lib/tauriCommands");
    await addTimelineEventCmd("issue-1", "log_uploaded", "File uploaded", '{"file":"app.log"}');
    expect(mockInvoke).toHaveBeenCalledWith("add_timeline_event", {
      issueId: "issue-1",
      eventType: "log_uploaded",
      description: "File uploaded",
      metadata: '{"file":"app.log"}',
    });
  });

  it("getTimelineEventsCmd calls invoke with correct params", async () => {
    mockInvoke.mockResolvedValueOnce([] as never);

    const { getTimelineEventsCmd } = await import("@/lib/tauriCommands");
    const result = await getTimelineEventsCmd("issue-1");
    expect(mockInvoke).toHaveBeenCalledWith("get_timeline_events", { issueId: "issue-1" });
    expect(result).toEqual([]);
  });
});
