// Copyright (c) 2025 Shaun Arman
// MIT License - see LICENSE file for details

import React, { useState, useEffect, useRef } from "react";
import { Routes, Route } from "react-router-dom";
import {
  Home,
  Plus,
  Clock,
  Cpu,
  Bot,
  Shield,
  Link,
  Plug,
  ChevronLeft,
  ChevronRight,
  Sun,
  Moon,
  Terminal,
  FileCode,
  RefreshCw,
  Server,
  Server as ServerIcon,
  Settings,
  Database,
  Wrench,
} from "lucide-react";
import { Toaster } from "sonner";
import { SidebarNav, type NavItem } from "@/components/SidebarNav";
import { useSettingsStore } from "@/stores/settingsStore";
import {
  getAppVersionCmd,
  loadAiProvidersCmd,
  testProviderConnectionCmd,
  shutdownPortForwardsCmd,
} from "@/lib/tauriCommands";

import Dashboard from "@/pages/Dashboard";
import NewIssue from "@/pages/NewIssue";
import LogUpload from "@/pages/LogUpload";
import Triage from "@/pages/Triage";
import Resolution from "@/pages/Resolution";
import RCA from "@/pages/RCA";
import Postmortem from "@/pages/Postmortem";
import History from "@/pages/History";
import RemoteDesktopPage from "@/pages/Remote/RemoteDesktopPage";
import { RemoteDesktopPage as RemoteDesktopPageAlt } from "@/pages/RemoteDesktop/RemoteDesktopPage";
import AIProviders from "@/pages/Settings/AIProviders";
import Ollama from "@/pages/Settings/Ollama";
import Integrations from "@/pages/Settings/Integrations";
import MCPServers from "@/pages/Settings/MCPServers";
import Security from "@/pages/Settings/Security";
import ShellExecution from "@/pages/Settings/ShellExecution";
import KubeconfigManager from "@/pages/Settings/KubeconfigManager";
import { KubernetesPage } from "@/pages/Kubernetes/KubernetesPage";
import { ShellApprovalModal } from "@/components/ShellApprovalModal";
import { ProxmoxRemotesPage } from "@/pages/Proxmox/RemotesPage";
import { ProxmoxVMsPage } from "@/pages/Proxmox/VMsPage";
import { ProxmoxContainersPage } from "@/pages/Proxmox/ContainersPage";
import { ProxmoxStoragePage } from "@/pages/Proxmox/StoragePage";
import { ProxmoxNetworkPage } from "@/pages/Proxmox/NetworkPage";
import { ProxmoxFirewallPage } from "@/pages/Proxmox/FirewallPage";
import { ProxmoxACLPage } from "@/pages/Proxmox/ACLPage";
import { ProxmoxBackupPage } from "@/pages/Proxmox/BackupPage";
import { ProxmoxCephPage } from "@/pages/Proxmox/CephPage";
import { ProxmoxPBSPage } from "@/pages/Proxmox/PBSPage";
import { ProxmoxSDNPage } from "@/pages/Proxmox/SDNPage";
import { ProxmoxHAPage } from "@/pages/Proxmox/HAPage";
import { ProxmoxTasksPage } from "@/pages/Proxmox/TasksPage";
import { ProxmoxCertificatesPage } from "@/pages/Proxmox/CertificatesPage";
import { ProxmoxSubscriptionPage } from "@/pages/Proxmox/SubscriptionPage";
import { ProxmoxNotesPage } from "@/pages/Proxmox/NotesPage";
import { ProxmoxSearchPage } from "@/pages/Proxmox/SearchPage";
import { ProxmoxAdminPage } from "@/pages/Proxmox/AdminPage";
import { ProxmoxNodeDetailPage } from "@/pages/Proxmox/NodeDetailPage";
import { ProxmoxConsolePage } from "@/pages/Proxmox/ConsolePage";
import { ProxmoxShellPage } from "@/pages/Proxmox/ShellPage";
import { ProxmoxSettings } from "@/pages/Settings/Proxmox";
import { Updater } from "@/pages/Settings/Updater";
import { RouteErrorBoundary } from "@/components/RouteErrorBoundary";

