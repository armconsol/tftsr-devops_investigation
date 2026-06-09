import React from "react";
import { Terminal as XTerminal, type ITerminalOptions } from "xterm";
import { FitAddon } from "xterm-addon-fit";
import { WebLinksAddon } from "xterm-addon-web-links";
import { Terminal as TerminalIcon, X, Plus, Settings } from "lucide-react";
import { execPodCmd } from "@/lib/tauriCommands";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  Button,
  Input,
} from "@/components/ui";

interface TerminalSession {
  id: string;
  clusterId: string;
  namespace: string;
  podName: string;
  containerName: string;
  shell: string;
  label: string;
}

interface TerminalProps {
  clusterId: string;
  namespace: string;
  podName?: string;
  containerName?: string;
}

interface TerminalSettings {
  copyOnSelect: boolean;
  fontFamily: string;
  fontSize: number;
}

const DEFAULT_SETTINGS: TerminalSettings = {
  copyOnSelect: false,
  fontFamily: '"JetBrains Mono", "Fira Code", monospace',
  fontSize: 13,
};

const STORAGE_KEY = "terminal-settings";

function loadSettings(): TerminalSettings {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      return { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
    }
  } catch {
    // Ignore parse errors
  }
  return DEFAULT_SETTINGS;
}

function saveSettings(settings: TerminalSettings): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}

function makeXtermOptions(settings: TerminalSettings): ITerminalOptions {
  return {
    cursorBlink: true,
    theme: {
      background: "#0f172a",
      foreground: "#4ade80",
      cursor: "#4ade80",
    },
    fontFamily: settings.fontFamily,
    fontSize: settings.fontSize,
    convertEol: true,
  };
}

