import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Plus, Pencil, Trash2, Monitor, Server, KeyRound, ChevronRight } from 'lucide-react';
import {
  startRdpSession,
  stopRdpSession,
  resizeRdpSession,
  getRemoteConnections,
  addRemoteConnectionCmd,
  updateRemoteConnectionCmd,
  deleteRemoteConnectionCmd,
  getRemoteConnectionCmd,
  type RdpSession as RdpSessionType,
  type RemoteConnectionSummary,
  type NewRemoteConnection,
  type RemoteConnectionUpdate,
  type RemoteConnection,
} from '../../lib/tauriCommands';
import {
  Button,
  Input,
  Label,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
  Switch,
  Alert,
  AlertDescription,
} from '@/components/ui/index';

// ─── Connection Form ──────────────────────────────────────────────────────────

interface ConnectionFormData {
  name: string;
  protocol: 'rdp' | 'vnc';
  hostname: string;
  port: string;
  username: string;
  password: string;
  domain: string;
  ssh_enabled: boolean;
  ssh_hostname: string;
  ssh_port: string;
  ssh_username: string;
  ssh_password: string;
  ssh_key_data: string;
  ssh_key_passphrase: string;
  resolution: string;
  auto_resize: boolean;
  stretch_to_fill: boolean;
}

const defaultForm = (): ConnectionFormData => ({
  name: '',
  protocol: 'rdp',
  hostname: '',
  port: '3389',
  username: '',
  password: '',
  domain: '',
  ssh_enabled: false,
  ssh_hostname: '',
  ssh_port: '22',
  ssh_username: '',
  ssh_password: '',
  ssh_key_data: '',
  ssh_key_passphrase: '',
  resolution: '1920x1080',
  auto_resize: true,
  stretch_to_fill: false,
});

const RESOLUTIONS = [
  '800x600', '1024x768', '1280x720', '1280x1024',
  '1366x768', '1440x900', '1600x900', '1920x1080',
  '1920x1200', '2560x1440',
];

interface ConnectionFormProps {
  initial?: Partial<ConnectionFormData>;
  onSave: (data: ConnectionFormData) => Promise<void>;
  onCancel: () => void;
  title: string;
  isEdit?: boolean;
}