// Database Management Pages
import { ConnectionManager } from "@/pages/Database/ConnectionManager";
import { SQLEditor } from "@/pages/Database/SQLEditor";
import { SchemaExplorer } from "@/pages/Database/SchemaExplorer";
import { TableBrowserPage } from "@/pages/Database/TableBrowserPage";
import { QueryHistoryPage } from "@/pages/Database/QueryHistory";
import { ImportExport } from "@/pages/Database/ImportExport";
import { ERDiagram } from "@/pages/Database/ERDiagram";
import { QueryBuilder } from "@/pages/Database/QueryBuilder";

const navItems: NavItem[] = [
  { id: "dashboard", to: "/", icon: Home, label: "Dashboard" },
  { id: "new-issue", to: "/new-issue", icon: Plus, label: "New Issue" },
  {
    id: "tools",
    icon: Wrench,
    label: "Tools",
    children: [
      { id: "kubernetes", to: "/kubernetes", icon: Server, label: "Kubernetes" },
      {
        id: "proxmox",
        to: "/proxmox",
        icon: ServerIcon,
        label: "Proxmox",
        children: [
          { id: "proxmox-search", to: "/proxmox/search", label: "Search" },
          { id: "proxmox-remotes", to: "/proxmox/remotes", label: "Remotes" },
          { id: "proxmox-vms", to: "/proxmox/vms", label: "VMs" },
          { id: "proxmox-containers", to: "/proxmox/containers", label: "Containers" },
          { id: "proxmox-storage", to: "/proxmox/storage", label: "Storage" },
          { id: "proxmox-network", to: "/proxmox/network", label: "Network" },
          { id: "proxmox-firewall", to: "/proxmox/firewall", label: "Firewall" },
          { id: "proxmox-ceph", to: "/proxmox/ceph", label: "Ceph" },
          { id: "proxmox-sdn", to: "/proxmox/sdn", label: "SDN" },
          { id: "proxmox-ha", to: "/proxmox/ha", label: "HA Groups" },
          { id: "proxmox-backup", to: "/proxmox/backup", label: "Backup" },
          { id: "proxmox-pbs", to: "/proxmox/pbs", label: "PBS" },
          { id: "proxmox-tasks", to: "/proxmox/tasks", label: "Tasks" },
          { id: "proxmox-notes", to: "/proxmox/notes", label: "Notes" },
          { id: "proxmox-certificates", to: "/proxmox/certificates", label: "Certificates" },
          { id: "proxmox-subscriptions", to: "/proxmox/subscriptions", label: "Subscriptions" },
          { id: "proxmox-admin", to: "/proxmox/admin", label: "Administration" },
          { id: "proxmox-nodes", to: "/proxmox/nodes", label: "Node Detail" },
        ],
      },
      {
        id: "database",
        icon: Database,
        label: "Database",
        children: [
          { id: "db-connections", to: "/database/connections", label: "Connections" },
          { id: "db-editor", to: "/database/editor", label: "SQL Editor" },
          { id: "db-query-builder", to: "/database/query-builder", label: "Query Builder" },
          { id: "db-schema", to: "/database/schema", label: "Schema Explorer" },
          { id: "db-browser", to: "/database/browser", label: "Table Browser" },
          { id: "db-history", to: "/database/history", label: "Query History" },
          { id: "db-import-export", to: "/database/import-export", label: "Import/Export" },
          { id: "db-er-diagram", to: "/database/er-diagram", label: "ER Diagram" },
        ],
      },
      { id: "remote-desktop", to: "/remote-desktop", icon: Server, label: "Remote Desktop" },
    ],
  },
  { id: "history", to: "/history", icon: Clock, label: "History" },
];

