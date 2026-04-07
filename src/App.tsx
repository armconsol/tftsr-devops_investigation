import React, { useState, useEffect } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { Routes, Route, NavLink, useLocation } from "react-router-dom";
import {
  Home,
  Plus,
  Clock,
  Cpu,
  Bot,
  Shield,
  Link,
  ChevronLeft,
  ChevronRight,
  Sun,
  Moon,
} from "lucide-react";
import { useSettingsStore } from "@/stores/settingsStore";
import { loadAiProvidersCmd, testProviderConnectionCmd } from "@/lib/tauriCommands";

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
import Security from "@/pages/Settings/Security";

const navItems = [
  { to: "/", icon: Home, label: "Dashboard" },
  { to: "/new-issue", icon: Plus, label: "New Issue" },
  { to: "/history", icon: Clock, label: "History" },
];

const settingsItems = [
  { to: "/settings/providers", icon: Cpu, label: "AI Providers" },
  { to: "/settings/ollama", icon: Bot, label: "Ollama" },
  { to: "/settings/integrations", icon: Link, label: "Integrations" },
  { to: "/settings/security", icon: Shield, label: "Security" },
];

export default function App() {
  const [collapsed, setCollapsed] = useState(false);
  const [appVersion, setAppVersion] = useState("");
  const { theme, setTheme, setProviders, getActiveProvider } = useSettingsStore();
  const location = useLocation();

  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => {});
  }, []);

  // Load providers and auto-test active provider on startup
  useEffect(() => {
    const initializeProviders = async () => {
      try {
        const providers = await loadAiProvidersCmd();
        setProviders(providers);

        // Auto-test the active provider
        const activeProvider = getActiveProvider();
        if (activeProvider) {
          console.log("Auto-testing active AI provider:", activeProvider.name);
          try {
            await testProviderConnectionCmd(activeProvider);
            console.log("✓ Active provider connection verified:", activeProvider.name);
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
    <div className={theme === "dark" ? "dark" : ""}>
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
            {navItems.map((item) => (
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
            ))}

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
            {!collapsed && (
              <span className="text-xs text-muted-foreground">
                {appVersion ? `v${appVersion}` : ""}
              </span>
            )}
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
            <Route path="/settings/integrations" element={<Integrations />} />
            <Route path="/settings/security" element={<Security />} />
          </Routes>
        </main>
      </div>
    </div>
  );
}
