import React from "react";
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { ProxmoxBackupPage } from "@/pages/Proxmox/BackupPage";
import * as proxmoxClient from "@/lib/proxmoxClient";

vi.mock("@/lib/proxmoxClient");
vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

const cluster = { id: "dc1", name: "DC1" } as never;

const rawJob = {
  id: "backup-local",
  schedule: "0 2 * * *",
  enabled: 1,
  storage: "local",
  vmid: "100",
  mode: "snapshot",
  node: "pve1",
};

describe("ProxmoxBackupPage actions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(proxmoxClient.listProxmoxClusters).mockResolvedValue([cluster]);
    vi.mocked(proxmoxClient.listProxmoxBackupJobs).mockResolvedValue([rawJob]);
    vi.mocked(proxmoxClient.triggerProxmoxBackupJob).mockResolvedValue(undefined);
    vi.mocked(proxmoxClient.updateProxmoxBackupJob).mockResolvedValue(undefined);
  });

  it("wires the Trigger action to triggerProxmoxBackupJob with the string id", async () => {
    render(<ProxmoxBackupPage />);
    const triggerBtn = await screen.findByTitle("Trigger Now");
    fireEvent.click(triggerBtn);
    await waitFor(() =>
      expect(proxmoxClient.triggerProxmoxBackupJob).toHaveBeenCalledWith(
        "dc1",
        "backup-local"
      )
    );
  });

  it("opens the edit dialog prefilled and saves via updateProxmoxBackupJob", async () => {
    render(<ProxmoxBackupPage />);
    const editBtn = await screen.findByTitle("Edit");
    fireEvent.click(editBtn);

    // Dialog opens in edit mode.
    expect(await screen.findByText("Edit Backup Job")).toBeDefined();

    const saveBtn = screen.getByText("Save Changes");
    fireEvent.click(saveBtn);

    await waitFor(() =>
      expect(proxmoxClient.updateProxmoxBackupJob).toHaveBeenCalledWith(
        "dc1",
        "backup-local",
        expect.objectContaining({ storage: "local", vmid: "100", mode: "snapshot" })
      )
    );
  });
});
