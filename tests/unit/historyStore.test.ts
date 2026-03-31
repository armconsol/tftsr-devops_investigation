import { describe, it, expect, beforeEach, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useHistoryStore } from "@/stores/historyStore";
import type { IssueSummary } from "@/lib/tauriCommands";

const mockInvoke = vi.mocked(invoke);

const makeIssue = (overrides: Partial<IssueSummary> = {}): IssueSummary => ({
  id: "issue-001",
  title: "Test issue",
  severity: "P3",
  status: "open",
  category: "linux",
  log_count: 0,
  step_count: 0,
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
  ...overrides,
});

describe("History Store", () => {
  beforeEach(() => {
    useHistoryStore.setState({ issues: [], isLoading: false, error: null, searchQuery: "" });
    mockInvoke.mockReset();
  });

  it("loadIssues populates the issues array", async () => {
    const issues = [makeIssue(), makeIssue({ id: "issue-002", title: "Another issue" })];
    mockInvoke.mockResolvedValueOnce(issues);

    await useHistoryStore.getState().loadIssues();

    expect(useHistoryStore.getState().issues).toHaveLength(2);
    expect(useHistoryStore.getState().isLoading).toBe(false);
  });

  it("loadIssues sets error on failure and clears isLoading", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("DB locked"));

    await useHistoryStore.getState().loadIssues();

    expect(useHistoryStore.getState().error).toContain("DB locked");
    expect(useHistoryStore.getState().isLoading).toBe(false);
  });

  it("isLoading is true while fetching (stat cards must show — not 0)", async () => {
    let resolve!: (v: unknown) => void;
    mockInvoke.mockReturnValueOnce(new Promise((r) => (resolve = r)));

    const p = useHistoryStore.getState().loadIssues();
    expect(useHistoryStore.getState().isLoading).toBe(true);

    resolve([makeIssue()]);
    await p;
    expect(useHistoryStore.getState().isLoading).toBe(false);
    expect(useHistoryStore.getState().issues).toHaveLength(1);
  });

  it("open issue count includes status=open and status=triaging", () => {
    useHistoryStore.setState({
      issues: [
        makeIssue({ status: "open" }),
        makeIssue({ id: "002", status: "triaging" }),
        makeIssue({ id: "003", status: "resolved" }),
        makeIssue({ id: "004", status: "open" }),
      ],
    });

    const { issues } = useHistoryStore.getState();
    const openCount = issues.filter(
      (i) => i.status === "open" || i.status === "triaging"
    ).length;

    expect(openCount).toBe(3);
  });

  it("newly created issue with status=open is counted as active", () => {
    useHistoryStore.setState({ issues: [makeIssue({ status: "open" })] });

    const { issues } = useHistoryStore.getState();
    const openCount = issues.filter(
      (i) => i.status === "open" || i.status === "triaging"
    ).length;

    expect(openCount).toBe(1);
  });

  it("resolved issues are not counted as active", () => {
    useHistoryStore.setState({
      issues: [makeIssue({ status: "resolved" }), makeIssue({ id: "002", status: "resolved" })],
    });

    const { issues } = useHistoryStore.getState();
    const openCount = issues.filter(
      (i) => i.status === "open" || i.status === "triaging"
    ).length;

    expect(openCount).toBe(0);
  });

  it("issue category field is present (not domain)", () => {
    const issue = makeIssue({ category: "kubernetes" });
    useHistoryStore.setState({ issues: [issue] });

    const stored = useHistoryStore.getState().issues[0];
    expect(stored.category).toBe("kubernetes");
  });
});