export function ConnectionForm({ initial, onSave, onCancel, title, isEdit = false }: ConnectionFormProps) {
  const [form, setForm] = useState<ConnectionFormData>({ ...defaultForm(), ...initial });
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState('connection');

  const set = <K extends keyof ConnectionFormData>(key: K, value: ConnectionFormData[K]) =>
    setForm((f) => ({ ...f, [key]: value }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    if (!form.name.trim()) { setError('Name is required'); return; }
    if (!form.hostname.trim()) { setError('Hostname is required'); return; }
    if (!isEdit && !form.password.trim()) { setError('Password is required'); return; }
    if (form.ssh_enabled && !form.ssh_hostname.trim()) { setError('SSH hostname is required when tunnel is enabled'); return; }
    setSaving(true);
    try {
      await onSave(form);
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <DialogHeader>
        <DialogTitle>{title}</DialogTitle>
      </DialogHeader>
      <div className="py-4 space-y-4 max-h-[65vh] overflow-y-auto pr-1">
        {error && (
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList>
            <TabsTrigger value="connection">Connection</TabsTrigger>
            <TabsTrigger value="ssh">SSH Tunnel</TabsTrigger>
            <TabsTrigger value="display">Display</TabsTrigger>
          </TabsList>

          <TabsContent value="connection">
            <div className="space-y-3 pt-3">
              <div className="space-y-1">
                <Label htmlFor="cf-name">Name</Label>
                <Input id="cf-name" value={form.name} onChange={(e) => set('name', e.target.value)} placeholder="My RDP Server" />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <Label>Protocol</Label>
                  <Select value={form.protocol} onValueChange={(v) => {
                    set('protocol', v as 'rdp' | 'vnc');
                    set('port', v === 'rdp' ? '3389' : '5900');
                  }}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="rdp">RDP</SelectItem>
                      <SelectItem value="vnc">VNC</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-1">
                  <Label htmlFor="cf-port">Port</Label>
                  <Input id="cf-port" type="number" value={form.port} onChange={(e) => set('port', e.target.value)} />
                </div>
              </div>
              <div className="space-y-1">
                <Label htmlFor="cf-hostname">Hostname / IP</Label>
                <Input id="cf-hostname" value={form.hostname} onChange={(e) => set('hostname', e.target.value)} placeholder="192.168.1.100" />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <Label htmlFor="cf-username">Username</Label>
                  <Input id="cf-username" value={form.username} onChange={(e) => set('username', e.target.value)} placeholder="Administrator" />
                </div>
                <div className="space-y-1">
                  <Label htmlFor="cf-domain">Domain</Label>
                  <Input id="cf-domain" value={form.domain} onChange={(e) => set('domain', e.target.value)} placeholder="CORP" />
                </div>
              </div>
              <div className="space-y-1">
                <Label htmlFor="cf-password">Password{isEdit && <span className="text-muted-foreground font-normal"> (leave blank to keep existing)</span>}</Label>
                <Input id="cf-password" type="password" value={form.password} onChange={(e) => set('password', e.target.value)} placeholder={isEdit ? '••••••••' : ''} />
              </div>
            </div>
          </TabsContent>

          <TabsContent value="ssh">
            <div className="space-y-3 pt-3">
              <div className="flex items-center gap-2">
                <Switch
                  checked={form.ssh_enabled}
                  onCheckedChange={(v) => set('ssh_enabled', v)}
                />
                <Label>Enable SSH tunnel</Label>
              </div>
              {form.ssh_enabled && (
                <>
                  <div className="grid grid-cols-3 gap-3">
                    <div className="col-span-2 space-y-1">
                      <Label htmlFor="cf-ssh-hostname">SSH Host</Label>
                      <Input id="cf-ssh-hostname" value={form.ssh_hostname} onChange={(e) => set('ssh_hostname', e.target.value)} placeholder="bastion.example.com" />
                    </div>
                    <div className="space-y-1">
                      <Label htmlFor="cf-ssh-port">SSH Port</Label>
                      <Input id="cf-ssh-port" type="number" value={form.ssh_port} onChange={(e) => set('ssh_port', e.target.value)} />
                    </div>
                  </div>
                  <div className="space-y-1">
                    <Label htmlFor="cf-ssh-username">SSH Username</Label>
                    <Input id="cf-ssh-username" value={form.ssh_username} onChange={(e) => set('ssh_username', e.target.value)} />
                  </div>
                  <div className="space-y-1">
                    <Label htmlFor="cf-ssh-password">SSH Password (optional)</Label>
                    <Input id="cf-ssh-password" type="password" value={form.ssh_password} onChange={(e) => set('ssh_password', e.target.value)} placeholder={isEdit ? '••••••••' : ''} />
                  </div>
                  <div className="space-y-1">
                    <Label htmlFor="cf-ssh-key">SSH Private Key (PEM, optional)</Label>
                    <textarea
                      id="cf-ssh-key"
                      className="w-full min-h-[80px] px-3 py-2 text-sm rounded-md border border-input bg-background font-mono resize-y"
                      value={form.ssh_key_data}
                      onChange={(e) => set('ssh_key_data', e.target.value)}
                      placeholder="-----BEGIN OPENSSH PRIVATE KEY-----"
                    />
                  </div>
                  <div className="space-y-1">
                    <Label htmlFor="cf-ssh-passphrase">Key Passphrase (optional)</Label>
                    <Input id="cf-ssh-passphrase" type="password" value={form.ssh_key_passphrase} onChange={(e) => set('ssh_key_passphrase', e.target.value)} placeholder={isEdit ? '••••••••' : ''} />
                  </div>
                  {isEdit && (
                    <p className="text-xs text-muted-foreground">SSH credential fields are write-only. Leave blank to keep existing stored values.</p>
                  )}
                </>
              )}
            </div>
          </TabsContent>

          <TabsContent value="display">
            <div className="space-y-3 pt-3">
              <div className="flex items-center gap-2">
                <Switch
                  checked={form.auto_resize}
                  onCheckedChange={(v) => set('auto_resize', v)}
                />
                <Label>Auto-resize to window</Label>
              </div>
              {!form.auto_resize && (
                <div className="space-y-1">
                  <Label>Resolution</Label>
                  <Select value={form.resolution} onValueChange={(v) => set('resolution', v)}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      {RESOLUTIONS.map((r) => (
                        <SelectItem key={r} value={r}>{r.replace('x', ' × ')}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              )}
              <div className="flex items-center gap-2">
                <Switch
                  checked={form.stretch_to_fill}
                  onCheckedChange={(v) => set('stretch_to_fill', v)}
                />
                <Label>Stretch to fill</Label>
              </div>
            </div>
          </TabsContent>
        </Tabs>
      </div>
      <DialogFooter>
        <Button type="button" variant="outline" onClick={onCancel}>Cancel</Button>
        <Button type="submit" disabled={saving}>{saving ? 'Saving…' : 'Save'}</Button>
      </DialogFooter>
    </form>
  );
}

// ─── Connection List ──────────────────────────────────────────────────────────

interface ConnectionListProps {
  connections: RemoteConnectionSummary[];
  onConnect: (id: string) => void;
  onEdit: (id: string) => void;
  onDelete: (id: string) => void;
  onAdd: () => void;
}

function ConnectionList({ connections, onConnect, onEdit, onDelete, onAdd }: ConnectionListProps) {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Saved Connections</h2>
        <Button size="sm" onClick={onAdd}>
          <Plus className="w-4 h-4 mr-1" /> Add Connection
        </Button>
      </div>

      {connections.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 text-muted-foreground gap-3">
          <Monitor className="w-12 h-12 opacity-30" />
          <p className="text-sm">No connections saved yet.</p>
          <Button size="sm" variant="outline" onClick={onAdd}>
            <Plus className="w-4 h-4 mr-1" /> Add your first connection
          </Button>
        </div>
      ) : (
        <div className="grid gap-2">
          {connections.map((c) => (
            <div
              key={c.id}
              className="flex items-center justify-between p-3 rounded-lg border bg-card hover:bg-accent/30 transition-colors group"
            >
              <div className="flex items-center gap-3 min-w-0">
                <div className="w-9 h-9 rounded-md bg-primary/10 flex items-center justify-center shrink-0">
                  {c.protocol === 'rdp' ? (
                    <Monitor className="w-4 h-4 text-primary" />
                  ) : (
                    <Server className="w-4 h-4 text-primary" />
                  )}
                </div>
                <div className="min-w-0">
                  <p className="font-medium text-sm truncate">{c.name}</p>
                  <p className="text-xs text-muted-foreground truncate">
                    {c.protocol.toUpperCase()} · {c.hostname}:{c.port}
                    {c.ssh_enabled && <span className="ml-1 inline-flex items-center gap-0.5"><KeyRound className="w-3 h-3" /> SSH</span>}
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                <Button size="sm" variant="ghost" onClick={() => onEdit(c.id)} title="Edit">
                  <Pencil className="w-3.5 h-3.5" />
                </Button>
                <Button size="sm" variant="ghost" onClick={() => onDelete(c.id)} title="Delete" className="text-destructive hover:text-destructive">
                  <Trash2 className="w-3.5 h-3.5" />
                </Button>
                <Button size="sm" onClick={() => onConnect(c.id)}>
                  Connect <ChevronRight className="w-3.5 h-3.5 ml-1" />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

// ─── RDP Canvas View ──────────────────────────────────────────────────────────

interface SessionViewProps {
  session: RdpSessionType;
  onDisconnect: () => void;
}

function SessionView({ session, onDisconnect }: SessionViewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const resizeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const sessionRef = useRef(session);
  const [wsStatus, setWsStatus] = useState<'connecting' | 'connected' | 'error' | 'disconnected'>('connecting');

  useEffect(() => { sessionRef.current = session; }, [session]);

  const sendResize = useCallback((width: number, height: number) => {
    const w = Math.max(200, Math.min(8192, width % 2 === 0 ? width : width - 1));
    const h = Math.max(200, Math.min(8192, height % 2 === 0 ? height : height - 1));
    resizeRdpSession(sessionRef.current.id, w, h).catch(() => {});
  }, []);

  const scheduleResize = useCallback(() => {
    if (!containerRef.current) return;
    if (resizeTimerRef.current) clearTimeout(resizeTimerRef.current);
    resizeTimerRef.current = setTimeout(() => {
      const el = containerRef.current;
      if (!el) return;
      const { width, height } = el.getBoundingClientRect();
      sendResize(Math.floor(width), Math.floor(height));
    }, 150);
  }, [sendResize]);

  useEffect(() => {
    window.addEventListener('resize', scheduleResize);
    return () => {
      window.removeEventListener('resize', scheduleResize);
      if (resizeTimerRef.current) clearTimeout(resizeTimerRef.current);
    };
  }, [scheduleResize]);

  useEffect(() => {
    if (!containerRef.current) return;
    const ro = new ResizeObserver(() => scheduleResize());
    ro.observe(containerRef.current);
    return () => ro.disconnect();
  }, [scheduleResize]);

  useEffect(() => {
    const ws = new WebSocket(session.websocket_url);
    wsRef.current = ws;
    ws.binaryType = 'arraybuffer';

    ws.onopen = () => {
      setWsStatus('connected');
      if (containerRef.current) {
        const { width, height } = containerRef.current.getBoundingClientRect();
        sendResize(Math.floor(width), Math.floor(height));
      }
    };

    ws.onmessage = (event) => {
      if (!(event.data instanceof ArrayBuffer)) return;
      const canvas = canvasRef.current;
      if (!canvas) return;
      const ctx = canvas.getContext('2d');
      if (!ctx) return;
      const view = new DataView(event.data);
      const width = view.getUint32(0, true);
      const height = view.getUint32(4, true);
      if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
      }
      ctx.putImageData(new ImageData(new Uint8ClampedArray(event.data, 8), width, height), 0, 0);
    };

    ws.onerror = () => setWsStatus('error');
    ws.onclose = () => {
      setWsStatus('disconnected');
    };

    return () => {
      ws.close();
      // Do NOT stop the RDP session here - it should only be stopped when
      // the user explicitly disconnects. The WebSocket might reconnect,
      // and stopping the session prematurely causes the black screen issue.
    };
  }, [session.websocket_url]);

  const sendInput = (payload: Record<string, unknown>) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(payload));
    }
  };

  // Map a pointer event from on-screen canvas pixels to the RDP framebuffer's
  // native resolution. The canvas is stretched by CSS (`w-full h-full`) while its
  // backing store stays at the RDP width/height, so coordinates must be scaled or
  // clicks land in the wrong place.
  const toRemoteCoords = (e: React.MouseEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return null;
    const r = canvas.getBoundingClientRect();
    if (r.width === 0 || r.height === 0) return null;
    const x = Math.round((e.clientX - r.left) * (canvas.width / r.width));
    const y = Math.round((e.clientY - r.top) * (canvas.height / r.height));
    return { x, y };
  };

  const statusColor = { connected: '#22c55e', connecting: '#eab308', error: '#ef4444', disconnected: '#6b7280' }[wsStatus];
  const statusLabel = { connected: 'Connected', connecting: 'Connecting…', error: 'Error', disconnected: 'Disconnected' }[wsStatus];

  return (
    <div className="flex flex-col h-full gap-2">
      <div className="flex items-center justify-between bg-card border rounded-lg px-4 py-2 shrink-0">
        <div>
          <span className="font-medium text-sm">{session.hostname}:{session.port}</span>
          <span className="text-xs text-muted-foreground ml-2">· {session.resolution}</span>
          {session.ssh_enabled && <span className="text-xs text-muted-foreground ml-2">· SSH tunnel</span>}
        </div>
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1.5">
            <div className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: statusColor }} />
            <span className="text-xs">{statusLabel}</span>
          </div>
          <Button size="sm" variant="destructive" onClick={onDisconnect}>Disconnect</Button>
        </div>
      </div>

      <div ref={containerRef} className="flex-1 bg-black rounded-lg overflow-hidden">
        <canvas
          ref={canvasRef}
          className="w-full h-full"
          tabIndex={0}
          onMouseDown={(e) => {
            canvasRef.current?.focus();
            const c = toRemoteCoords(e);
            if (c) sendInput({ type: 'mouse', x: c.x, y: c.y, button: e.button, pressed: true });
          }}
          onMouseUp={(e) => {
            const c = toRemoteCoords(e);
            if (c) sendInput({ type: 'mouse', x: c.x, y: c.y, button: e.button, pressed: false });
          }}
          onMouseMove={(e) => {
            const c = toRemoteCoords(e);
            if (c) sendInput({ type: 'mouse_move', x: c.x, y: c.y });
          }}
          onWheel={(e) => {
            const c = toRemoteCoords(e);
            if (c) sendInput({ type: 'wheel', x: c.x, y: c.y, delta: e.deltaY });
          }}
          onContextMenu={(e) => e.preventDefault()}
          onKeyDown={(e) => { e.preventDefault(); sendInput({ type: 'keyboard', code: e.code, pressed: true }); }}
          onKeyUp={(e) => { e.preventDefault(); sendInput({ type: 'keyboard', code: e.code, pressed: false }); }}
        />
      </div>
    </div>
  );
}

// ─── Main Page ────────────────────────────────────────────────────────────────

export const RemoteDesktopPage: React.FC = () => {
  const [connections, setConnections] = useState<RemoteConnectionSummary[]>([]);
  const [activeSession, setActiveSession] = useState<RdpSessionType | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [pageError, setPageError] = useState<string | null>(null);

  const [showAddDialog, setShowAddDialog] = useState(false);
  const [editTarget, setEditTarget] = useState<RemoteConnection | null>(null);
  const [deleteTargetId, setDeleteTargetId] = useState<string | null>(null);
  const [connectPassword, setConnectPassword] = useState('');
  const [connectTargetId, setConnectTargetId] = useState<string | null>(null);
  const [connectError, setConnectError] = useState<string | null>(null);

  const loadConnections = useCallback(async () => {
    try {
      const conns = await getRemoteConnections();
      setConnections(conns);
    } catch (err) {
      setPageError('Failed to load connections');
      console.error(err);
    }
  }, []);

  useEffect(() => { loadConnections(); }, [loadConnections]);

  const handleAdd = async (data: ConnectionFormData) => {
    const newConn: NewRemoteConnection = {
      name: data.name,
      protocol: data.protocol,
      hostname: data.hostname,
      port: parseInt(data.port, 10),
      username: data.username || undefined,
      password: data.password,
      domain: data.domain || undefined,
      ssh_enabled: data.ssh_enabled,
      ssh_hostname: data.ssh_enabled ? data.ssh_hostname || undefined : undefined,
      ssh_port: data.ssh_enabled ? parseInt(data.ssh_port, 10) : undefined,
      ssh_username: data.ssh_enabled ? data.ssh_username || undefined : undefined,
      ssh_password: data.ssh_enabled && data.ssh_password ? data.ssh_password : undefined,
      ssh_key_data: data.ssh_enabled && data.ssh_key_data ? data.ssh_key_data : undefined,
      ssh_key_passphrase: data.ssh_enabled && data.ssh_key_passphrase ? data.ssh_key_passphrase : undefined,
      resolution: data.resolution,
      auto_resize: data.auto_resize,
      stretch_to_fill: data.stretch_to_fill,
    };
    await addRemoteConnectionCmd(newConn);
    setShowAddDialog(false);
    await loadConnections();
  };

  const handleEdit = async (data: ConnectionFormData) => {
    if (!editTarget) return;
    const update: RemoteConnectionUpdate = {
      name: data.name,
      protocol: data.protocol,
      hostname: data.hostname,
      port: parseInt(data.port, 10),
      username: data.username || null,
      domain: data.domain || null,
      ssh_enabled: data.ssh_enabled,
      ssh_hostname: data.ssh_enabled ? data.ssh_hostname || undefined : undefined,
      ssh_port: data.ssh_enabled ? parseInt(data.ssh_port, 10) : undefined,
      ssh_username: data.ssh_enabled ? data.ssh_username || undefined : undefined,
      resolution: data.resolution,
      auto_resize: data.auto_resize,
      stretch_to_fill: data.stretch_to_fill,
    };
    await updateRemoteConnectionCmd(editTarget.id, update);
    setEditTarget(null);
    await loadConnections();
  };

  const openEditDialog = async (id: string) => {
    try {
      const conn = await getRemoteConnectionCmd(id);
      if (conn) setEditTarget(conn);
    } catch (err) {
      console.error('Failed to load connection for edit', err);
    }
  };

  const confirmDelete = async () => {
    if (!deleteTargetId) return;
    try {
      await deleteRemoteConnectionCmd(deleteTargetId);
    } catch (err) {
      console.error('Failed to delete connection', err);
    }
    setDeleteTargetId(null);
    await loadConnections();
  };

  const openConnectDialog = (id: string) => {
    setConnectTargetId(id);
    setConnectPassword('');
    setConnectError(null);
  };

  // Attempt to connect using the password saved on the connection entry, without
  // prompting. Only if that fails (no stored password, or auth error) do we fall
  // back to the password dialog.
  const handleConnectStored = async (id: string) => {
    setIsLoading(true);
    setPageError(null);
    try {
      const session = await startRdpSession(id);
      setActiveSession(session);
    } catch (err) {
      openConnectDialog(id);
      setConnectError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const handleConnect = async () => {
    if (!connectTargetId) return;
    setIsLoading(true);
    setConnectError(null);
    try {
      const session = await startRdpSession(connectTargetId, connectPassword);
      setActiveSession(session);
      setConnectTargetId(null);
      setConnectPassword('');
    } catch (err) {
      setConnectError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const handleDisconnect = async () => {
    if (!activeSession) return;
    try { await stopRdpSession(activeSession.id); } catch { /* ignore */ }
    setActiveSession(null);
  };

  const editInitial = editTarget
    ? {
        name: editTarget.name,
        protocol: editTarget.protocol,
        hostname: editTarget.hostname,
        port: String(editTarget.port),
        username: editTarget.username ?? '',
        password: '',
        domain: editTarget.domain ?? '',
        ssh_enabled: editTarget.ssh_enabled,
        ssh_hostname: editTarget.ssh_hostname ?? '',
        ssh_port: String(editTarget.ssh_port ?? 22),
        ssh_username: editTarget.ssh_username ?? '',
        ssh_password: '',
        ssh_key_data: '',
        ssh_key_passphrase: '',
        resolution: editTarget.resolution ?? '1920x1080',
        auto_resize: editTarget.auto_resize,
        stretch_to_fill: editTarget.stretch_to_fill,
      }
    : undefined;

  return (
    <div className="h-full flex flex-col p-6 gap-4">
      <div className="flex items-center gap-2 shrink-0">
        <Monitor className="w-5 h-5 text-primary" />
        <h1 className="text-2xl font-bold">Remote Desktop</h1>
      </div>

      {pageError && (
        <Alert variant="destructive">
          <AlertDescription>{pageError}</AlertDescription>
        </Alert>
      )}

      {activeSession ? (
        <SessionView session={activeSession} onDisconnect={handleDisconnect} />
      ) : (
        <ConnectionList
          connections={connections}
          onConnect={handleConnectStored}
          onEdit={openEditDialog}
          onDelete={setDeleteTargetId}
          onAdd={() => setShowAddDialog(true)}
        />
      )}

      {/* Add Dialog */}
      <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
        <DialogContent className="max-w-lg">
          <ConnectionForm
            title="Add Connection"
            onSave={handleAdd}
            onCancel={() => setShowAddDialog(false)}
          />
        </DialogContent>
      </Dialog>

      {/* Edit Dialog */}
      <Dialog open={!!editTarget} onOpenChange={(open) => { if (!open) setEditTarget(null); }}>
        <DialogContent className="max-w-lg">
          {editTarget && (
            <ConnectionForm
              title="Edit Connection"
              initial={editInitial}
              onSave={handleEdit}
              onCancel={() => setEditTarget(null)}
              isEdit
            />
          )}
        </DialogContent>
      </Dialog>

      {/* Delete Confirm Dialog */}
      <Dialog open={!!deleteTargetId} onOpenChange={(open) => { if (!open) setDeleteTargetId(null); }}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle>Delete Connection</DialogTitle>
          </DialogHeader>
          <p className="text-sm text-muted-foreground py-2">
            This will permanently remove the connection and its stored credentials.
          </p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteTargetId(null)}>Cancel</Button>
            <Button variant="destructive" onClick={confirmDelete}>Delete</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Connect Dialog */}
      <Dialog open={!!connectTargetId} onOpenChange={(open) => { if (!open) setConnectTargetId(null); }}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle>Connect</DialogTitle>
          </DialogHeader>
          <div className="py-3 space-y-3">
            {connectError && (
              <Alert variant="destructive">
                <AlertDescription>{connectError}</AlertDescription>
              </Alert>
            )}
            <div className="space-y-1">
              <Label htmlFor="conn-password">Password</Label>
              <Input
                id="conn-password"
                type="password"
                value={connectPassword}
                onChange={(e) => setConnectPassword(e.target.value)}
                onKeyDown={(e) => { if (e.key === 'Enter') void handleConnect(); }}
                autoFocus
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConnectTargetId(null)}>Cancel</Button>
            <Button onClick={handleConnect} disabled={isLoading}>
              {isLoading ? 'Connecting…' : 'Connect'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default RemoteDesktopPage;

// Re-export for compatibility with the RemoteDesktop path import in App.tsx
export type { ConnectionFormData };
