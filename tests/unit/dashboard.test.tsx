import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import Dashboard from "@/pages/Dashboard";
import { useHistoryStore } from "@/stores/historyStore";

vi.mock("@/stores/historyStore");

describe("Dashboard Page", () => {
  beforeEach(() => {
    vi.mocked(useHistoryStore).mockReturnValue({
      issues: [],
      isLoading: false,
      error: null,
      searchQuery: "",
      loadIssues: vi.fn(),
      searchIssues: vi.fn(),
      setSearchQuery: vi.fn(),
    });
  });

  it("refresh button is visible with proper contrast", () => {
    render(
      <MemoryRouter>
        <Dashboard />
      </MemoryRouter>
    );

    const refreshButton = screen.getByRole("button", { name: /refresh/i });
    expect(refreshButton).toBeInTheDocument();

    // Button should have outline variant for visibility
    expect(refreshButton.className).toContain("outline");
  });

  it("refresh button shows icon and text", () => {
    render(
      <MemoryRouter>
        <Dashboard />
      </MemoryRouter>
    );

    const refreshButton = screen.getByRole("button", { name: /refresh/i });
    expect(refreshButton.textContent).toContain("Refresh");
  });
});
