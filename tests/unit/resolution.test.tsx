import { describe, it, expect, beforeEach, vi } from "vitest";
import { render } from "@testing-library/react";
import { screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import Resolution from "@/pages/Resolution";
import * as tauriCommands from "@/lib/tauriCommands";

vi.mock("@/lib/tauriCommands");

const mockIssueDetail = {
  issue: {
    id: "test-id",
    title: "Test Issue",
    description: "Test Description",
    severity: "P3",
    status: "open",
    category: "linux",
    source: "manual",
    created_at: "2026-01-01",
    updated_at: "2026-01-01",
    assigned_to: "",
    tags: "[]",
  },
  log_files: [],
  image_attachments: [],
  resolution_steps: [
    {
      id: "step-1",
      issue_id: "test-id",
      step_order: 1,
      why_question: "Why did the service crash?",
      answer: "Out of memory",
      evidence: "dmesg shows OOM killer",
      created_at: "2026-01-01",
    },
  ],
  conversations: [],
  timeline_events: [],
};

describe("Resolution Page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(tauriCommands.getIssueCmd).mockResolvedValue(mockIssueDetail);
  });

  it("renders resolution steps with readable text", async () => {
    render(
      <MemoryRouter initialEntries={["/issue/test-id/resolution"]}>
        <Routes>
          <Route path="/issue/:id/resolution" element={<Resolution />} />
        </Routes>
      </MemoryRouter>
    );

    const question = await screen.findByText("Why did the service crash?");
    const answer = await screen.findByText("Out of memory");
    const evidence = await screen.findByText("dmesg shows OOM killer");

    // Check that text has proper contrast classes
    expect(question).toBeInTheDocument();
    expect(answer).toBeInTheDocument();
    expect(evidence).toBeInTheDocument();

    // Check for readable color classes (not muted-foreground which is too dark)
    const questionParent = question.closest("p");
    const answerParent = answer.closest("p");
    const evidenceParent = evidence.closest("p");

    expect(questionParent?.className).toContain("text-foreground");
    expect(answerParent?.className).toMatch(/text-foreground/);
    expect(evidenceParent?.className).toMatch(/text-foreground/);
  });
});
