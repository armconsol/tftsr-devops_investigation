import React, { useEffect, useRef, useState } from "react";
import { X } from "lucide-react";
import { Terminal as XTerminal, type ITerminalOptions } from "xterm";
import { FitAddon } from "xterm-addon-fit";
import { WebLinksAddon } from "xterm-addon-web-links";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  startPtyExecSessionCmd,
  sendPtyStdinCmd,
  resizePtySessionCmd,
  terminatePtySessionCmd,
} from "@/lib/tauriCommands";

interface InteractiveShellModalProps {
  clusterId: string;
  namespace: string;
  pod: string;
  container?: string;
  onClose: () => void;
}

const XTERM_OPTIONS: ITerminalOptions = {
  cursorBlink: true,
  theme: {
    background: "#0f172a",
    foreground: "#4ade80",
    cursor: "#4ade80",
  },
  fontFamily: '"JetBrains Mono", "Fira Code", monospace',
  fontSize: 13,
  convertEol: true,
  rows: 24,
  cols: 80,
};

export function InteractiveShellModal({
  clusterId,
  namespace,
  pod,
  container,
  onClose,
}: InteractiveShellModalProps) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const unlistenOutputRef = useRef<UnlistenFn | null>(null);
  const unlistenClosedRef = useRef<UnlistenFn | null>(null);
  const unlistenErrorRef = useRef<UnlistenFn | null>(null);

  // Initialize terminal and start session
  useEffect(() => {
    if (!terminalRef.current) return;

    const term = new XTerminal(XTERM_OPTIONS);
    const fitAddon = new FitAddon();
    const webLinksAddon = new WebLinksAddon();

    term.loadAddon(fitAddon);
    term.loadAddon(webLinksAddon);
    term.open(terminalRef.current);

    try {
      fitAddon.fit();
    } catch {
      // Ignore first-frame race
    }

    xtermRef.current = term;
    fitAddonRef.current = fitAddon;

    // Start PTY session
    (async () => {
      try {
        term.write("\r\n\x1b[1;32mConnecting to pod...\x1b[0m\r\n");

        const sid = await startPtyExecSessionCmd(
          clusterId,
          namespace,
          pod,
          container
        );
        setSessionId(sid);

        // Listen for output from backend
        const unlistenOutput = await listen<number[]>(
          `terminal-output-${sid}`,
          (event) => {
            const data = new Uint8Array(event.payload);
            term.write(data);
          }
        );
        unlistenOutputRef.current = unlistenOutput;

        // Listen for session closed
        const unlistenClosed = await listen(`terminal-closed-${sid}`, () => {
          term.write("\r\n\x1b[1;31m[Session closed]\x1b[0m\r\n");
        });
        unlistenClosedRef.current = unlistenClosed;

        // Listen for errors
        const unlistenError = await listen<string>(
          `terminal-error-${sid}`,
          (event) => {
            term.write(`\r\n\x1b[1;31m[Error: ${event.payload}]\x1b[0m\r\n`);
          }
        );
        unlistenErrorRef.current = unlistenError;

        // Handle user input
        term.onData((data) => {
          if (sid) {
            const bytes = Array.from(new TextEncoder().encode(data));
            sendPtyStdinCmd(sid, bytes).catch((err) => {
              term.write(`\r\n\x1b[31mError sending input: ${err}\x1b[0m\r\n`);
            });
          }
        });

        // Handle terminal resize
        term.onResize((size) => {
          if (sid) {
            resizePtySessionCmd(sid, size.rows, size.cols).catch((err) => {
              console.error("Failed to resize PTY:", err);
            });
          }
        });
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setError(msg);
        term.write(`\r\n\x1b[1;31mFailed to start session: ${msg}\x1b[0m\r\n`);
      }
    })();

    // Cleanup on unmount
    return () => {
      if (unlistenOutputRef.current) {
        unlistenOutputRef.current();
      }
      if (unlistenClosedRef.current) {
        unlistenClosedRef.current();
      }
      if (unlistenErrorRef.current) {
        unlistenErrorRef.current();
      }
      if (sessionId) {
        terminatePtySessionCmd(sessionId).catch(console.error);
      }
      term.dispose();
      fitAddon.dispose();
    };
  }, [clusterId, namespace, pod, container]);

  // Handle window resize
  useEffect(() => {
    const handleResize = () => {
      if (fitAddonRef.current) {
        try {
          fitAddonRef.current.fit();
        } catch {
          // Ignore
        }
      }
    };

    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  const handleClose = () => {
    if (sessionId) {
      terminatePtySessionCmd(sessionId).catch(console.error);
    }
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70">
      <div className="w-[90vw] h-[85vh] bg-slate-900 rounded-lg shadow-2xl flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 bg-slate-800 border-b border-slate-700 rounded-t-lg">
          <div className="flex items-center gap-2">
            <span className="text-green-400 font-mono text-sm">
              kubectl exec -it {pod}
              {container && ` -c ${container}`} -- sh
            </span>
          </div>
          <button
            onClick={handleClose}
            className="text-slate-400 hover:text-red-400 transition-colors"
            aria-label="Close"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Error display */}
        {error && (
          <div className="px-4 py-2 bg-red-900/30 text-red-400 text-sm border-b border-red-900/50">
            {error}
          </div>
        )}

        {/* Terminal */}
        <div ref={terminalRef} className="flex-1 overflow-hidden bg-slate-950" />

        {/* Footer */}
        <div className="px-4 py-2 bg-slate-800 border-t border-slate-700 rounded-b-lg">
          <p className="text-xs text-slate-500">
            Interactive shell session - Press Ctrl+D or type "exit" to close
          </p>
        </div>
      </div>
    </div>
  );
}
