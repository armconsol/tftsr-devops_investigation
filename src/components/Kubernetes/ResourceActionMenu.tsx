import React from "react";
import { MoreHorizontal } from "lucide-react";
import { Button } from "@/components/ui";
import { useSmartPosition } from "@/hooks/useSmartPosition";

export interface ResourceAction {
  label: string;
  icon: React.ElementType;
  onClick: () => void;
  variant?: "default" | "destructive";
  disabled?: boolean;
  hidden?: boolean;
}

interface ResourceActionMenuProps {
  actions: ResourceAction[];
  triggerLabel?: string;
}

export function ResourceActionMenu({ actions, triggerLabel = "Actions" }: ResourceActionMenuProps) {
  const [open, setOpen] = React.useState(false);
  const ref = React.useRef<HTMLDivElement>(null);
  const contentRef = React.useRef<HTMLDivElement>(null);
  const flipUpward = useSmartPosition(open, contentRef);

  const visible = actions.filter((a) => !a.hidden);

  React.useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  if (visible.length === 0) return null;

  return (
    <div ref={ref} className="relative inline-block text-left">
      <Button
        variant="ghost"
        size="sm"
        aria-label={triggerLabel}
        onClick={(e) => {
          e.stopPropagation();
          setOpen((v) => !v);
        }}
      >
        <MoreHorizontal className="h-4 w-4" />
      </Button>

      {open && (
        <div
          ref={contentRef}
          className={`absolute right-0 z-50 w-48 rounded-md border bg-card shadow-lg ${
            flipUpward ? "bottom-full mb-1" : "top-full mt-1"
          }`}
        >
          <div className="py-1">
            {visible.map((action, idx) => {
              const Icon = action.icon;
              return (
                <button
                  key={idx}
                  disabled={action.disabled}
                  onClick={(e) => {
                    e.stopPropagation();
                    setOpen(false);
                    action.onClick();
                  }}
                  className={[
                    "flex w-full items-center gap-2 px-3 py-2 text-sm transition-colors",
                    action.disabled
                      ? "cursor-not-allowed opacity-50"
                      : "cursor-pointer hover:bg-accent hover:text-accent-foreground",
                    action.variant === "destructive"
                      ? "text-destructive hover:text-destructive"
                      : "text-foreground",
                  ]
                    .filter(Boolean)
                    .join(" ")}
                >
                  <Icon className="h-4 w-4 shrink-0" />
                  {action.label}
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
