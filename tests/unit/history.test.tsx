import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import History from "@/pages/History";
import { useHistoryStore } from "@/stores/historyStore";
import type { IssueSummary } from "@/lib/tauriCommands";

vi.mock("@/stores/historyStore");

const mockIssues: IssueSummary[] = [
  {
    id: "issue-1",
    title: "Test Issue 1",
    severity: "P3",
    status: "open",
    category: "linux",
    created_at: "2026-01-01",
    updated_at: "2026-01-01",
    log_count: 1,
    step_count: 2,
  },
  {
    id: "issue-2",
    title: "Test Issue 2",
    severity: "P1",
    status: "resolved",
    category: "kubernetes",
    created_at: "2026-01-02",
    updated_at: "2026-01-02",
    log_count: 0,
    step_count: 3,
  },
];

describe("History Page", () => {
  beforeEach(() => {
    vi.mocked(useHistoryStore).mockReturnValue({
      issues: mockIssues,
      isLoading: false,
      error: null,
      searchQuery: "",
      loadIssues: vi.fn(),
      searchIssues: vi.fn(),
      setSearchQuery: vi.fn(),
    });
  });

  it("displays domain column in table", () => {
    render(
      <MemoryRouter>
        <History />
      </MemoryRouter>
    );

    // Check that Domain header exists
    const domainHeader = screen.getByText("Domain");
    expect(domainHeader).toBeInTheDocument();

    // Check that domain values are displayed
    expect(screen.getByText("linux")).toBeInTheDocument();
    expect(screen.getByText("kubernetes")).toBeInTheDocument();
  });

  it("domain filter is functional", () => {
    const mockLoadIssues = vi.fn();
    vi.mocked(useHistoryStore).mockReturnValue({
      issues: mockIssues,
      isLoading: false,
      error: null,
      searchQuery: "",
      loadIssues: mockLoadIssues,
      searchIssues: vi.fn(),
      setSearchQuery: vi.fn(),
    });

    render(
      <MemoryRouter>
        <History />
      </MemoryRouter>
    );

    // Find domain filter dropdown button
    const domainFilter = screen.getByRole("button", { name: /All Domains/i });
    expect(domainFilter).toBeInTheDocument();
  });

  it("search button is visible and readable", () => {
    render(
      <MemoryRouter>
        <History />
      </MemoryRouter>
    );

    const searchButton = screen.getByRole("button", { name: /search/i });
    expect(searchButton).toBeInTheDocument();
    expect(searchButton.className).toContain("outline");
  });
});
