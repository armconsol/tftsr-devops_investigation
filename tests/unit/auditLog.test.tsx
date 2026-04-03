import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import Security from "@/pages/Settings/Security";
import * as tauriCommands from "@/lib/tauriCommands";

vi.mock("@/lib/tauriCommands");

const mockAuditEntries: tauriCommands.AuditEntry[] = [
  {
    id: "audit-1",
    timestamp: "2026-04-02T10:00:00Z",
    action: "generate_rca",
    entity_type: "document",
    entity_id: "doc-123",
    user_id: "user-1",
    details: JSON.stringify({
      issue_id: "issue-456",
      content_hash: "abc123",
      data: "Sample RCA content"
    }),
  },
  {
    id: "audit-2",
    timestamp: "2026-04-02T11:00:00Z",
    action: "ai_chat",
    entity_type: "conversation",
    entity_id: "conv-789",
    user_id: "user-1",
    details: JSON.stringify({
      message: "What caused the issue?",
      response_hash: "def456"
    }),
  },
];

describe("Audit Log", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(tauriCommands.getAuditLogCmd).mockResolvedValue(mockAuditEntries);
  });

  it("displays audit entries", async () => {
    render(<Security />);

    // Wait for audit log to load
    await screen.findByText("Audit Log");

    // Check that the table has rows (header + data rows)
    const table = screen.getByRole("table");
    expect(table).toBeInTheDocument();

    const rows = screen.getAllByRole("row");
    expect(rows.length).toBeGreaterThan(1); // At least header + 1 data row
  });

  it("provides way to view transmitted data details", async () => {
    render(<Security />);

    await screen.findByText("Audit Log");

    // Should have View/Hide buttons for expanding details
    const viewButtons = await screen.findAllByRole("button", { name: /View/i });
    expect(viewButtons.length).toBeGreaterThan(0);
  });

  it("details column or button exists for viewing data", async () => {
    render(<Security />);

    await screen.findByText("Audit Log");

    // The audit log should have a Details column header
    const detailsHeader = screen.getByText("Details");
    expect(detailsHeader).toBeInTheDocument();

    // Should have view buttons
    const viewButtons = await screen.findAllByRole("button", { name: /View/i });
    expect(viewButtons.length).toBe(2); // One for each mock entry
  });
});
