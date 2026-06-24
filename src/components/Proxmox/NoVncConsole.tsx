import React, { useEffect, useRef, useState, useCallback } from 'react';
import RFB from '@novnc/novnc';
import { Button } from '@/components/ui/index';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/index';
import { Loader2, Power, RotateCcw, Keyboard, AlertCircle } from 'lucide-react';
import { openVncConsole, openLxcConsole } from '@/lib/proxmoxClient';

export type ConsoleKind = 'qemu' | 'lxc';

interface NoVncConsoleProps {
  clusterId: string;
  node: string;
  vmId: number;
  kind: ConsoleKind;
  title?: string;
}

type Status = 'connecting' | 'connected' | 'disconnected' | 'error';

/**
 * In-app noVNC graphical console. Requests a vncproxy session from the backend
 * (which also starts a local websocket proxy injecting the auth cookie), then
 * renders the live VNC stream onto a canvas via the noVNC RFB client.
 */
export function NoVncConsole({ clusterId, node, vmId, kind, title }: NoVncConsoleProps) {
  const screenRef = useRef<HTMLDivElement>(null);
  const rfbRef = useRef<RFB | null>(null);
  const [status, setStatus] = useState<Status>('connecting');
  const [error, setError] = useState<string>('');

  const cleanup = useCallback(() => {
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
    cleanup();
    setStatus('connecting');
    setError('');

    if (!clusterId || !node || !vmId) {
      setStatus('error');
      setError('Missing console parameters (cluster, node, or VM id).');
      return;
    }

    try {
      const session =
        kind === 'lxc'
          ? await openLxcConsole(clusterId, node, vmId)
          : await openVncConsole(clusterId, node, vmId);

      if (!screenRef.current) return;

      const rfb = new RFB(screenRef.current, session.local_url, {
        credentials: { password: session.ticket },
        shared: true,
      });
      rfb.scaleViewport = true;
      rfb.clipViewport = true;
      rfb.background = '#000000';

      rfb.addEventListener('connect', () => setStatus('connected'));
      rfb.addEventListener('disconnect', (e: Event) => {
        const detail = (e as CustomEvent<{ clean: boolean }>).detail;
        if (detail && !detail.clean) {
          setStatus('error');
          setError('Connection closed unexpectedly. The VNC proxy may be unreachable or the session ticket may have expired — try Reconnect.');
        } else {
          setStatus('disconnected');
        }
      });
      rfb.addEventListener('securityfailure', (e: Event) => {
        const detail = (e as CustomEvent<{ reason?: string }>).detail;
        setStatus('error');
        setError(`Authentication failed: ${detail?.reason ?? 'unknown reason'}`);
      });

      rfbRef.current = rfb;
    } catch (err) {
      setStatus('error');
      setError(`Failed to open console: ${err}`);
    }
  }, [clusterId, node, vmId, kind, cleanup]);

  useEffect(() => {
    connect();
    return cleanup;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [clusterId, node, vmId, kind]);

  const handleCtrlAltDel = useCallback(() => {
    rfbRef.current?.sendCtrlAltDel();
  }, []);

  return (
    <div className="flex h-full flex-col gap-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-sm">
          <span className="font-medium">{title ?? `Console — ${node} / VM ${vmId}`}</span>
          <StatusBadge status={status} />
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={handleCtrlAltDel} disabled={status !== 'connected'}>
            <Keyboard className="mr-2 h-4 w-4" />
            Ctrl+Alt+Del
          </Button>
          <Button variant="outline" size="sm" onClick={connect}>
            <RotateCcw className="mr-2 h-4 w-4" />
            Reconnect
          </Button>
          <Button variant="outline" size="sm" onClick={cleanup} disabled={status !== 'connected'}>
            <Power className="mr-2 h-4 w-4" />
            Disconnect
          </Button>
        </div>
      </div>

      {error && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertTitle>Console error</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      <div className="relative flex-1 overflow-hidden rounded-md border bg-black">
        {status === 'connecting' && (
          <div className="absolute inset-0 z-10 flex items-center justify-center bg-black/60 text-white">
            <Loader2 className="mr-2 h-5 w-5 animate-spin" />
            Connecting…
          </div>
        )}
        <div ref={screenRef} className="h-full w-full" />
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
