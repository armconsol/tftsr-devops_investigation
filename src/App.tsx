import React, { useState, useEffect, useRef } from "react";
import { Routes, Route, NavLink, useLocation } from "react-router-dom";
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
  ChevronDown,
  Sun,
  Moon,
  Terminal,
  FileCode,
  RefreshCw,
  Server,
  Server as ServerIcon,
  Settings,
} from "lucide-react";
import { Toaster } from "sonner";
import { useSettingsStore } from "@/stores/settingsStore";
import { getAppVersionCmd, loadAiProvidersCmd, testProviderConnectionCmd, shutdownPortForwardsCmd } from "@/lib/tauriCommands";

import Dashboard from "@/pages/Dashboard";
import NewIssue from "@/pages/NewIssue";
import LogUpload from "@/pages/LogUpload";
import Triage from "@/pages/Triage";
import Resolution from "@/pages/Resolution";
import RCA from "@/pages/RCA";
import Postmortem from "@/pages/Postmortem";
import History from "@/pages/History";
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
import { RemoteDesktopPage } from "@/pages/RemoteDesktop/RemoteDesktopPage";
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
import { ProxmoxUpdatesPage } from "@/pages/Proxmox/UpdatesPage";
import { ProxmoxNodeDetailPage } from "@/pages/Proxmox/NodeDetailPage";
import { ProxmoxConsolePage } from "@/pages/Proxmox/ConsolePage";
import { ProxmoxShellPage } from "@/pages/Proxmox/ShellPage";
import { ProxmoxSettings } from "@/pages/Settings/Proxmox";
import { Updater } from "@/pages/Settings/Updater";
import { RouteErrorBoundary } from "@/components/RouteErrorBoundary";

const navItems = [
  { to: "/", icon: Home, label: "Dashboard" },
  { to: "/new-issue", icon: Plus, label: "New Issue" },
  { to: "/kubernetes", icon: Server, label: "Kubernetes" },
  {
    to: "/proxmox",
    icon: ServerIcon,
    label: "Proxmox",
    children: [
      { to: "/proxmox/search", label: "Search" },
      { to: "/proxmox/remotes", label: "Remotes" },
      { to: "/proxmox/vms", label: "VMs" },
      { to: "/proxmox/containers", label: "Containers" },
      { to: "/proxmox/storage", label: "Storage" },
      { to: "/proxmox/network", label: "Network" },
      { to: "/proxmox/firewall", label: "Firewall" },
      { to: "/proxmox/ceph", label: "Ceph" },
      { to: "/proxmox/sdn", label: "SDN" },
      { to: "/proxmox/ha", label: "HA Groups" },
      { to: "/proxmox/backup", label: "Backup" },
      { to: "/proxmox/pbs", label: "PBS" },
      { to: "/proxmox/tasks", label: "Tasks" },
      { to: "/proxmox/notes", label: "Notes" },
      { to: "/proxmox/certificates", label: "Certificates" },
      { to: "/proxmox/subscriptions", label: "Subscriptions" },
      { to: "/proxmox/admin", label: "Administration" },
      { to: "/proxmox/updates", label: "Updates" },
      { to: "/proxmox/nodes", label: "Node Detail" },
    ],
  },
  { to: "/remote-desktop", icon: Server, label: "Remote Desktop" },
  { to: "/history", icon: Clock, label: "History" },
];

const settingsItems = [
  { to: "/settings/providers", icon: Cpu, label: "AI Providers" },
  { to: "/settings/ollama", icon: Bot, label: "Ollama" },
  { to: "/settings/shell", icon: Terminal, label: "Shell Execution" },
  { to: "/settings/kubeconfig", icon: FileCode, label: "Kubeconfig" },
  { to: "/settings/integrations", icon: Link, label: "Integrations" },
  { to: "/settings/mcp", icon: Plug, label: "MCP Servers" },
  { to: "/settings/security", icon: Shield, label: "Security" },
  { to: "/settings/updater", icon: RefreshCw, label: "Updater" },
  { to: "/settings/proxmox", icon: Settings, label: "Proxmox" },
];

export default function App() {
  const [collapsed, setCollapsed] = useState(false);
  const [expandedSections, setExpandedSections] = useState<string[]>([]);
  const [appVersion, setAppVersion] = useState("");
  const { theme, setTheme, setProviders, getActiveProvider } = useSettingsStore();
  const cleanupDone = useRef(false);
  const location = useLocation();

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
            {navItems.map((item) => {
              if (item.children) {
                const isExpanded = expandedSections.includes(item.to);
                const isActive = location.pathname.startsWith(item.to);
                return (
                  <div key={item.to}>
                    <button
                      onClick={() =>
                        setExpandedSections((prev) =>
                          prev.includes(item.to)
                            ? prev.filter((t) => t !== item.to)
                            : [...prev, item.to]
                        )
                      }
                      className={`w-full flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                        isActive
                          ? "bg-primary text-primary-foreground"
                          : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                      }`}
                    >
                      <item.icon className="w-4 h-4 shrink-0" />
                      {!collapsed && <span>{item.label}</span>}
                      {!collapsed && (
                        isExpanded
                          ? <ChevronDown className="w-3 h-3 ml-auto" />
                          : <ChevronRight className="w-3 h-3 ml-auto" />
                      )}
                    </button>
                    {!collapsed && isExpanded && (
                      <div className="ml-4 space-y-1 pl-4 border-l border-muted">
                        {item.children.map((child) => (
                          <NavLink
                            key={child.to}
                            to={child.to}
                            className={({ isActive: childActive }) =>
                              `flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors ${
                                childActive
                                  ? "bg-primary text-primary-foreground"
                                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                              }`
                            }
                          >
                            <span className="w-4 h-4 shrink-0" />
                            <span>{child.label}</span>
                          </NavLink>
                        ))}
                      </div>
                    )}
                  </div>
                );
              }
              return (
                <NavLink
                  key={item.to}
                  to={item.to}
                  end={item.to === "/"}
                  className={({ isActive }) =>
                    `flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                      isActive
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                    }`
                  }
                >
                  <item.icon className="w-4 h-4 shrink-0" />
                  {!collapsed && <span>{item.label}</span>}
                </NavLink>
              );
            })}

            {/* Settings section */}
            <div className="pt-4">
              {!collapsed && (
                <p className="px-3 text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
                  Settings
                </p>
              )}
              {settingsItems.map((item) => (
                <NavLink
                  key={item.to}
                  to={item.to}
                  className={({ isActive }) =>
                    `flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                      isActive
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                    }`
                  }
                >
                  <item.icon className="w-4 h-4 shrink-0" />
                  {!collapsed && <span>{item.label}</span>}
                </NavLink>
              ))}
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
          <Route path="/proxmox/updates" element={<ProxmoxUpdatesPage />} />
          <Route path="/proxmox/nodes" element={<ProxmoxNodeDetailPage />} />
          <Route path="/proxmox/console/:clusterId/:node/:vmid/:kind" element={<ProxmoxConsolePage />} />
          <Route path="/proxmox/shell/:clusterId/:node" element={<ProxmoxShellPage />} />
          <Route path="/settings/updater" element={<Updater />} />
          <Route path="/settings/proxmox" element={<ProxmoxSettings />} />
            <Route path="/settings/integrations" element={<Integrations />} />
            <Route path="/settings/mcp" element={<MCPServers />} />
            <Route path="/settings/security" element={<Security />} />
          </Routes>
          </RouteErrorBoundary>
        </main>
      </div>
      <Toaster richColors position="top-right" />
    </>
  );
}
