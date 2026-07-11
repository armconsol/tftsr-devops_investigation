import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { Updater } from "@/pages/Settings/Updater";
import { checkAppUpdatesCmd } from "@/lib/tauriCommands";

vi.mock("@/lib/tauriCommands", async () => {
  const actual = await vi.importActual<typeof import("@/lib/tauriCommands")>("@/lib/tauriCommands");
  return {
    ...actual,
    checkAppUpdatesCmd: vi.fn(),
    installAppUpdatesCmd: vi.fn(),
  };
});

const mockCheck = checkAppUpdatesCmd as unknown as ReturnType<typeof vi.fn>;

beforeEach(() => {
  mockCheck.mockReset();
  mockCheck.mockResolvedValue({
    updateAvailable: false,
    currentVersion: "3.1.0",
    latestVersion: "3.1.0",
    releaseUrl: "https://example.com",
    releaseNotes: "",
  });
});

describe("Updater — no update channel UI", () => {
  it("does not render an Update Channel section", async () => {
    render(<Updater />);
    await waitFor(() => expect(screen.getByText("Up to Date")).toBeInTheDocument());
    expect(screen.queryByText("Update Channel")).not.toBeInTheDocument();
    expect(screen.queryByText("Stable")).not.toBeInTheDocument();
    expect(screen.queryByText("Pre-Release")).not.toBeInTheDocument();
  });

  it("still shows current/latest version and the Check Now control", async () => {
    render(<Updater />);
    await waitFor(() => expect(screen.getAllByText(/3\.1\.0/).length).toBeGreaterThan(0));
    expect(screen.getByRole("button", { name: /check now/i })).toBeInTheDocument();
  });
});
