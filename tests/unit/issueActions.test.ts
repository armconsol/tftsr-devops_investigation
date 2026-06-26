import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";

const mockInvoke = vi.mocked(invoke);

// Helper to capture the args passed to invoke
function lastInvokeArgs() {
  const calls = mockInvoke.mock.calls;
  return calls[calls.length - 1];
}

describe("updateIssueCmd IPC args", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("passes updates as a nested object (not spread)", async () => {
    mockInvoke.mockResolvedValueOnce({} as never);
    const { updateIssueCmd } = await import("@/lib/tauriCommands");
    await updateIssueCmd("issue-1", { status: "resolved" });

    const [cmd, args] = lastInvokeArgs();
    expect(cmd).toBe("update_issue");
    // args must have 'issueId' and 'updates' as separate keys
    expect((args as Record<string, unknown>).issueId).toBe("issue-1");
    expect((args as Record<string, unknown>).updates).toEqual({ status: "resolved" });
    // must NOT have status at the top level
    expect((args as Record<string, unknown>).status).toBeUndefined();
  });
});

describe("addFiveWhyCmd IPC args", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("passes correct Rust parameter names", async () => {
    mockInvoke.mockResolvedValueOnce({} as never);
    const { addFiveWhyCmd } = await import("@/lib/tauriCommands");
    await addFiveWhyCmd("issue-1", 1, "Why did it fail?", "Network timeout", "");

    const [cmd, args] = lastInvokeArgs();
    expect(cmd).toBe("add_five_why");
    const a = args as Record<string, unknown>;
    expect(a.issueId).toBe("issue-1");
    expect(a.stepOrder).toBe(1);
    expect(a.whyQuestion).toBe("Why did it fail?");
    expect(a.answer).toBe("Network timeout");
    expect(a.evidence).toBe("");
  });
});

describe("getIssueMessagesCmd IPC args", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("calls get_issue_messages with issueId", async () => {
    mockInvoke.mockResolvedValueOnce([]);
    const { getIssueMessagesCmd } = await import("@/lib/tauriCommands");
    await getIssueMessagesCmd("issue-1");

    const [cmd, args] = lastInvokeArgs();
    expect(cmd).toBe("get_issue_messages");
    expect((args as Record<string, unknown>).issueId).toBe("issue-1");
  });
});

describe("close intent detection", () => {
  const closePatterns = [
    "close this issue",
    "please close",
    "mark as resolved",
    "mark resolved",
    "issue is fixed",
    "issue is resolved",
    "resolve this",
    "this is resolved",
  ];

  const notCloseMessages = [
    "why did the server close the connection",
    "the issue resolves around DNS",
    "how do I fix this",
  ];

  function isCloseIntent(message: string): boolean {
    const lower = message.toLowerCase();
    return closePatterns.some((p) => lower.includes(p));
  }

  it("detects close intent patterns", () => {
    expect(isCloseIntent("Please close this issue as it was a test")).toBe(true);
    expect(isCloseIntent("Mark as resolved")).toBe(true);
    expect(isCloseIntent("This issue is fixed now")).toBe(true);
    expect(isCloseIntent("issue is resolved")).toBe(true);
  });

  it("does not flag non-close messages as close intent", () => {
    for (const msg of notCloseMessages) {
      expect(isCloseIntent(msg)).toBe(false);
    }
  });
});