const settingsNavItem: NavItem = {
  id: "settings",
  icon: Settings,
  label: "Settings",
  children: [
    { id: "settings-providers", to: "/settings/providers", icon: Cpu, label: "AI Providers" },
    { id: "settings-ollama", to: "/settings/ollama", icon: Bot, label: "Ollama" },
    { id: "settings-shell", to: "/settings/shell", icon: Terminal, label: "Shell Execution" },
    { id: "settings-kubeconfig", to: "/settings/kubeconfig", icon: FileCode, label: "Kubeconfig" },
    { id: "settings-integrations", to: "/settings/integrations", icon: Link, label: "Integrations" },
    { id: "settings-mcp", to: "/settings/mcp", icon: Plug, label: "MCP Servers" },
    { id: "settings-security", to: "/settings/security", icon: Shield, label: "Security" },
    { id: "settings-updater", to: "/settings/updater", icon: RefreshCw, label: "Updater" },
    { id: "settings-proxmox", to: "/settings/proxmox", icon: Settings, label: "Proxmox" },
  ],
};

export default function App() {
  const [collapsed, setCollapsed] = useState(false);
  const [expandedSections, setExpandedSections] = useState<string[]>([]);
  const [appVersion, setAppVersion] = useState("");
  const {
    theme,
    setTheme,
    setProviders,
    getActiveProvider,
  } = useSettingsStore();
  const cleanupDone = useRef(false);

  useEffect(() => {
    getAppVersionCmd().then(setAppVersion).catch(() => {});
  }, []);

  // Cleanup port forwards on app unmount
  useEffect(() => {
    return () => {
      if (!cleanupDone.current) {
        cleanupDone.current = true;
        void shutdownPortForwardsCmd().catch((err) => {
          console.error("Failed to shutdown port forwards:", err);
        });
      }
    };
  }, []);

  // Apply dark mode class to html element for proper CSS cascade
  useEffect(() => {
    if (theme === "dark") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }, [theme]);

  // Load providers and auto-test active provider on startup
  useEffect(() => {
    const initializeProviders = async () => {
      try {
        const providers = await loadAiProvidersCmd();
        setProviders(providers);

        // Auto-test the active provider
        const activeProvider = getActiveProvider();
        if (activeProvider) {
          try {
            await testProviderConnectionCmd(activeProvider);
          } catch (err) {
            console.warn("⚠ Active provider connection test failed:", activeProvider.name, err);
          }
        }
      } catch (err) {
        console.error("Failed to initialize AI providers:", err);
      }
    };
    initializeProviders();
  }, [setProviders, getActiveProvider]);

  return (
    <>
      <ShellApprovalModal />
      <div className="grid h-screen" style={{ gridTemplateColumns: collapsed ? "64px 1fr" : "240px 1fr" }}>
        {/* Sidebar */}
        <aside className="bg-card border-r flex flex-col h-screen overflow-y-auto">
          {/* Logo */}
          <div className="flex items-center justify-between px-4 py-4 border-b">
            {!collapsed && (
              <span className="text-lg font-bold text-foreground tracking-tight">
                Troubleshooting and RCA Assistant
              </span>
            )}
            <button
              onClick={() => setCollapsed((c) => !c)}
              className="p-1 rounded hover:bg-accent text-muted-foreground"
            >
              {collapsed ? <ChevronRight className="w-4 h-4" /> : <ChevronLeft className="w-4 h-4" />}
            </button>
          </div>

          {/* Main nav */}
          <nav className="flex-1 px-2 py-3 space-y-1">
            <SidebarNav
              items={navItems}
              collapsed={collapsed}
              expandedSections={expandedSections}
              onToggleSection={(id) =>
                setExpandedSections((prev) =>
                  prev.includes(id) ? prev.filter((t) => t !== id) : [...prev, id]
                )
              }
            />

            {/* Settings section */}
            <div className="pt-4">
              <SidebarNav
                items={[settingsNavItem]}
                collapsed={collapsed}
                expandedSections={expandedSections}
                onToggleSection={(id) =>
                  setExpandedSections((prev) =>
                    prev.includes(id) ? prev.filter((t) => t !== id) : [...prev, id]
                  )
                }
              />
            </div>
          </nav>

          {/* Version + Theme toggle */}
          <div className="px-4 py-3 border-t flex items-center justify-between">
            <span className="text-xs text-muted-foreground">
              {appVersion ? `v${appVersion}` : ""}
            </span>
            <button
              onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
              className="p-1 rounded hover:bg-accent text-muted-foreground"
              title={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
            >
              {theme === "dark" ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
            </button>
          </div>
        </aside>

        {/* Main content */}
        <main className="overflow-y-auto bg-background">
          <RouteErrorBoundary>
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/new-issue" element={<NewIssue />} />
            <Route path="/issue/:id/logs" element={<LogUpload />} />
            <Route path="/issue/:id/triage" element={<Triage />} />
            <Route path="/issue/:id/resolution" element={<Resolution />} />
            <Route path="/issue/:id/rca" element={<RCA />} />
            <Route path="/issue/:id/postmortem" element={<Postmortem />} />
            <Route path="/history" element={<History />} />
            <Route path="/remote" element={<RemoteDesktopPageAlt />} />
            <Route path="/settings/providers" element={<AIProviders />} />
            <Route path="/settings/ollama" element={<Ollama />} />
            <Route path="/settings/shell" element={<ShellExecution />} />
            <Route path="/settings/kubeconfig" element={<KubeconfigManager />} />
            <Route path="/kubernetes" element={<KubernetesPage />} />
          <Route path="/proxmox/remotes" element={<ProxmoxRemotesPage />} />
          <Route path="/proxmox/vms" element={<ProxmoxVMsPage />} />
          <Route path="/proxmox/containers" element={<ProxmoxContainersPage />} />
          <Route path="/proxmox/storage" element={<ProxmoxStoragePage />} />
          <Route path="/proxmox/network" element={<ProxmoxNetworkPage />} />
          <Route path="/proxmox/firewall" element={<ProxmoxFirewallPage />} />
          <Route path="/remote-desktop" element={<RemoteDesktopPage />} />
          <Route path="/proxmox/acl" element={<ProxmoxACLPage />} />
          <Route path="/proxmox/backup" element={<ProxmoxBackupPage />} />
          <Route path="/proxmox/ceph" element={<ProxmoxCephPage />} />
          <Route path="/proxmox/pbs" element={<ProxmoxPBSPage />} />
          <Route path="/proxmox/sdn" element={<ProxmoxSDNPage />} />
          <Route path="/proxmox/ha" element={<ProxmoxHAPage />} />
          <Route path="/proxmox/tasks" element={<ProxmoxTasksPage />} />
          <Route path="/proxmox/certificates" element={<ProxmoxCertificatesPage />} />
          <Route path="/proxmox/subscriptions" element={<ProxmoxSubscriptionPage />} />
          <Route path="/proxmox/notes" element={<ProxmoxNotesPage />} />
          <Route path="/proxmox/search" element={<ProxmoxSearchPage />} />
          <Route path="/proxmox/admin" element={<ProxmoxAdminPage />} />
          <Route path="/proxmox/nodes" element={<ProxmoxNodeDetailPage />} />
          <Route path="/proxmox/console/:clusterId/:node/:vmid/:kind" element={<ProxmoxConsolePage />} />
          <Route path="/proxmox/shell/:clusterId/:node" element={<ProxmoxShellPage />} />
          <Route path="/settings/updater" element={<Updater />} />
          <Route path="/settings/proxmox" element={<ProxmoxSettings />} />
            <Route path="/settings/integrations" element={<Integrations />} />
            <Route path="/settings/mcp" element={<MCPServers />} />
            <Route path="/settings/security" element={<Security />} />

            {/* Database Management Routes */}
            <Route path="/database/connections" element={<ConnectionManager />} />
            <Route path="/database/editor" element={<SQLEditor />} />
            <Route path="/database/query-builder" element={<QueryBuilder />} />
            <Route path="/database/schema" element={<SchemaExplorer />} />
            <Route path="/database/browser" element={<TableBrowserPage />} />
            <Route path="/database/history" element={<QueryHistoryPage />} />
            <Route path="/database/import-export" element={<ImportExport />} />
            <Route path="/database/er-diagram" element={<ERDiagram />} />
          </Routes>
          </RouteErrorBoundary>
        </main>
      </div>
      <Toaster richColors position="top-right" />
    </>
  );
}
