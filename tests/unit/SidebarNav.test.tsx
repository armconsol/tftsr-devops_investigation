import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { useState } from "react";
import { Home, Server, Database, Wrench, Settings as SettingsIcon } from "lucide-react";
import { SidebarNav, type NavItem } from "@/components/SidebarNav";

const items: NavItem[] = [
  { id: "dashboard", to: "/", icon: Home, label: "Dashboard" },
  {
    id: "tools",
    icon: Wrench,
    label: "Tools",
    children: [
      { id: "kubernetes", to: "/kubernetes", icon: Server, label: "Kubernetes" },
      {
        id: "proxmox",
        to: "/proxmox",
        icon: Server,
        label: "Proxmox",
        children: [
          { id: "proxmox-vms", to: "/proxmox/vms", label: "VMs" },
          { id: "proxmox-ceph", to: "/proxmox/ceph", label: "Ceph" },
        ],
      },
      {
        id: "database",
        icon: Database,
        label: "Database",
        children: [{ id: "db-connections", to: "/database/connections", label: "Connections" }],
      },
      { id: "remote-desktop", to: "/remote-desktop", icon: Server, label: "Remote Desktop" },
    ],
  },
  {
    id: "settings",
    icon: SettingsIcon,
    label: "Settings",
    children: [{ id: "settings-providers", to: "/settings/providers", label: "AI Providers" }],
  },
];

function SidebarNavHarness() {
  const [expanded, setExpanded] = useState<string[]>([]);
  const onToggleSection = (id: string) =>
    setExpanded((prev) => (prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id]));
  return (
    <SidebarNav
      items={items}
      collapsed={false}
      expandedSections={expanded}
      onToggleSection={onToggleSection}
    />
  );
}

function renderHarness() {
  return render(
    <MemoryRouter>
      <SidebarNavHarness />
    </MemoryRouter>
  );
}

describe("SidebarNav", () => {
  it("renders top-level items and keeps Tools collapsed by default", () => {
    renderHarness();
    expect(screen.getByText("Dashboard")).toBeInTheDocument();
    expect(screen.getByText("Tools")).toBeInTheDocument();
    expect(screen.queryByText("Kubernetes")).not.toBeInTheDocument();
    expect(screen.queryByText("Proxmox")).not.toBeInTheDocument();
    expect(screen.queryByText("Database")).not.toBeInTheDocument();
    expect(screen.queryByText("Remote Desktop")).not.toBeInTheDocument();
  });

  it("expands Tools to reveal Kubernetes, Proxmox, Database, Remote Desktop", async () => {
    const user = userEvent.setup();
    renderHarness();
    await user.click(screen.getByText("Tools"));

    expect(screen.getByText("Kubernetes")).toBeInTheDocument();
    expect(screen.getByText("Proxmox")).toBeInTheDocument();
    expect(screen.getByText("Database")).toBeInTheDocument();
    expect(screen.getByText("Remote Desktop")).toBeInTheDocument();
  });

  it("expands nested Proxmox group to a second level", async () => {
    const user = userEvent.setup();
    renderHarness();
    await user.click(screen.getByText("Tools"));
    expect(screen.queryByText("VMs")).not.toBeInTheDocument();

    await user.click(screen.getByText("Proxmox"));
    expect(screen.getByText("VMs")).toBeInTheDocument();
    expect(screen.getByText("Ceph")).toBeInTheDocument();
  });

  it("keeps Settings collapsed by default and toggles it open", async () => {
    const user = userEvent.setup();
    renderHarness();
    expect(screen.getByText("Settings")).toBeInTheDocument();
    expect(screen.queryByText("AI Providers")).not.toBeInTheDocument();

    await user.click(screen.getByText("Settings"));
    expect(screen.getByText("AI Providers")).toBeInTheDocument();
  });

  it("does not render a link to the removed /proxmox/updates page", async () => {
    const user = userEvent.setup();
    renderHarness();
    await user.click(screen.getByText("Tools"));
    await user.click(screen.getByText("Proxmox"));
    expect(screen.queryByText("Updates")).not.toBeInTheDocument();
  });
});
