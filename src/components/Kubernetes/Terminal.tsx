import React from "react";
import { Terminal as TerminalIcon, X, Plus } from "lucide-react";
import { Button } from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";

interface TerminalSession {
  id: string;
  clusterId: string;
  namespace: string;
  pod: string;
  container: string;
  command: string;
}

interface TerminalProps {
  clusterId: string;
  namespace: string;
}

export function Terminal({ clusterId, namespace }: TerminalProps) {
  const [sessions, setSessions] = React.useState<TerminalSession[]>([]);
  const [activeSessionId, setActiveSessionId] = React.useState<string | null>(null);
  const [isCreating, setIsCreating] = React.useState(false);

  const terminalRefs = React.useRef<Record<string, { destroy: () => void }>>({});
  const containerRefs = React.useRef<Record<string, HTMLDivElement | null>>({});

  const addSession = React.useCallback(() => {
    setIsCreating(true);
    const newSession: TerminalSession = {
      id: `session-${Date.now()}`,
      clusterId,
      namespace: namespace === "all" ? "default" : namespace,
      pod: "",
      container: "",
      command: "bash",
    };
    setSessions((prev) => [...prev, newSession]);
    setActiveSessionId(newSession.id);
    setIsCreating(false);
  }, [clusterId, namespace]);

  const removeSession = (sessionId: string) => {
    setSessions((prev) => prev.filter((s) => s.id !== sessionId));
    if (activeSessionId === sessionId) {
      setActiveSessionId(null);
    }
    if (terminalRefs.current[sessionId]) {
      terminalRefs.current[sessionId].destroy();
      delete terminalRefs.current[sessionId];
    }
  };

  const resizeTerminal = (sessionId: string) => {
    const terminal = terminalRefs.current[sessionId];
    const container = containerRefs.current[sessionId];
    if (terminal && container) {
      // Placeholder for resize logic
      // Requires xterm-addon-fit dependency
    }
  };

  React.useEffect(() => {
    // Initialize with a default session
    if (sessions.length === 0 && !isCreating) {
      addSession();
    }
  }, [sessions.length, isCreating, addSession]);

  const initTerminal = (sessionId: string, element: HTMLDivElement | null) => {
    if (!element || terminalRefs.current[sessionId]) return;

    // Placeholder for terminal initialization
    // Requires xterm, xterm-addon-fit, xterm-addon-web-links dependencies
    const terminal = { destroy: () => {} };
    terminalRefs.current[sessionId] = terminal;
    containerRefs.current[sessionId] = element;

    // Handle resize
    window.addEventListener("resize", () => resizeTerminal(sessionId));
  };

  return (
    <div className="h-full overflow-hidden flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <TerminalIcon className="w-5 h-5" />
          <h2 className="text-xl font-semibold">Terminal</h2>
        </div>
        <Button onClick={addSession} disabled={isCreating}>
          <Plus className="w-4 h-4 mr-2" />
          New Terminal
        </Button>
      </div>

      {sessions.length === 0 ? (
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center space-y-4">
            <TerminalIcon className="w-16 h-16 mx-auto text-muted-foreground" />
            <p className="text-muted-foreground">No terminals open</p>
            <Button onClick={addSession}>
              <Plus className="w-4 h-4 mr-2" />
              Open Terminal
            </Button>
          </div>
        </div>
      ) : (
        <div className="flex-1 flex flex-col overflow-hidden">
          <Tabs value={activeSessionId || sessions[0]?.id} onValueChange={setActiveSessionId}>
            <TabsList className="grid grid-cols-10 mb-2">
              {sessions.map((session) => (
                <TabsTrigger
                  key={session.id}
                  value={session.id}
                  className="flex items-center gap-2"
                >
                  <span className="truncate max-w-[100px]">
                    {session.pod || "new"} / {session.container || "bash"}
                  </span>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      removeSession(session.id);
                    }}
                    className="hover:text-destructive"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </TabsTrigger>
              ))}
            </TabsList>

            {sessions.map((session) => (
              <TabsContent
                key={session.id}
                value={session.id}
                className="flex-1 overflow-hidden"
              >
                <div
                  ref={(el) => initTerminal(session.id, el)}
                  className="w-full h-full bg-slate-900 rounded-md overflow-hidden"
                />
              </TabsContent>
            ))}
          </Tabs>
        </div>
      )}
    </div>
  );
}
