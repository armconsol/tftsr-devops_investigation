import React, { useEffect, useRef, useState, useCallback } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import type { NodeShellSession } from '@/lib/proxmoxClient';
import {
  buildLoginLine,
  encodeData,
  encodeResize,
  encodePing,
} from '@/lib/termproxy';
import { readClipboardText, writeClipboardText } from '@/lib/clipboard';
import { isCopyShortcut, isPasteShortcut } from '@/lib/consoleClipboard';

type Status = 'connecting' | 'connected' | 'disconnected' | 'error';

interface XtermConsoleProps {
  session: NodeShellSession;
  onStatusChange?: (status: Status) => void;
}

/**
 * xterm.js terminal bound to a Proxmox term-proxy websocket (used for PBS host
 * shells, which expose `termproxy` rather than a graphical `vncshell`).
 *
 * The backend has already started a local websocket proxy that injects the
 * correct auth cookie; here we speak the Proxmox term-proxy wire protocol over
 * that socket.
 */
export function XtermConsole({ session, onStatusChange }: XtermConsoleProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const pingRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const [status, setStatusState] = useState<Status>('connecting');

  const setStatus = useCallback(
    (s: Status) => {
      setStatusState(s);
      onStatusChange?.(s);
    },
    [onStatusChange]
  );

  useEffect(() => {
    const term = new Terminal({
      cursorBlink: true,
      fontFamily: 'monospace',
      fontSize: 13,
      theme: { background: '#000000' },
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    termRef.current = term;
    fitRef.current = fit;

    if (containerRef.current) {
      term.open(containerRef.current);
      fit.fit();
    }

    const ws = new WebSocket(session.localUrl);
    ws.binaryType = 'arraybuffer';
    wsRef.current = ws;

    // Ctrl/Cmd+Shift+C copies the current selection; Ctrl/Cmd+Shift+V pastes
    // the system clipboard into the session. Returning false stops xterm from
    // also forwarding the chord to the remote shell. All other keys (including
    // a bare Ctrl+C used to send SIGINT) pass straight through.
    term.attachCustomKeyEventHandler((e: KeyboardEvent): boolean => {
      if (e.type !== 'keydown') return true;
      if (isCopyShortcut(e)) {
        const selection = term.getSelection();
        if (selection) void writeClipboardText(selection);
        return false;
      }
      if (isPasteShortcut(e)) {
        void readClipboardText().then((text) => {
          if (text && ws.readyState === WebSocket.OPEN) ws.send(encodeData(text));
        });
        return false;
      }
      return true;
    });

    const decoder = new TextDecoder();

    ws.onopen = () => {
      // Authenticate, then announce the current terminal size.
      ws.send(buildLoginLine(session.user, session.ticket));
      ws.send(encodeResize(term.cols, term.rows));
      setStatus('connected');
      pingRef.current = setInterval(() => {
        if (ws.readyState === WebSocket.OPEN) ws.send(encodePing());
      }, 30000);
    };

    ws.onmessage = (ev: MessageEvent) => {
      if (typeof ev.data === 'string') {
        term.write(ev.data);
      } else if (ev.data instanceof ArrayBuffer) {
        term.write(decoder.decode(new Uint8Array(ev.data)));
      } else if (ev.data instanceof Blob) {
        ev.data.arrayBuffer().then((buf) => {
          term.write(decoder.decode(new Uint8Array(buf)));
        });
      }
    };

    ws.onerror = () => setStatus('error');
    ws.onclose = () => setStatus('disconnected');

    const dataDisposable = term.onData((data) => {
      if (ws.readyState === WebSocket.OPEN) ws.send(encodeData(data));
    });

    const resizeDisposable = term.onResize(({ cols, rows }) => {
      if (ws.readyState === WebSocket.OPEN) ws.send(encodeResize(cols, rows));
    });

    const handleWindowResize = () => {
      try {
        fit.fit();
      } catch {
        /* ignore */
      }
    };
    window.addEventListener('resize', handleWindowResize);

    return () => {
      window.removeEventListener('resize', handleWindowResize);
      if (pingRef.current) clearInterval(pingRef.current);
      dataDisposable.dispose();
      resizeDisposable.dispose();
      try {
        ws.close();
      } catch {
        /* ignore */
      }
      term.dispose();
      termRef.current = null;
      fitRef.current = null;
      wsRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [session.localUrl]);

  return (
    <div className="relative h-full w-full overflow-hidden rounded-md border bg-black">
      {status === 'connecting' && (
        <div className="absolute inset-0 z-10 flex items-center justify-center bg-black/60 text-sm text-white">
          Connecting…
        </div>
      )}
      <div ref={containerRef} className="h-full w-full" />
    </div>
  );
}
