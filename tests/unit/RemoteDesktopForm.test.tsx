import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ConnectionForm } from "../../src/pages/Remote/RemoteDesktopPage";

// The page module pulls in tauriCommands which calls @tauri-apps/api; stub it so
// the form can render in isolation.
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

// Resolve the Tabs content wrapper (root div carries the "mt-2" class and toggles
// block/hidden) that owns a uniquely-labelled child.
function contentOwning(text: string): HTMLElement {
  const el = screen.getByText(text).closest('div[class*="mt-2"]');
  if (!el) throw new Error(`No tab content wrapper found for: ${text}`);
  return el as HTMLElement;
}

describe("ConnectionForm tabs", () => {
  it("switches to the SSH Tunnel and Display tabs when their triggers are clicked", async () => {
    const user = userEvent.setup();
    render(
      <ConnectionForm
        title="Add Connection"
        onSave={vi.fn().mockResolvedValue(undefined)}
        onCancel={vi.fn()}
      />,
    );

    // Initially the Connection tab is active; SSH/Display content is hidden.
    expect(contentOwning("Enable SSH tunnel")).toHaveClass("hidden");
    expect(contentOwning("Auto-resize to window")).toHaveClass("hidden");

    // Clicking "SSH Tunnel" reveals the SSH content.
    await user.click(screen.getByRole("button", { name: "SSH Tunnel" }));
    expect(contentOwning("Enable SSH tunnel")).toHaveClass("block");

    // Clicking "Display" reveals the Display content.
    await user.click(screen.getByRole("button", { name: "Display" }));
    expect(contentOwning("Auto-resize to window")).toHaveClass("block");
    // ...and the SSH content is hidden again.
    expect(contentOwning("Enable SSH tunnel")).toHaveClass("hidden");
  });
});