function makeSessionId() {
  return `session-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
}

function makeLabel(podName: string, containerName: string) {
  return `${podName}/${containerName}`;
}

export function Terminal({ clusterId, namespace, podName, containerName }: TerminalProps) {
  const [sessions, setSessions] = React.useState<TerminalSession[]>([]);
  const [activeSessionId, setActiveSessionId] = React.useState<string | null>(null);
  const [sessionShells, setSessionShells] = React.useState<Record<string, string>>({});
  const [settings, setSettings] = React.useState<TerminalSettings>(loadSettings());
  const [settingsOpen, setSettingsOpen] = React.useState(false);

  const terminalRefs = React.useRef<Record<string, XTerminal>>({});
  const fitAddonRefs = React.useRef<Record<string, FitAddon>>({});
  const inputBuffers = React.useRef<Record<string, string>>({});
  // Keep a ref mirror of sessionShells so closures inside mountTerminal always
  // read the latest shell without needing to re-register onData on every change.
  const sessionShellsRef = React.useRef<Record<string, string>>({});

  // ── auto-create session when pod/container are provided as props ────────────
  React.useEffect(() => {
    if (podName && containerName && sessions.length === 0) {
      const id = makeSessionId();
      const session: TerminalSession = {
        id,
        clusterId,
        namespace: namespace === "all" ? "default" : namespace,
        podName,
        containerName,
        shell: "bash",
        label: makeLabel(podName, containerName),
      };
      setSessions([session]);
      setActiveSessionId(id);
      setSessionShells({ [id]: "bash" });
      sessionShellsRef.current = { [id]: "bash" };
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [podName, containerName, clusterId, namespace]);

  // ── resize all open terminals when the window resizes ──────────────────────
  React.useEffect(() => {
    const onResize = () => {
      Object.values(fitAddonRefs.current).forEach((fa) => {
        try { fa.fit(); } catch { /* ignore */ }
      });
    };
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  // ── dispose all terminals on unmount ────────────────────────────────────────
  React.useEffect(() => {
    // Capture ref snapshots for cleanup — stable Maps that accumulate over the
    // component lifetime; snapshot at cleanup time is intentional.
    const terms = terminalRefs.current;
    const fitAddons = fitAddonRefs.current;
    return () => {
      Object.entries(terms).forEach(([, term]) => term.dispose());
      Object.entries(fitAddons).forEach(([, fa]) => fa.dispose());
    };
  }, []);

  // ── dispose a single session's resources ────────────────────────────────────
  const disposeSession = React.useCallback((sessionId: string) => {
    terminalRefs.current[sessionId]?.dispose();
    fitAddonRefs.current[sessionId]?.dispose();
    delete terminalRefs.current[sessionId];
    delete fitAddonRefs.current[sessionId];
    delete inputBuffers.current[sessionId];
  }, []);

  // ── mount an xterm instance into a DOM element ──────────────────────────────
  const mountTerminal = React.useCallback(
    (sessionId: string, session: TerminalSession, element: HTMLDivElement) => {
      if (terminalRefs.current[sessionId]) return;

      const xtermOptions = makeXtermOptions(settings);
      const term = new XTerminal(xtermOptions);
      const fitAddon = new FitAddon();
      const webLinksAddon = new WebLinksAddon();

      term.loadAddon(fitAddon);
      term.loadAddon(webLinksAddon);
      term.open(element);

      // Copy-on-select functionality
      if (settings.copyOnSelect) {
        term.onSelectionChange(() => {
          const selection = term.getSelection();
          if (selection) {
            navigator.clipboard.writeText(selection).catch(() => {
              // Ignore clipboard errors
            });
          }
        });
      }

      try { fitAddon.fit(); } catch { /* first-frame race — safe to ignore */ }

      terminalRefs.current[sessionId] = term;
      fitAddonRefs.current[sessionId] = fitAddon;
      inputBuffers.current[sessionId] = "";

      term.write(`\r\n\x1b[1;32m$ Connected to ${session.podName}/${session.containerName}\x1b[0m\r\n$ `);

      term.onData((data: string) => {
        const buf = inputBuffers.current[sessionId] ?? "";

        if (data === "\r") {
          const cmd = buf.trim();
          inputBuffers.current[sessionId] = "";
          term.write("\r\n");

          if (cmd === "") {
            term.write("$ ");
            return;
          }

          const shell = sessionShellsRef.current[sessionId] ?? session.shell;
          execPodCmd(session.clusterId, session.namespace, session.podName, session.containerName, cmd, shell)
            .then((res) => {
              if (res.stdout) {
                term.write(res.stdout.replace(/\n/g, "\r\n"));
                if (!res.stdout.endsWith("\n")) term.write("\r\n");
              }
              if (res.stderr) {
                term.write(`\x1b[31m${res.stderr.replace(/\n/g, "\r\n")}\x1b[0m`);
                if (!res.stderr.endsWith("\n")) term.write("\r\n");
              }
              term.write("$ ");
            })
            .catch((err: unknown) => {
              const msg = err instanceof Error ? err.message : String(err);
              term.write(`\x1b[31mError: ${msg}\x1b[0m\r\n$ `);
            });
        } else if (data === "\x7f" || data === "\b") {
          if (buf.length > 0) {
            inputBuffers.current[sessionId] = buf.slice(0, -1);
            term.write("\b \b");
          }
        } else if (data >= " " || data === "\t") {
          inputBuffers.current[sessionId] = buf + data;
          term.write(data);
        }
      });
    },
    [settings] // Include settings to rebuild terminals with new config
  );

  // ── callback ref: fires when a container div is set/unset ──────────────────
  const setContainerRef = (session: TerminalSession) => (el: HTMLDivElement | null) => {
    if (el && !terminalRefs.current[session.id]) {
      mountTerminal(session.id, session, el);
    }
  };

  // ── session actions ─────────────────────────────────────────────────────────
  const addSession = () => {
    const id = makeSessionId();
    const session: TerminalSession = {
      id,
      clusterId,
      namespace: namespace === "all" ? "default" : namespace,
      podName: "",
      containerName: "",
      shell: "bash",
      label: "new",
    };
    setSessions((prev) => [...prev, session]);
    setActiveSessionId(id);
    sessionShellsRef.current = { ...sessionShellsRef.current, [id]: "bash" };
    setSessionShells((prev) => ({ ...prev, [id]: "bash" }));
  };

  const removeSession = (sessionId: string) => {
    disposeSession(sessionId);
    setSessions((prev) => {
      const next = prev.filter((s) => s.id !== sessionId);
      if (activeSessionId === sessionId) {
        setActiveSessionId(next[next.length - 1]?.id ?? null);
      }
      return next;
    });
    setSessionShells((prev) => {
      const next = { ...prev };
      delete next[sessionId];
      return next;
    });
  };

  const setShell = (sessionId: string, shell: string) => {
    sessionShellsRef.current = { ...sessionShellsRef.current, [sessionId]: shell };
    setSessionShells((prev) => ({ ...prev, [sessionId]: shell }));
  };

  const updateSettings = (newSettings: Partial<TerminalSettings>) => {
    const updated = { ...settings, ...newSettings };
    setSettings(updated);
    saveSettings(updated);

    // Apply settings to all existing terminals
    Object.entries(terminalRefs.current).forEach(([, term]) => {
      term.options.fontFamily = updated.fontFamily;
      term.options.fontSize = updated.fontSize;
    });

    // Fit all terminals after font changes
    Object.values(fitAddonRefs.current).forEach((fa) => {
      try { fa.fit(); } catch { /* ignore */ }
    });
  };

  // ── empty state ─────────────────────────────────────────────────────────────
  if (sessions.length === 0) {
    return (
      <div className="h-full flex items-center justify-center bg-slate-950 rounded-lg">
        <div className="text-center space-y-4">
          <TerminalIcon className="w-16 h-16 mx-auto text-green-600 opacity-40" />
          <p className="text-green-500 text-sm">Select a pod to connect</p>
        </div>
      </div>
    );
  }

  const currentShell = activeSessionId ? (sessionShells[activeSessionId] ?? "bash") : "bash";

  return (
    <div className="h-full overflow-hidden flex flex-col bg-slate-950 rounded-lg">
      {/* Tab bar */}
      <div className="flex items-center gap-1 px-2 pt-2 bg-slate-900 border-b border-slate-700 flex-shrink-0">
        {sessions.map((session) => (
          <button
            key={session.id}
            role="tab"
            aria-selected={activeSessionId === session.id}
            onClick={() => setActiveSessionId(session.id)}
            className={`flex items-center gap-1 px-3 py-1.5 rounded-t text-xs font-mono transition-colors
              ${
                activeSessionId === session.id
                  ? "bg-slate-950 text-green-400 border border-b-0 border-slate-600"
                  : "text-slate-400 hover:text-slate-200"
              }`}
          >
            <span className="truncate max-w-[120px]">{session.label}</span>
            <button
              aria-label="close"
              onClick={(e) => {
                e.stopPropagation();
                removeSession(session.id);
              }}
              className="ml-1 hover:text-red-400 transition-colors"
            >
              <X className="w-3 h-3" />
            </button>
          </button>
        ))}

        <button
          aria-label="add session"
          onClick={addSession}
          className="p-1.5 text-slate-400 hover:text-green-400 transition-colors"
        >
          <Plus className="w-4 h-4" />
        </button>

        {activeSessionId && (
          <div className="ml-auto pr-2 flex items-center gap-2">
            <select
              value={currentShell}
              onChange={(e) => setShell(activeSessionId, e.target.value)}
              className="bg-slate-800 text-slate-300 text-xs rounded border border-slate-600 px-2 py-0.5 focus:outline-none focus:ring-1 focus:ring-green-500"
            >
              <option value="bash">bash</option>
              <option value="sh">sh</option>
              <option value="zsh">zsh</option>
            </select>
            <button
              aria-label="settings"
              onClick={() => setSettingsOpen(true)}
              className="p-1.5 text-slate-400 hover:text-green-400 transition-colors"
            >
              <Settings className="w-4 h-4" />
            </button>
          </div>
        )}
      </div>

      {/* Terminal panes */}
      <div className="flex-1 overflow-hidden">
        {sessions.map((session) => (
          <div
            key={session.id}
            className={`w-full h-full ${activeSessionId === session.id ? "block" : "hidden"}`}
          >
            <div
              ref={setContainerRef(session)}
              className="w-full h-full bg-slate-950"
            />
          </div>
        ))}
      </div>

      {/* Settings Dialog */}
      <Dialog open={settingsOpen} onOpenChange={setSettingsOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Terminal Settings</DialogTitle>
          </DialogHeader>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <label htmlFor="copy-on-select" className="text-sm font-medium">
                Copy on Select
              </label>
              <input
                id="copy-on-select"
                type="checkbox"
                checked={settings.copyOnSelect}
                onChange={(e) => updateSettings({ copyOnSelect: e.target.checked })}
                className="rounded border-input"
              />
            </div>

            <div className="space-y-2">
              <label htmlFor="font-family" className="text-sm font-medium block">
                Font Family
              </label>
              <Input
                id="font-family"
                type="text"
                value={settings.fontFamily}
                onChange={(e) => updateSettings({ fontFamily: e.target.value })}
                placeholder="e.g., monospace, Courier New"
              />
            </div>

            <div className="space-y-2">
              <label htmlFor="font-size" className="text-sm font-medium block">
                Font Size
              </label>
              <Input
                id="font-size"
                type="number"
                min={8}
                max={24}
                value={settings.fontSize}
                onChange={(e) => updateSettings({ fontSize: Number(e.target.value) })}
              />
            </div>

            <div className="flex justify-end gap-2">
              <Button variant="outline" onClick={() => setSettingsOpen(false)}>
                Close
              </Button>
              <Button
                onClick={() => {
                  updateSettings(DEFAULT_SETTINGS);
                  setSettingsOpen(false);
                }}
              >
                Reset to Defaults
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
