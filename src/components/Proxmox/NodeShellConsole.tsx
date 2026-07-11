import React, { useEffect, useRef, useState, useCallback } from 'react';
import RFB from '@novnc/novnc';
import { Button } from '@/components/ui/index';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/index';
import { Loader2, Power, RotateCcw, Clipboard, AlertCircle } from 'lucide-react';
import { openNodeShell, type NodeShellSession } from '@/lib/proxmoxClient';
import { XtermConsole } from './XtermConsole';
import { readClipboardText, writeClipboardText } from '@/lib/clipboard';
import { isPasteShortcut } from '@/lib/consoleClipboard';

type Status = 'connecting' | 'connected' | 'disconnected' | 'error';

interface NodeShellConsoleProps {
  clusterId: string;
  node: string;
  title?: string;
  /** Shell command PVE should run: "login" (default) or "upgrade" (pveupgrade). */
  cmd?: 'login' | 'upgrade';
}

/**
 * Host (node) shell console for a stored remote.
 *
 * Requests a tagged shell session from the backend and renders either:
 *  - a graphical noVNC canvas (PVE `vncshell`), or
 *  - an xterm.js terminal (PBS `termproxy`).
 */
export function NodeShellConsole({ clusterId, node, title, cmd }: NodeShellConsoleProps) {
  const screenRef = useRef<HTMLDivElement>(null);
  const rfbRef = useRef<RFB | null>(null);
  const [session, setSession] = useState<NodeShellSession | null>(null);
  const [status, setStatus] = useState<Status>('connecting');
  const [error, setError] = useState<string>('');

  const cleanupRfb = useCallback(() => {
    if (rfbRef.current) {
      try {
        rfbRef.current.disconnect();
      } catch {
        /* ignore */
      }
      rfbRef.current = null;
    }
  }, []);

  const connect = useCallback(async () => {
    cleanupRfb();
    setSession(null);
    setStatus('connecting');
    setError('');

    if (!clusterId || !node) {
      setStatus('error');
      setError('Missing shell parameters (cluster or node).');
      return;
    }

    try {
      const s = await openNodeShell(clusterId, node, cmd);
      setSession(s);
    } catch (err) {
      setStatus('error');
      setError(`Failed to open shell: ${err}`);
    }
  }, [clusterId, node, cmd, cleanupRfb]);

  useEffect(() => {
    connect();
    return cleanupRfb;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [clusterId, node, cmd]);

  // Host → guest paste for the graphical (noVNC) shell. The xterm branch wires
  // its own paste handler. Bound to a button and Ctrl/Cmd+Shift+V.
  const handlePaste = useCallback(async () => {
    if (!rfbRef.current || status !== 'connected' || session?.kind !== 'novnc') return;
    const text = await readClipboardText();
    if (text) rfbRef.current.clipboardPasteFrom(text);
  }, [status, session]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (session?.kind === 'novnc' && isPasteShortcut(e)) {
        e.preventDefault();
        void handlePaste();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [handlePaste, session]);

  // noVNC rendering for PVE graphical shells.
  useEffect(() => {
    if (!session || session.kind !== 'novnc' || !screenRef.current) return;
    cleanupRfb();
    try {
      const rfb = new RFB(screenRef.current, session.localUrl, {
        credentials: { password: session.password ?? session.ticket },
        shared: true,
      });
      rfb.scaleViewport = true;
      rfb.clipViewport = true;
      rfb.background = '#000000';
      rfb.addEventListener('connect', () => setStatus('connected'));
      rfb.addEventListener('disconnect', (e: Event) => {
        const detail = (e as CustomEvent<{ clean: boolean }>).detail;
        setStatus('disconnected');
        if (detail && !detail.clean) setError('Connection closed unexpectedly.');
      });
      rfb.addEventListener('securityfailure', (e: Event) => {
        const detail = (e as CustomEvent<{ reason?: string }>).detail;
        setStatus('error');
        setError(`Authentication failed: ${detail?.reason ?? 'unknown reason'}`);
      });
      // Guest → host copy: mirror the remote clipboard to the system clipboard.
      rfb.addEventListener('clipboard', (e: Event) => {
        const detail = (e as CustomEvent<{ text?: string }>).detail;
        if (detail?.text) void writeClipboardText(detail.text);
      });
      rfbRef.current = rfb;
    } catch (err) {
      setStatus('error');
      setError(`Failed to render console: ${err}`);
    }
    return cleanupRfb;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [session]);

  return (
    <div className="flex h-full flex-col gap-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-sm">
          <span className="font-medium">{title ?? `Shell — ${node}`}</span>
          <StatusBadge status={status} />
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={connect}>
            <RotateCcw className="mr-2 h-4 w-4" />
            Reconnect
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={handlePaste}
            disabled={status !== 'connected' || session?.kind !== 'novnc'}
            title="Paste clipboard into shell (Ctrl+Shift+V)"
          >
            <Clipboard className="mr-2 h-4 w-4" />
            Paste
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={cleanupRfb}
            disabled={status !== 'connected' || session?.kind !== 'novnc'}
          >
            <Power className="mr-2 h-4 w-4" />
            Disconnect
          </Button>
        </div>
      </div>

      {error && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertTitle>Shell error</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      <div className="relative min-h-0 flex-1 overflow-hidden rounded-md border bg-black">
        {!session && status === 'connecting' && (
          <div className="absolute inset-0 z-10 flex items-center justify-center bg-black/60 text-white">
            <Loader2 className="mr-2 h-5 w-5 animate-spin" />
            Opening shell…
          </div>
        )}
        {session && session.kind === 'xterm' ? (
          <XtermConsole session={session} onStatusChange={setStatus} />
        ) : (
          <div ref={screenRef} className="h-full w-full" />
        )}
      </div>
    </div>
  );
}

function StatusBadge({ status }: { status: Status }) {
  const map: Record<Status, { label: string; cls: string }> = {
    connecting: { label: 'Connecting', cls: 'bg-amber-100 text-amber-700' },
    connected: { label: 'Connected', cls: 'bg-green-100 text-green-700' },
    disconnected: { label: 'Disconnected', cls: 'bg-gray-100 text-gray-600' },
    error: { label: 'Error', cls: 'bg-red-100 text-red-700' },
  };
  const { label, cls } = map[status];
  return <span className={`rounded-full px-2 py-0.5 text-xs ${cls}`}>{label}</span>;
}
