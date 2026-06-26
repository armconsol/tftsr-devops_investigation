import React from "react";
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { RemotesList } from "@/components/Proxmox/RemotesList";

const remote = {
  id: "r1",
  name: "pve-1",
  type: "pve" as const,
  url: "https://172.0.0.21:8006",
  status: "connected" as const,
};

describe("RemotesList — Console (Shell) action", () => {
  it("renders a Console (Shell) item and calls onShell with the remote", () => {
    const onShell = vi.fn();
    render(<RemotesList remotes={[remote]} onShell={onShell} />);

    fireEvent.click(screen.getByTitle("More actions"));
    fireEvent.click(screen.getByRole("button", { name: /console \(shell\)/i }));

    expect(onShell).toHaveBeenCalledTimes(1);
    expect(onShell).toHaveBeenCalledWith(remote);
  });
});
