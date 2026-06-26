import React from "react";
import { FileText, Terminal, Play } from "lucide-react";
import { Button } from "@/components/ui";

export interface QuickAction {
  type: "logs" | "shell" | "exec" | "custom";
  icon?: React.ElementType;
  tooltip: string;
  onClick: () => void;
  disabled?: boolean;
  variant?: "default" | "destructive" | "outline" | "ghost";
}

interface QuickActionColumnProps {
  actions: QuickAction[];
}

const DEFAULT_ICONS: Record<string, React.ElementType> = {
  logs: FileText,
  shell: Terminal,
  exec: Play,
};

export function QuickActionColumn({ actions }: QuickActionColumnProps) {
  return (
    <div className="flex items-center gap-1">
      {actions.map((action, index) => {
        const Icon = action.icon || DEFAULT_ICONS[action.type];
        return (
          <Button
            key={index}
            variant={action.variant || "ghost"}
            size="sm"
            onClick={(e) => {
              e.stopPropagation();
              action.onClick();
            }}
            disabled={action.disabled}
            title={action.tooltip}
            className="h-7 w-7 p-0"
          >
            {Icon && <Icon className="h-3.5 w-3.5" />}
          </Button>
        );
      })}
    </div>
  );
}
