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

  it("does not submit the form (close the dialog) when switching tabs in edit mode", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn().mockResolvedValue(undefined);
    const onCancel = vi.fn();

    // Edit mode with all required fields pre-filled: password is optional here, so a
    // stray form submission would pass validation and close the dialog.
    render(
      <ConnectionForm
        title="Edit Connection"
        isEdit
        initial={{ name: "My Server", hostname: "192.168.1.10" }}
        onSave={onSave}
        onCancel={onCancel}
      />,
    );

    await user.click(screen.getByRole("button", { name: "SSH Tunnel" }));
    await user.click(screen.getByRole("button", { name: "Display" }));

    // Switching tabs must never trigger a save/submit.
    expect(onSave).not.toHaveBeenCalled();
  });
});
