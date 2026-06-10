import React, { useCallback, useEffect, useRef } from "react";
import { ChevronDown } from "lucide-react";
import {
  useBottomPanelStore,
  BottomPanelTabType,
  MIN_PANEL_HEIGHT,
  MAX_PANEL_HEIGHT,
  type BottomPanelTab,
} from "@/stores/bottomPanelStore";
import { BottomPanelManager } from "./BottomPanelManager";
import { LogsTab, type LogsTabData } from "./dock/LogsTab";
import { TerminalTab, type TerminalTabData } from "./dock/TerminalTab";
import { YamlEditorTab, type YamlEditorTabData } from "./dock/YamlEditorTab";
import { cn } from "@/lib/utils";

/**
 * Bottom dock panel — DevTools-style. Houses tabs for pod logs, terminals, YAML
 * editing, resource creation, and Helm install/upgrade flows.
 *
 * Renders only when the store reports the panel as open and at least one tab
 * exists. Visibility, active tab, and tab list all live in the store; this
 * component owns drag-resize, keyboard shortcuts, and content dispatch.
 */
export function BottomPanel() {
  const isOpen = useBottomPanelStore((s) => s.isOpen);
  const height = useBottomPanelStore((s) => s.height);
  const tabs = useBottomPanelStore((s) => s.tabs);
  const activeTabId = useBottomPanelStore((s) => s.activeTabId);
  const setHeight = useBottomPanelStore((s) => s.setHeight);
  const closePanel = useBottomPanelStore((s) => s.closePanel);
  const closeActiveTab = useBottomPanelStore((s) => s.closeActiveTab);
  const closeTab = useBottomPanelStore((s) => s.closeTab);
  const nextTab = useBottomPanelStore((s) => s.nextTab);
  const previousTab = useBottomPanelStore((s) => s.previousTab);

  const dragStateRef = useRef<{ startY: number; startHeight: number } | null>(null);

  // ── Drag-to-resize ────────────────────────────────────────────────────────
  const handleDragMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      dragStateRef.current = { startY: e.clientY, startHeight: height };

      const onMove = (ev: MouseEvent) => {
        if (!dragStateRef.current) return;
        const delta = dragStateRef.current.startY - ev.clientY;
        const next = dragStateRef.current.startHeight + delta;
        setHeight(next);
      };
      const onUp = () => {
        dragStateRef.current = null;
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", onUp);
      };
      window.addEventListener("mousemove", onMove);
      window.addEventListener("mouseup", onUp);
    },
    [height, setHeight]
  );

  // ── Keyboard shortcuts ────────────────────────────────────────────────────
  useEffect(() => {
    if (!isOpen) return;

    const onKey = (e: KeyboardEvent) => {
      // Ctrl+W — close active tab
      if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key.toLowerCase() === "w") {
        e.preventDefault();
        closeActiveTab();
        return;
      }
      // Shift+Escape — hide the panel
      if (e.shiftKey && e.key === "Escape") {
        e.preventDefault();
        closePanel();
        return;
      }
      // Ctrl+. — next tab
      if (e.ctrlKey && !e.shiftKey && e.key === ".") {
        e.preventDefault();
        nextTab();
        return;
      }
      // Ctrl+, — previous tab
      if (e.ctrlKey && !e.shiftKey && e.key === ",") {
        e.preventDefault();
        previousTab();
        return;
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [isOpen, closeActiveTab, closePanel, nextTab, previousTab]);

  if (!isOpen || tabs.length === 0) return null;

  const activeTab = tabs.find((t) => t.id === activeTabId) ?? tabs[0]!;
  const clampedHeight = Math.min(MAX_PANEL_HEIGHT, Math.max(MIN_PANEL_HEIGHT, height));

  return (
    <div
      data-testid="bottom-panel"
      className={cn(
        "flex flex-col border-t border-border bg-background shrink-0",
        "shadow-[0_-2px_8px_rgba(0,0,0,0.04)]"
      )}
      style={{ height: `${clampedHeight}px` }}
    >
      {/* Drag handle */}
      <div
        data-testid="bottom-panel-drag-handle"
        role="separator"
        aria-orientation="horizontal"
        aria-label="Resize bottom panel"
        onMouseDown={handleDragMouseDown}
        className="h-1 w-full cursor-row-resize bg-border hover:bg-primary/50 transition-colors flex-shrink-0"
      />

      {/* Tab strip */}
      <div className="flex items-stretch border-b border-border bg-card flex-shrink-0">
        <BottomPanelManager className="flex-1" />
        <button
          type="button"
          aria-label="Hide bottom panel"
          onClick={closePanel}
          className="px-2 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
        >
          <ChevronDown className="w-4 h-4" />
        </button>
      </div>

      {/* Active tab content */}
      <div className="flex-1 overflow-hidden min-h-0 bg-background">
        <TabContent tab={activeTab} onClose={closeTab} />
      </div>
    </div>
  );
}

// ─── Tab dispatcher ───────────────────────────────────────────────────────────

interface TabContentProps {
  tab: BottomPanelTab;
  onClose: (id: string) => void;
}

function TabContent({ tab, onClose }: TabContentProps) {
  switch (tab.type) {
    case BottomPanelTabType.POD_LOGS:
      return <LogsTab data={(tab.data ?? {}) as LogsTabData} />;

    case BottomPanelTabType.TERMINAL:
      return <TerminalTab data={(tab.data ?? {}) as TerminalTabData} />;

    case BottomPanelTabType.EDIT_RESOURCE:
    case BottomPanelTabType.CREATE_RESOURCE:
    case BottomPanelTabType.INSTALL_CHART:
    case BottomPanelTabType.UPGRADE_CHART:
      return (
        <YamlEditorTab
          tabId={tab.id}
          data={
            { ...(tab.data ?? {}), mode: tab.type } as YamlEditorTabData
          }
          onClose={onClose}
        />
      );

    default:
      return (
        <div className="p-4 text-xs text-muted-foreground">
          Unsupported tab type.
        </div>
      );
  }
}
