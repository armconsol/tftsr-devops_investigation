import React from "react";
import { Button } from "@/components/ui";
import { Settings, Bell, User, Search, Plus, RefreshCw } from "lucide-react";
import { Badge } from "@/components/ui";
import { useKubernetesStore } from "@/stores/kubernetesStore";
import { useStore } from "zustand";

interface HotbarProps {
  onRefresh: () => void;
  onAddResource: () => void;
  onSettings: () => void;
  onNotifications?: () => void;
  notificationCount?: number;
}

export function Hotbar({ onRefresh, onAddResource, onSettings, onNotifications, notificationCount = 0 }: HotbarProps) {
  const clusters = useStore(useKubernetesStore, (state) => state.clusters);
  const selectedClusterId = useStore(useKubernetesStore, (state) => state.selectedClusterId);
  const selectedCluster = clusters.find((c: { id: string }) => c.id === selectedClusterId);

  return (
    <div className="h-12 bg-background border-b flex items-center justify-between px-4">
      <div className="flex items-center gap-2">
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="sm" onClick={onRefresh}>
            <RefreshCw className="w-4 h-4" />
          </Button>
          <Button variant="ghost" size="sm" onClick={onAddResource}>
            <Plus className="w-4 h-4" />
          </Button>
        </div>
        <div className="h-6 w-px bg-border mx-2" />
        <div className="flex items-center gap-2">
          <Search className="w-4 h-4 text-muted-foreground" />
          <span className="text-sm text-muted-foreground">
            {selectedCluster?.name || "No cluster selected"}
          </span>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={onNotifications}
            aria-label="Notifications"
          >
            <Bell className="w-4 h-4" />
            {notificationCount > 0 && (
              <Badge variant="destructive" className="h-4 w-4 flex items-center justify-center p-0 text-[10px]">
                {notificationCount}
              </Badge>
            )}
          </Button>
          <Button variant="ghost" size="sm" onClick={onSettings}>
            <Settings className="w-4 h-4" />
          </Button>
          <Button variant="ghost" size="sm">
            <User className="w-4 h-4" />
          </Button>
        </div>
      </div>
    </div>
  );
}
