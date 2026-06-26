import React from "react";
import { Command, X } from "lucide-react";
import { Input } from "@/components/ui";
import { Button } from "@/components/ui";
import { Badge } from "@/components/ui";

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  onNavigate?: (section: string) => void;
  clusterId?: string;
  namespace?: string;
}

interface PaletteCommand {
  id: string;
  label: string;
  category: string;
  action: "navigate";
  target: string;
}

const COMMANDS: PaletteCommand[] = [
  { id: "nav-overview", label: "Go to Overview", category: "Navigate", action: "navigate", target: "overview" },
  { id: "nav-pods", label: "Go to Pods", category: "Navigate", action: "navigate", target: "pods" },
  { id: "nav-deployments", label: "Go to Deployments", category: "Navigate", action: "navigate", target: "deployments" },
  { id: "nav-services", label: "Go to Services", category: "Navigate", action: "navigate", target: "services" },
  { id: "nav-nodes", label: "Go to Nodes", category: "Navigate", action: "navigate", target: "nodes" },
  { id: "nav-events", label: "Go to Events", category: "Navigate", action: "navigate", target: "events" },
  { id: "nav-configmaps", label: "Go to Config Maps", category: "Navigate", action: "navigate", target: "configmaps" },
  { id: "nav-secrets", label: "Go to Secrets", category: "Navigate", action: "navigate", target: "secrets" },
  { id: "nav-pvc", label: "Go to PVCs", category: "Navigate", action: "navigate", target: "pvcs" },
  { id: "nav-ingresses", label: "Go to Ingresses", category: "Navigate", action: "navigate", target: "ingresses" },
  { id: "nav-portfwd", label: "Go to Port Forwarding", category: "Navigate", action: "navigate", target: "portforwarding" },
  { id: "nav-rbac", label: "Go to Roles", category: "Navigate", action: "navigate", target: "roles" },
];

export function CommandPalette({ isOpen, onClose, onNavigate }: CommandPaletteProps) {
  const [query, setQuery] = React.useState("");
  const [selectedIndex, setSelectedIndex] = React.useState(0);

  const filteredCommands = COMMANDS.filter((cmd) =>
    cmd.label.toLowerCase().includes(query.toLowerCase())
  );

  // Reset selection when filter changes
  React.useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  // Reset query when palette opens
  React.useEffect(() => {
    if (isOpen) {
      setQuery("");
      setSelectedIndex(0);
    }
  }, [isOpen]);

  // Escape key handler
  React.useEffect(() => {
    if (!isOpen) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [isOpen, onClose]);

  const executeCommand = (cmd: PaletteCommand) => {
    onNavigate?.(cmd.target);
    onClose();
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((prev) => Math.min(prev + 1, filteredCommands.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((prev) => Math.max(prev - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (filteredCommands[selectedIndex]) {
        executeCommand(filteredCommands[selectedIndex]);
      }
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-20 bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-2xl bg-background rounded-lg shadow-2xl border">
        <div className="border-b px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Command className="w-5 h-5" />
            <h3 className="font-semibold">Command Palette</h3>
          </div>
          <Button variant="ghost" size="sm" onClick={onClose}>
            <X className="w-4 h-4" />
          </Button>
        </div>
        <div className="p-4 space-y-4">
          <div className="relative">
            <Input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Type a command or search..."
              autoFocus
              className="pl-10"
            />
            <Command className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          </div>

          <div className="space-y-1 max-h-80 overflow-y-auto">
            {filteredCommands.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                No commands found
              </div>
            ) : (
              filteredCommands.map((cmd, index) => (
                <div
                  key={cmd.id}
                  className={`flex items-center justify-between p-3 rounded-md cursor-pointer transition-colors ${
                    index === selectedIndex
                      ? "bg-accent text-accent-foreground"
                      : "hover:bg-accent/50"
                  }`}
                  onClick={() => executeCommand(cmd)}
                >
                  <span>{cmd.label}</span>
                  <Badge variant="secondary" className="text-xs">
                    {cmd.category}
                  </Badge>
                </div>
              ))
            )}
          </div>

          <div className="pt-4 border-t flex items-center justify-center gap-4 text-xs text-muted-foreground">
            <div className="flex items-center gap-1">
              <kbd className="px-1 py-0.5 bg-muted rounded border">↑</kbd>
              <kbd className="px-1 py-0.5 bg-muted rounded border">↓</kbd>
              <span>to navigate</span>
            </div>
            <div className="flex items-center gap-1">
              <kbd className="px-1 py-0.5 bg-muted rounded border">↵</kbd>
              <span>to select</span>
            </div>
            <div className="flex items-center gap-1">
              <kbd className="px-1 py-0.5 bg-muted rounded border">esc</kbd>
              <span>to close</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
