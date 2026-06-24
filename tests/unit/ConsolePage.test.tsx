import React from "react";
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { ProxmoxConsolePage } from "@/pages/Proxmox/ConsolePage";

// The console renderer calls invoke() on mount; ensure it is NOT reached for
// an invalid VM id.
const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

function renderAt(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <Routes>
        <Route
          path="/proxmox/console/:clusterId/:node/:vmid/:kind"
          element={<ProxmoxConsolePage />}
        />
      </Routes>
    </MemoryRouter>
  );
}

describe("ProxmoxConsolePage — vmid validation", () => {
  it("shows an error and does not open a console for a non-numeric vmid", () => {
    renderAt("/proxmox/console/c1/node1/not-a-number/qemu");
    expect(screen.getByText(/invalid vm id/i)).toBeInTheDocument();
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("shows an error for a zero vmid", () => {
    renderAt("/proxmox/console/c1/node1/0/qemu");
    expect(screen.getByText(/invalid vm id/i)).toBeInTheDocument();
  });
});
