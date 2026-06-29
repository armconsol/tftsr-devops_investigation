import React, { useState, useEffect, useRef, useCallback } from 'react';
import {
  startRdpSession,
  stopRdpSession,
  resizeRdpSession,
  getRemoteConnections,
  type RdpSession as RdpSessionType,
  type RemoteConnectionSummary,
} from '../../lib/tauriCommands';

export const RemoteDesktopPage: React.FC = () => {
  const [connections, setConnections] = useState<RemoteConnectionSummary[]>([]);
  const [activeSession, setActiveSession] = useState<RdpSessionType | null>(null);
  const [selectedConnection, setSelectedConnection] = useState<string>('');
  const [password, setPassword] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [autoResize, setAutoResize] = useState(true);
  const [customResolution, setCustomResolution] = useState('1920x1080');
  const [wsStatus, setWsStatus] = useState<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected');

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const resizeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const activeSessionRef = useRef<RdpSessionType | null>(null);

  // Keep ref in sync so resize callbacks always see the latest session
  useEffect(() => {
    activeSessionRef.current = activeSession;
  }, [activeSession]);

  const loadConnections = useCallback(async () => {
    try {
      const conns = await getRemoteConnections();
      setConnections(conns.filter((c: RemoteConnectionSummary) => c.protocol === 'rdp'));
    } catch (err) {
      setError('Failed to load connections');
      console.error(err);
    }
  }, []);

  useEffect(() => {
    loadConnections();
  }, [loadConnections]);

  // ── Dynamic resize ─────────────────────────────────────────────────────────

  const sendResize = useCallback((width: number, height: number) => {
    const session = activeSessionRef.current;
    if (!session) return;
    // Clamp to Display Control valid range
    const w = Math.max(200, Math.min(8192, width % 2 === 0 ? width : width - 1));
    const h = Math.max(200, Math.min(8192, height % 2 === 0 ? height : height - 1));
    resizeRdpSession(session.id, w, h).catch((e: unknown) =>
      console.warn('resize_rdp_session failed:', e)
    );
  }, []);

  const scheduleResize = useCallback(() => {
    if (!autoResize || !containerRef.current || !activeSessionRef.current) return;
    if (resizeTimerRef.current) clearTimeout(resizeTimerRef.current);
    resizeTimerRef.current = setTimeout(() => {
      const el = containerRef.current;
      if (!el) return;
      const { width, height } = el.getBoundingClientRect();
      sendResize(Math.floor(width), Math.floor(height));
    }, 150);
  }, [autoResize, sendResize]);

  // Window resize listener
  useEffect(() => {
    window.addEventListener('resize', scheduleResize);
    return () => {
      window.removeEventListener('resize', scheduleResize);
      if (resizeTimerRef.current) clearTimeout(resizeTimerRef.current);
    };
  }, [scheduleResize]);

  // ResizeObserver on the canvas container for panel/sidebar changes
  useEffect(() => {
    if (!containerRef.current) return;
    const ro = new ResizeObserver(() => scheduleResize());
    ro.observe(containerRef.current);
    return () => ro.disconnect();
  }, [scheduleResize]);

  // ── Connection ─────────────────────────────────────────────────────────────

  const connectToRDP = async () => {
    if (!selectedConnection) { setError('Please select a connection'); return; }
    setIsLoading(true);
    setError(null);
    try {
      const session = await startRdpSession(selectedConnection, password);
      setActiveSession(session);
      setWsStatus('connecting');

      const ws = new WebSocket(session.websocket_url);
      wsRef.current = ws;
      ws.binaryType = 'arraybuffer';

      ws.onopen = () => {
        setWsStatus('connected');
        // Fire an initial resize so the server knows our current canvas size
        if (autoResize && containerRef.current) {
          const { width, height } = containerRef.current.getBoundingClientRect();
          sendResize(Math.floor(width), Math.floor(height));
        }
      };

      ws.onmessage = (event) => {
        if (event.data instanceof ArrayBuffer) renderFrame(event.data);
      };

      ws.onerror = () => {
        setWsStatus('error');
        setError('WebSocket connection failed');
        wsRef.current?.close();
        wsRef.current = null;
      };

      ws.onclose = () => {
        setWsStatus('disconnected');
        const session = activeSessionRef.current;
        if (session) {
          stopRdpSession(session.id).catch(() => {});
        }
      };
    } catch (err) {
      console.error('Failed to start RDP session:', err);
      setError(err instanceof Error ? err.message : 'Failed to connect');
    } finally {
      setIsLoading(false);
    }
  };

  const disconnectFromRDP = async () => {
    if (resizeTimerRef.current) clearTimeout(resizeTimerRef.current);
    wsRef.current?.close();
    wsRef.current = null;
    if (activeSession) {
      try { await stopRdpSession(activeSession.id); } catch { /* ignore */ }
    }
    setActiveSession(null);
    setWsStatus('disconnected');
    const ctx = canvasRef.current?.getContext('2d');
    if (ctx && canvasRef.current) {
      ctx.fillStyle = '#000';
      ctx.fillRect(0, 0, canvasRef.current.width, canvasRef.current.height);
    }
  };

  // ── Frame rendering ────────────────────────────────────────────────────────

  const renderFrame = (frameData: ArrayBuffer) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const view = new DataView(frameData);
    const width = view.getUint32(0, true);
    const height = view.getUint32(4, true);
    if (canvas.width !== width || canvas.height !== height) {
      canvas.width = width;
      canvas.height = height;
    }
    const pixels = new Uint8ClampedArray(frameData, 8);
    ctx.putImageData(new ImageData(pixels, width, height), 0, 0);
  };

  // ── Input forwarding ───────────────────────────────────────────────────────

  const sendWsInput = (payload: Record<string, unknown>) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(payload));
    }
  };

  const handleMouseDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!canvasRef.current) return;
    const r = canvasRef.current.getBoundingClientRect();
    sendWsInput({ type: 'mouse', x: Math.floor(e.clientX - r.left), y: Math.floor(e.clientY - r.top), button: e.button, pressed: true });
  };
  const handleMouseUp = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!canvasRef.current) return;
    const r = canvasRef.current.getBoundingClientRect();
    sendWsInput({ type: 'mouse', x: Math.floor(e.clientX - r.left), y: Math.floor(e.clientY - r.top), button: e.button, pressed: false });
  };
  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!e.buttons) return;
    if (!canvasRef.current) return;
    const r = canvasRef.current.getBoundingClientRect();
    sendWsInput({ type: 'mouse_move', x: Math.floor(e.clientX - r.left), y: Math.floor(e.clientY - r.top) });
  };
  const handleKeyDown = (e: React.KeyboardEvent<HTMLCanvasElement>) => {
    e.preventDefault();
    sendWsInput({ type: 'keyboard', code: e.code, pressed: true });
  };
  const handleKeyUp = (e: React.KeyboardEvent<HTMLCanvasElement>) => {
    sendWsInput({ type: 'keyboard', code: e.code, pressed: false });
  };

  // ── Status helpers ─────────────────────────────────────────────────────────

  const wsColor = { connected: '#22c55e', connecting: '#eab308', error: '#ef4444', disconnected: '#6b7280' }[wsStatus];
  const wsLabel = { connected: 'Connected', connecting: 'Connecting…', error: 'Error', disconnected: 'Disconnected' }[wsStatus];

  // ── Render ─────────────────────────────────────────────────────────────────

  return (
    <div className="min-h-screen bg-gray-900 text-white p-6">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-3xl font-bold mb-6">Remote Desktop</h1>

        {!activeSession ? (
          <div className="bg-gray-800 rounded-lg p-6 max-w-md">
            <h2 className="text-xl font-semibold mb-4">Connect to Remote Desktop</h2>

            {error && (
              <div className="bg-red-500/20 border border-red-500 text-red-400 px-4 py-3 rounded mb-4">
                {error}
              </div>
            )}

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">Connection</label>
                <select
                  value={selectedConnection}
                  onChange={(e) => setSelectedConnection(e.target.value)}
                  className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="">Select a connection…</option>
                  {connections.map((c) => (
                    <option key={c.id} value={c.id}>{c.name} ({c.hostname})</option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Password</label>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Enter RDP password"
                  className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>

              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="autoResize"
                  checked={autoResize}
                  onChange={(e) => setAutoResize(e.target.checked)}
                  className="w-4 h-4"
                />
                <label htmlFor="autoResize" className="text-sm">Auto-resize to fit window</label>
              </div>

              {!autoResize && (
                <div>
                  <label className="block text-sm font-medium mb-1">Resolution</label>
                  <select
                    value={customResolution}
                    onChange={(e) => setCustomResolution(e.target.value)}
                    className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  >
                    {['800x600','1024x768','1280x720','1280x1024','1366x768',
                      '1440x900','1600x900','1920x1080','1920x1200','2560x1440'].map((r) => (
                      <option key={r} value={r}>{r.replace('x', ' × ')}</option>
                    ))}
                  </select>
                </div>
              )}

              <button
                onClick={connectToRDP}
                disabled={isLoading || !selectedConnection}
                className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white font-medium py-2 px-4 rounded transition-colors"
              >
                {isLoading ? 'Connecting…' : 'Connect'}
              </button>
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            <div className="flex items-center justify-between bg-gray-800 rounded-lg p-4">
              <div>
                <h2 className="text-xl font-semibold">{activeSession.connection_id}</h2>
                <p className="text-gray-400 text-sm">{activeSession.hostname}:{activeSession.port}</p>
              </div>
              <div className="flex items-center gap-4">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded-full" style={{ backgroundColor: wsColor }} />
                  <span className="text-sm">{wsLabel}</span>
                </div>
                <button
                  onClick={disconnectFromRDP}
                  disabled={isLoading}
                  className="bg-red-600 hover:bg-red-700 disabled:bg-gray-600 text-white font-medium py-2 px-4 rounded transition-colors"
                >
                  Disconnect
                </button>
              </div>
            </div>

            <div
              ref={containerRef}
              className="bg-black rounded-lg overflow-hidden"
              style={{ height: 'calc(100vh - 220px)' }}
            >
              <canvas
                ref={canvasRef}
                className="w-full h-full"
                tabIndex={0}
                onMouseDown={handleMouseDown}
                onMouseUp={handleMouseUp}
                onMouseMove={handleMouseMove}
                onKeyDown={handleKeyDown}
                onKeyUp={handleKeyUp}
              />
            </div>

            <div className="bg-gray-800 rounded-lg p-4 text-sm">
              <div className="grid grid-cols-3 gap-4">
                <div><span className="text-gray-400">Resolution:</span> {activeSession.resolution}</div>
                <div><span className="text-gray-400">SSH Tunnel:</span> {activeSession.ssh_enabled ? 'Yes' : 'No'}</div>
                <div><span className="text-gray-400">Auto-Resize:</span> {autoResize ? 'On' : 'Off'}</div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default RemoteDesktopPage;
