import React from "react";
import { X } from "lucide-react";
import { useBottomPanelStore, type BottomPanelTab } from "@/stores/bottomPanelStore";
import { cn } from "@/lib/utils";

interface BottomPanelManagerProps {
  className?: string;
}

/**
 * Renders the tab strip + close buttons. Active tab content is rendered
 * separately by `BottomPanel`.
 */
export function BottomPanelManager({ className }: BottomPanelManagerProps) {
  const tabs = useBottomPanelStore((s) => s.tabs);
  const activeTabId = useBottomPanelStore((s) => s.activeTabId);
  const setActiveTab = useBottomPanelStore((s) => s.setActiveTab);
  const closeTab = useBottomPanelStore((s) => s.closeTab);

  return (
    <div
      role="tablist"
      aria-label="Dock tabs"
      className={cn(
        "flex items-center gap-0.5 overflow-x-auto",
        className
      )}
    >
      {tabs.map((tab) => (
        <TabButton
          key={tab.id}
          tab={tab}
          isActive={tab.id === activeTabId}
          onActivate={() => setActiveTab(tab.id)}
          onClose={() => closeTab(tab.id)}
        />
      ))}
    </div>
  );
}

interface TabButtonProps {
  tab: BottomPanelTab;
  isActive: boolean;
  onActivate: () => void;
  onClose: () => void;
}

function TabButton({ tab, isActive, onActivate, onClose }: TabButtonProps) {
  return (
    <div
      role="tab"
      aria-selected={isActive}
      onClick={onActivate}
      className={cn(
        "flex items-center gap-1.5 px-3 py-1.5 text-xs cursor-pointer select-none border-r border-border min-w-0",
        "transition-colors",
        isActive
          ? "bg-background text-foreground border-t-2 border-t-primary"
          : "bg-card text-muted-foreground hover:bg-accent hover:text-foreground border-t-2 border-t-transparent"
      )}
      title={tab.title}
    >
      <span className="truncate max-w-[180px]">{tab.title}</span>
      <button
        type="button"
        aria-label={`Close tab ${tab.title}`}
        onClick={(e) => {
          e.stopPropagation();
          onClose();
        }}
        className="rounded-sm p-0.5 hover:bg-destructive/20 hover:text-destructive transition-colors"
      >
        <X className="w-3 h-3" />
      </button>
    </div>
  );
}
