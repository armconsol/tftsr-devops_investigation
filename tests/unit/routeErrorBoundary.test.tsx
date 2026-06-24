import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route, NavLink } from "react-router-dom";
import { RouteErrorBoundary } from "@/components/RouteErrorBoundary";

const Boom = () => {
  throw new Error("page exploded");
};

const Safe = () => <div>safe page</div>;

/**
 * Simulates the App shell: a persistent navigation sidebar rendered OUTSIDE the
 * route error boundary, and the routed pages rendered INSIDE it. A crash in a
 * page must not unmount the navigation.
 */
function Shell({ initialPath }: { initialPath: string }) {
  return (
    <MemoryRouter initialEntries={[initialPath]}>
      <nav>
        <NavLink to="/safe">Go Safe</NavLink>
        <NavLink to="/boom">Go Boom</NavLink>
      </nav>
      <RouteErrorBoundary>
        <Routes>
          <Route path="/safe" element={<Safe />} />
          <Route path="/boom" element={<Boom />} />
        </Routes>
      </RouteErrorBoundary>
    </MemoryRouter>
  );
}

describe("RouteErrorBoundary", () => {
  it("renders the routed page when it does not throw", () => {
    render(<Shell initialPath="/safe" />);
    expect(screen.getByText("safe page")).toBeInTheDocument();
  });

  it("keeps navigation mounted when a page throws", () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    render(<Shell initialPath="/boom" />);
    // Page crashed -> fallback shown, but the nav must still be present.
    expect(screen.getByText(/This page failed to load/i)).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Go Safe" })).toBeInTheDocument();
    consoleError.mockRestore();
  });

  it("recovers automatically when navigating to another route after a crash", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    const user = userEvent.setup();
    render(<Shell initialPath="/boom" />);
    expect(screen.getByText(/This page failed to load/i)).toBeInTheDocument();

    // Navigating should reset the boundary (key changes on pathname) and render
    // the safe page without requiring an app restart.
    await user.click(screen.getByRole("link", { name: "Go Safe" }));
    expect(screen.getByText("safe page")).toBeInTheDocument();
    expect(screen.queryByText(/This page failed to load/i)).not.toBeInTheDocument();
    consoleError.mockRestore();
  });
});
