import React from "react";
import { Command, X } from "lucide-react";
import { Input } from "@/components/ui";
import { Button } from "@/components/ui";
import { Badge } from "@/components/ui";

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  onCommand: (command: string) => void;
}

export function CommandPalette({ isOpen, onClose, onCommand }: CommandPaletteProps) {
  const [query, setQuery] = React.useState("");

  if (!isOpen) return null;

  const commands = [
    { name: "Open Terminal", command: "terminal:open" },
    { name: "Create Pod", command: "resource:create:pod" },
    { name: "Create Deployment", command: "resource:create:deployment" },
    { name: "Create Service", command: "resource:create:service" },
    { name: "View Logs", command: "logs:view" },
    { name: "Scale Resource", command: "resource:scale" },
    { name: "Delete Resource", command: "resource:delete" },
    { name: "Export YAML", command: "yaml:export" },
    { name: "Refresh Cluster", command: "cluster:refresh" },
    { name: "Switch Context", command: "context:switch" },
  ];

  const filteredCommands = commands.filter((cmd) =>
    cmd.name.toLowerCase().includes(query.toLowerCase())
  );

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
              placeholder="Type a command or search..."
              autoFocus
              className="pl-10"
            />
            <Command className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          </div>

          <div className="space-y-2 max-h-80 overflow-y-auto">
            {filteredCommands.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                No commands found
              </div>
            ) : (
              filteredCommands.map((cmd, index) => (
                <div
                  key={index}
                  className="flex items-center justify-between p-3 hover:bg-accent rounded-md cursor-pointer transition-colors"
                  onClick={() => {
                    onCommand(cmd.command);
                    onClose();
                  }}
                >
                  <span>{cmd.name}</span>
                  <Badge variant="secondary" className="text-xs font-mono">
                    {cmd.command}
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
