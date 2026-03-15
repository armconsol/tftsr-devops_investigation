import { describe, it, expect, beforeEach } from "vitest";
import { useSessionStore } from "@/stores/sessionStore";
import type { Issue, TriageMessage } from "@/lib/tauriCommands";

const mockIssue: Issue = {
  id: "issue-001",
  title: "Server not responding",
  description: "",
  severity: "p1",
  status: "open",
  category: "linux",
  source: "manual",
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
  resolved_at: undefined,
  assigned_to: "",
  tags: "[]",
};

const mockMessage: TriageMessage = {
  id: "msg-001",
  issue_id: "issue-001",
  role: "user",
  content: "The web server is returning 502 errors",
  why_level: 1,
  created_at: Date.now(),
};

describe("Session Store", () => {
  beforeEach(() => {
    useSessionStore.getState().reset();
  });

  it("has correct initial state", () => {
    const state = useSessionStore.getState();
    expect(state.currentIssue).toBeNull();
    expect(state.messages).toHaveLength(0);
    expect(state.currentWhyLevel).toBe(0);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it("starts a session with an issue", () => {
    useSessionStore.getState().startSession(mockIssue);
    const state = useSessionStore.getState();
    expect(state.currentIssue?.id).toBe("issue-001");
    expect(state.currentWhyLevel).toBe(1);
  });

  it("adds messages to the session", () => {
    useSessionStore.getState().addMessage(mockMessage);
    const state = useSessionStore.getState();
    expect(state.messages).toHaveLength(1);
    expect(state.messages[0].content).toBe("The web server is returning 502 errors");
  });

  it("updates why level", () => {
    useSessionStore.getState().setWhyLevel(3);
    expect(useSessionStore.getState().currentWhyLevel).toBe(3);
  });

  it("resets to initial state", () => {
    useSessionStore.getState().addMessage(mockMessage);
    useSessionStore.getState().reset();
    const state = useSessionStore.getState();
    expect(state.currentIssue).toBeNull();
    expect(state.messages).toHaveLength(0);
  });
});
