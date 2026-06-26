import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { RefreshCw, Plus, Settings } from 'lucide-react';
import { ContainerOverview } from '@/components/Proxmox';
import {
  listProxmoxClusters,
  listProxmoxContainers,
  listProxmoxNodes,
  startProxmoxContainer,
  stopProxmoxContainer,
  rebootProxmoxContainer,
  shutdownProxmoxContainer,
  getContainerConfig,
  createProxmoxContainer,
} from '@/lib/proxmoxClient';
import type { ContainerCreateParams } from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
import { toast } from 'sonner';

const defaultCreateParams: ContainerCreateParams = {
  vmid: 0,
  ostemplate: '',
  hostname: '',
  memory: undefined,
  cores: undefined,
  rootfs: '',
  net0: '',
  password: '',
  unprivileged: true,
  start: false,
};

export function ProxmoxContainersPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [selectedClusterId, setSelectedClusterId] = useState<string>('');
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [containers, setContainers] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [selectedContainer, setSelectedContainer] = useState<any | null>(null);

  // Config viewer state
  const [configCt, setConfigCt] = useState<{ node: string; vmid: number } | null>(null);
  const [ctConfig, setCtConfig] = useState<Record<string, unknown> | null>(null);
  const [configLoading, setConfigLoading] = useState(false);

  // Create container dialog state
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [createNode, setCreateNode] = useState('');
  const [availableNodes, setAvailableNodes] = useState<string[]>([]);
  const [createParams, setCreateParams] = useState<ContainerCreateParams>(defaultCreateParams);
  const [creating, setCreating] = useState(false);

  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        setClusters(cls);
        if (cls.length > 0) setSelectedClusterId(cls[0].id);
      })
      .catch((err) => {
        console.error('Failed to load clusters:', err);
        toast.error('Failed to load clusters');
      });
  }, []);

  // Load node list whenever cluster changes
  useEffect(() => {
    if (!selectedClusterId) return;
    listProxmoxNodes(selectedClusterId)
      .then((nodes: { node?: string }[]) => {
        const names = nodes.map((n) => n.node ?? '').filter(Boolean);
        setAvailableNodes(names);
        if (names.length > 0) setCreateNode(names[0]);
      })
      .catch(console.error);
  }, [selectedClusterId]);

  const loadContainers = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setIsLoading(true);
    try {
      const data = await listProxmoxContainers(clusterId);
      setContainers(data);
    } catch (err) {
      console.error('Failed to load containers:', err);
      toast.error('Failed to load containers');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (selectedClusterId) loadContainers(selectedClusterId);
  }, [selectedClusterId, loadContainers]);

  const handleViewConfig = async (node: string, vmid: number) => {
    setConfigCt({ node, vmid });
    setConfigLoading(true);
    setCtConfig(null);
    try {
      const config = await getContainerConfig(selectedClusterId, node, vmid);
      setCtConfig(config as Record<string, unknown>);
    } catch (e) {
      toast.error(String(e));
    } finally {
      setConfigLoading(false);
    }
  };

  const handleOpenCreateDialog = () => {
    setCreateParams({ ...defaultCreateParams });
    setShowCreateDialog(true);
  };

  const handleCreateContainer = async () => {
    if (!createParams.vmid || createParams.vmid < 100) {
      toast.error('VMID must be ≥ 100');
      return;
    }
    if (!createParams.ostemplate.trim()) {
      toast.error('OS Template is required');
      return;
    }
    setCreating(true);
    try {
      const params: ContainerCreateParams = {
        vmid: createParams.vmid,
        ostemplate: createParams.ostemplate.trim(),
        hostname: createParams.hostname?.trim() || undefined,
        memory: createParams.memory || undefined,
        cores: createParams.cores || undefined,
        rootfs: createParams.rootfs?.trim() || undefined,
        net0: createParams.net0?.trim() || undefined,
        password: createParams.password || undefined,
        unprivileged: createParams.unprivileged,
        start: createParams.start,
      };
      await createProxmoxContainer(selectedClusterId, createNode, params);
      toast.success(`Container CT${createParams.vmid} creation initiated`);
      setShowCreateDialog(false);
      await loadContainers(selectedClusterId);
    } catch (e) {
      toast.error(String(e));
    } finally {
      setCreating(false);
    }
  };

  if (clusters.length === 0 && !isLoading) {
    return (
      <div className="space-y-4">
        <div>
          <h1 className="text-2xl font-bold">Containers</h1>
          <p className="text-muted-foreground">Manage LXC containers</p>
        </div>
        <div className="text-center py-12 text-muted-foreground">
          <p>No Proxmox clusters configured.</p>
          <p className="text-sm mt-1">Add a remote connection first.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Containers</h1>
          <p className="text-muted-foreground">Manage LXC containers</p>
        </div>
        <div className="flex items-center space-x-2">
          {clusters.length > 1 && (
            <select
              className="rounded-md border px-3 py-1.5 text-sm bg-background"
              value={selectedClusterId}
              onChange={(e) => setSelectedClusterId(e.target.value)}
            >
              {clusters.map((c) => (
                <option key={c.id} value={c.id}>{c.name}</option>
              ))}
            </select>
          )}
          <Button variant="outline" size="sm" onClick={() => loadContainers(selectedClusterId)}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
          <Button size="sm" onClick={handleOpenCreateDialog} disabled={!selectedClusterId}>
            <Plus className="mr-2 h-4 w-4" />
            Create Container
          </Button>
        </div>
      </div>

      {selectedContainer ? (
        <ContainerOverview
          container={selectedContainer}
          onRefresh={() => loadContainers(selectedClusterId)}
          onPowerAction={async (action) => {
            const ct = selectedContainer;
            if (!ct || !selectedClusterId) return;
            const vmId: number = ct.vmid ?? ct.id;
            if (!vmId) {
              toast.error('Container ID not available');
              return;
            }
            const nodeId: string = ct.node ?? '';
            try {
              if (action === 'start') await startProxmoxContainer(selectedClusterId, nodeId, vmId);
              else if (action === 'stop') await stopProxmoxContainer(selectedClusterId, nodeId, vmId);
              else if (action === 'reboot') await rebootProxmoxContainer(selectedClusterId, nodeId, vmId);
              else if (action === 'shutdown') await shutdownProxmoxContainer(selectedClusterId, nodeId, vmId);
              toast.success(`Container ${action} initiated`);
              loadContainers(selectedClusterId);
            } catch (err) {
              toast.error(`Failed to ${action} container: ${err}`);
            }
          }}
          onConsole={() => { toast.info('Terminal access via VNC coming in a future release'); }}
        />
      ) : (
        <div className="grid grid-cols-1 gap-4">
          {containers.map((container) => (
            <Card
              key={container.vmid ?? container.id}
              className="cursor-pointer hover:shadow-md"
              onClick={() => setSelectedContainer(container)}
            >
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle>{container.name ?? `CT ${container.vmid}`}</CardTitle>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-8 w-8 p-0 shrink-0"
                  title="View Config"
                  onClick={(e) => {
                    e.stopPropagation();
                    void handleViewConfig(container.node ?? '', container.vmid ?? container.id);
                  }}
                >
                  <Settings className="h-4 w-4" />
                </Button>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-4 gap-4 text-sm">
                  <div>
                    <div className="text-muted-foreground">CT ID</div>
                    <div className="font-medium">{container.vmid}</div>
                  </div>
                  <div>
                    <div className="text-muted-foreground">Node</div>
                    <div className="font-medium">{container.node}</div>
                  </div>
                  <div>
                    <div className="text-muted-foreground">Status</div>
                    <div className="font-medium">{container.status}</div>
                  </div>
                  <div>
                    <div className="text-muted-foreground">Resources</div>
                    <div className="font-medium">
                      {container.maxcpu ?? container.cpu ?? '?'} CPU /{' '}
                      {container.maxmem
                        ? `${Math.round(container.maxmem / 1048576)} MB`
                        : container.memory
                          ? `${container.memory} MB`
                          : '?'}{' '}
                      RAM
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Container Config Dialog */}
      <Dialog open={!!configCt} onOpenChange={() => { setConfigCt(null); setCtConfig(null); }}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Container Config: {configCt?.vmid}</DialogTitle>
          </DialogHeader>
          {configLoading ? (
            <div className="text-sm text-muted-foreground">Loading...</div>
          ) : (
            <pre className="text-xs font-mono bg-muted p-3 rounded max-h-96 overflow-auto">
              {JSON.stringify(ctConfig, null, 2)}
            </pre>
          )}
        </DialogContent>
      </Dialog>

      {/* Create Container Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
        <DialogContent className="max-w-lg max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Create Container</DialogTitle>
          </DialogHeader>
          <div className="space-y-3 py-2">
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label>VMID *</Label>
                <Input
                  type="number"
                  min={100}
                  value={createParams.vmid || ''}
                  onChange={(e) => setCreateParams((p) => ({ ...p, vmid: parseInt(e.target.value) || 0 }))}
                  placeholder="e.g. 200"
                />
              </div>
              <div className="space-y-1">
                <Label>Node</Label>
                <select
                  className="w-full rounded-md border px-3 py-2 text-sm bg-background"
                  value={createNode}
                  onChange={(e) => setCreateNode(e.target.value)}
                >
                  {availableNodes.length === 0 ? (
                    <option value="">No nodes</option>
                  ) : (
                    availableNodes.map((n) => <option key={n} value={n}>{n}</option>)
                  )}
                </select>
              </div>
            </div>

            <div className="space-y-1">
              <Label>OS Template *</Label>
              <Input
                value={createParams.ostemplate}
                onChange={(e) => setCreateParams((p) => ({ ...p, ostemplate: e.target.value }))}
                placeholder="e.g. local:vztmpl/ubuntu-22.04-standard.tar.zst"
              />
            </div>

            <div className="space-y-1">
              <Label>Hostname</Label>
              <Input
                value={createParams.hostname ?? ''}
                onChange={(e) => setCreateParams((p) => ({ ...p, hostname: e.target.value }))}
                placeholder="e.g. my-container"
              />
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label>Memory (MB)</Label>
                <Input
                  type="number"
                  min={16}
                  value={createParams.memory ?? ''}
                  onChange={(e) => setCreateParams((p) => ({ ...p, memory: parseInt(e.target.value) || undefined }))}
                  placeholder="e.g. 512"
                />
              </div>
              <div className="space-y-1">
                <Label>Cores</Label>
                <Input
                  type="number"
                  min={1}
                  value={createParams.cores ?? ''}
                  onChange={(e) => setCreateParams((p) => ({ ...p, cores: parseInt(e.target.value) || undefined }))}
                  placeholder="e.g. 2"
                />
              </div>
            </div>

            <div className="space-y-1">
              <Label>Root FS</Label>
              <Input
                value={createParams.rootfs ?? ''}
                onChange={(e) => setCreateParams((p) => ({ ...p, rootfs: e.target.value }))}
                placeholder="e.g. local-lvm:8"
              />
            </div>

            <div className="space-y-1">
              <Label>Network</Label>
              <Input
                value={createParams.net0 ?? ''}
                onChange={(e) => setCreateParams((p) => ({ ...p, net0: e.target.value }))}
                placeholder="e.g. name=eth0,bridge=vmbr0,ip=dhcp"
              />
            </div>

            <div className="space-y-1">
              <Label>Password</Label>
              <Input
                type="password"
                value={createParams.password ?? ''}
                onChange={(e) => setCreateParams((p) => ({ ...p, password: e.target.value }))}
              />
            </div>

            <div className="flex items-center gap-6">
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="ct-unprivileged"
                  checked={createParams.unprivileged ?? true}
                  onChange={(e) => setCreateParams((p) => ({ ...p, unprivileged: e.target.checked }))}
                />
                <Label htmlFor="ct-unprivileged">Unprivileged</Label>
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="ct-start"
                  checked={createParams.start ?? false}
                  onChange={(e) => setCreateParams((p) => ({ ...p, start: e.target.checked }))}
                />
                <Label htmlFor="ct-start">Start after create</Label>
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreateDialog(false)} disabled={creating}>
              Cancel
            </Button>
            <Button onClick={() => void handleCreateContainer()} disabled={creating}>
              {creating ? 'Creating...' : 'Create'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
