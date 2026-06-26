import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/react";
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

    // Wait for table to appear after async audit data loads
    const table = await screen.findByRole("table");
    expect(table).toBeInTheDocument();

    const rows = screen.getAllByRole("row");
    expect(rows.length).toBeGreaterThan(1); // At least header + 1 data row
  });

  it("provides way to view transmitted data details", async () => {
    render(<Security />);

    // Wait for async data to load and render the table
    const viewButtons = await screen.findAllByRole("button", { name: /View/i });
    expect(viewButtons.length).toBeGreaterThan(0);
  });

  it("details column or button exists for viewing data", async () => {
    render(<Security />);

    // Wait for async data to load and render the table
    await screen.findByRole("table");

    const detailsHeader = screen.getByText("Details");
    expect(detailsHeader).toBeInTheDocument();

    const viewButtons = screen.getAllByRole("button", { name: /View/i });
    expect(viewButtons.length).toBe(2); // One for each mock entry
  });
});
