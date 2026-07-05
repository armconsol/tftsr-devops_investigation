import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Badge } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Switch } from '@/components/ui/index';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/index';
import { RefreshCw, Plus, Play, Square, RotateCcw, Trash2 } from 'lucide-react';
import { confirm } from '@tauri-apps/plugin-dialog';
import { PoolList, OSDList, CephHealthWidget, NodeSelect } from '@/components/Proxmox';
import { useProxmoxNodes } from '@/hooks/useProxmoxNodes';
import { usePolling } from '@/hooks/usePolling';
import {
  listCephPools,
  listCephOsd,
  getCephHealth,
  listCephMonitors,
  listCephManagers,
  listCephfs,
  getCephFlags,
  setCephFlag,
  createCephMonitor,
  deleteCephMonitor,
  createCephManager,
  deleteCephManager,
  cephServiceAction,
} from '@/lib/proxmoxClient';
import type { CephMonitor, CephMgr, CephFs, CephHealth, CephPool, CephOsd, CephFlag } from '@/lib/proxmoxClient';
import { useProxmoxClusters } from '@/hooks/useProxmoxClusters';
import { toast } from 'sonner';

const POLL_INTERVAL_MS = 30_000;

export function ProxmoxCephPage() {
  const {
    clusters,
    selectedClusterId: clusterId,
    setSelectedClusterId: setClusterId,
    loading: clustersLoading,
    error: clustersError,
  } = useProxmoxClusters();
  const [health, setHealth] = useState<CephHealth | null>(null);
  const [pools, setPools] = useState<CephPool[]>([]);
  const [osds, setOsds] = useState<CephOsd[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isCephEnabled, setIsCephEnabled] = useState<boolean | null>(null);

  const [activeTab, setActiveTab] = useState('monitors');
  const [monitors, setMonitors] = useState<CephMonitor[]>([]);
  const [managers, setManagers] = useState<CephMgr[]>([]);
  const [cephFsList, setCephFsList] = useState<CephFs[]>([]);
  const [cephFlags, setCephFlags] = useState<CephFlag[] | null>(null);
  const [actionPending, setActionPending] = useState(false);
  const [createMonDialog, setCreateMonDialog] = useState(false);
  const [createMonId, setCreateMonId] = useState('');
  const [createMgrDialog, setCreateMgrDialog] = useState(false);
  const [createMgrId, setCreateMgrId] = useState('');
  const { nodeNames, selectedNode: tabNode, setSelectedNode, loading: nodesLoading } =
    useProxmoxNodes(clusterId);

  const loadData = useCallback(async (cId: string, node: string) => {
    if (!cId || !node) return;
    setLoading(true);
    setError(null);

    let cephAvailable = false;
    try {
      const h = await getCephHealth(cId, node);
      setHealth(h);
      cephAvailable = true;
    } catch {
      setIsCephEnabled(false);
      setLoading(false);
      return;
    }

    if (cephAvailable) {
      setIsCephEnabled(true);
      const [poolsResult, osdsResult] = await Promise.allSettled([
        listCephPools(cId, node),
        listCephOsd(cId, node),
      ]);

      if (poolsResult.status === 'fulfilled') {
        setPools(poolsResult.value);
      } else {
        toast.error('Failed to load Ceph pools');
      }

      if (osdsResult.status === 'fulfilled') {
        setOsds(osdsResult.value);
      } else {
        toast.error('Failed to load Ceph OSDs');
      }
    }

    setLoading(false);
  }, []);

  // Mirror the shared cluster list/errors into this page's own error banner
  // and gate the "Ceph not configured" state on there being no clusters at all.
  useEffect(() => {
    if (clustersLoading) return;
    if (clustersError) {
      setError('Failed to load clusters');
      setIsCephEnabled(false);
    } else if (clusters.length === 0) {
      setIsCephEnabled(false);
    }
  }, [clustersLoading, clustersError, clusters.length]);

  useEffect(() => {
    if (!clusterId || !tabNode) return;
    loadData(clusterId, tabNode);
  }, [clusterId, tabNode, loadData]);

  const handleClusterChange = (id: string) => {
    setClusterId(id);
    setHealth(null);
    setPools([]);
    setOsds([]);
    setMonitors([]);
    setManagers([]);
    setCephFsList([]);
    setCephFlags(null);
    setIsCephEnabled(null);
  };

  const canLoadTabData = !!clusterId && !!tabNode && isCephEnabled === true;

  const loadMonitors = useCallback(async () => {
    if (!clusterId || !tabNode) return;
    try {
      const data = await listCephMonitors(clusterId, tabNode);
      setMonitors(data);
    } catch {
      toast.error('Failed to load Ceph monitors');
    }
  }, [clusterId, tabNode]);

  const loadManagers = useCallback(async () => {
    if (!clusterId || !tabNode) return;
    try {
      const data = await listCephManagers(clusterId, tabNode);
      setManagers(data);
    } catch {
      toast.error('Failed to load Ceph managers');
    }
  }, [clusterId, tabNode]);

  const loadCephFs = useCallback(async () => {
    if (!clusterId || !tabNode) return;
    try {
      const data = await listCephfs(clusterId, tabNode);
      setCephFsList(data);
    } catch {
      toast.error('Failed to load CephFS');
    }
  }, [clusterId, tabNode]);

  const loadFlags = useCallback(async () => {
    if (!clusterId || !tabNode) return;
    try {
      const data = await getCephFlags(clusterId, tabNode);
      setCephFlags(data);
    } catch {
      toast.error('Failed to load Ceph flags');
    }
  }, [clusterId, tabNode]);

  usePolling(loadMonitors, POLL_INTERVAL_MS, canLoadTabData && activeTab === 'monitors');
  usePolling(loadManagers, POLL_INTERVAL_MS, canLoadTabData && activeTab === 'managers');
  usePolling(loadCephFs, POLL_INTERVAL_MS, canLoadTabData && activeTab === 'cephfs');
  usePolling(loadFlags, POLL_INTERVAL_MS, canLoadTabData && activeTab === 'flags');

  const handleRefresh = () => {
    if (clusterId && tabNode) loadData(clusterId, tabNode);
  };

  const runServiceAction = useCallback(
    async (kind: 'mon' | 'mgr', name: string, action: 'start' | 'stop' | 'restart') => {
      if (!clusterId || !tabNode) return;
      setActionPending(true);
      try {
        const upid = await cephServiceAction(clusterId, tabNode, `${kind}.${name}`, action);
        toast.success(`${action} ${kind} ${name}: ${upid}`);
        if (kind === 'mon') await loadMonitors();
        else await loadManagers();
      } catch (e) {
        toast.error(`Failed to ${action} ${kind} ${name}: ${e}`);
      } finally {
        setActionPending(false);
      }
    },
    [clusterId, tabNode, loadMonitors, loadManagers]
  );

  const handleDestroyMonitor = useCallback(
    async (mon: CephMonitor) => {
      if (!clusterId || !tabNode) return;
      const confirmed = await confirm(
        `Are you sure you want to destroy monitor "${mon.name}"? This cannot be undone.`,
        { title: 'Destroy Monitor', kind: 'warning' }
      );
      if (!confirmed) return;
      setActionPending(true);
      try {
        const upid = await deleteCephMonitor(clusterId, tabNode, mon.name);
        toast.success(`Monitor ${mon.name} destroyed: ${upid}`);
        await loadMonitors();
      } catch (e) {
        toast.error(`Failed to destroy monitor ${mon.name}: ${e}`);
      } finally {
        setActionPending(false);
      }
    },
    [clusterId, tabNode, loadMonitors]
  );

  const handleDestroyManager = useCallback(
    async (mgr: CephMgr) => {
      if (!clusterId || !tabNode) return;
      const confirmed = await confirm(
        `Are you sure you want to destroy manager "${mgr.name}"? This cannot be undone.`,
        { title: 'Destroy Manager', kind: 'warning' }
      );
      if (!confirmed) return;
      setActionPending(true);
      try {
        const upid = await deleteCephManager(clusterId, tabNode, mgr.name);
        toast.success(`Manager ${mgr.name} destroyed: ${upid}`);
        await loadManagers();
      } catch (e) {
        toast.error(`Failed to destroy manager ${mgr.name}: ${e}`);
      } finally {
        setActionPending(false);
      }
    },
    [clusterId, tabNode, loadManagers]
  );

  const handleCreateMonitor = useCallback(async () => {
    if (!clusterId || !tabNode || !createMonId.trim()) return;
    setActionPending(true);
    try {
      const upid = await createCephMonitor(clusterId, tabNode, createMonId.trim());
      toast.success(`Monitor ${createMonId} created: ${upid}`);
      setCreateMonDialog(false);
      setCreateMonId('');
      await loadMonitors();
    } catch (e) {
      toast.error(`Failed to create monitor: ${e}`);
    } finally {
      setActionPending(false);
    }
  }, [clusterId, tabNode, createMonId, loadMonitors]);

  const handleCreateManager = useCallback(async () => {
    if (!clusterId || !tabNode || !createMgrId.trim()) return;
    setActionPending(true);
    try {
      const upid = await createCephManager(clusterId, tabNode, createMgrId.trim());
      toast.success(`Manager ${createMgrId} created: ${upid}`);
      setCreateMgrDialog(false);
      setCreateMgrId('');
      await loadManagers();
    } catch (e) {
      toast.error(`Failed to create manager: ${e}`);
    } finally {
      setActionPending(false);
    }
  }, [clusterId, tabNode, createMgrId, loadManagers]);

  const handleToggleFlag = useCallback(
    async (flag: CephFlag, value: boolean) => {
      if (!clusterId) return;
      const previous = cephFlags;
      setCephFlags((prev) =>
        prev
          ? prev.map((f) => (f.name === flag.name ? { ...f, value: value ? 1 : 0 } : f))
          : prev
      );
      try {
        await setCephFlag(clusterId, flag.name, value);
        toast.success(`Flag ${flag.name} ${value ? 'set' : 'unset'}`);
      } catch (e) {
        setCephFlags(previous);
        toast.error(`Failed to update flag ${flag.name}: ${e}`);
      }
    },
    [clusterId, cephFlags]
  );

  const clusterSelector = clusters.length > 1 && (
    <Select value={clusterId} onValueChange={handleClusterChange}>
      <SelectTrigger className="h-8 w-48 text-sm">
        <SelectValue placeholder="Select datacenter" />
      </SelectTrigger>
      <SelectContent>
        {clusters.map((c) => (
          <SelectItem key={c.id} value={c.id}>{c.name}</SelectItem>
        ))}
      </SelectContent>
    </Select>
  );

  if (isCephEnabled === false) {
    return (
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Ceph Storage</h1>
            <p className="text-muted-foreground">Manage Ceph clusters and storage</p>
          </div>
          <div className="flex space-x-2">{clusterSelector}</div>
        </div>
        <Card>
          <CardContent className="py-12 text-center text-muted-foreground">
            {error ? (
              <p>{error}</p>
            ) : (
              <>
                <p className="text-base font-medium">Ceph is not configured on this datacenter</p>
                <p className="text-sm mt-1">
                  Ceph storage requires a dedicated Ceph cluster deployment on the Proxmox nodes.
                  {clusters.length > 1 && ' Select a different datacenter above if Ceph runs elsewhere.'}
                </p>
              </>
            )}
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Ceph Storage</h1>
          <p className="text-muted-foreground">Manage Ceph clusters and storage</p>
        </div>
        <div className="flex items-center space-x-2">
          {clusterSelector}
          <Button variant="outline" size="sm" onClick={handleRefresh} disabled={loading}>
            <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Ceph Health</CardTitle>
          </CardHeader>
          <CardContent>
            {health ? (
              <CephHealthWidget health={health} />
            ) : (
              <p className="text-sm text-muted-foreground">Loading health data...</p>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Pools</CardTitle>
          </CardHeader>
          <CardContent>
            <PoolList
              pools={pools}
              onRefresh={handleRefresh}
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>OSDs</CardTitle>
          </CardHeader>
          <CardContent>
            <OSDList
              osds={osds}
              onRefresh={handleRefresh}
            />
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Ceph Details</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-2">
            <Label className="text-sm">Node:</Label>
            <NodeSelect
              nodeNames={nodeNames}
              value={tabNode}
              onChange={setSelectedNode}
              loading={nodesLoading}
              disabled={!clusterId}
            />
          </div>

          <Tabs value={activeTab} onValueChange={setActiveTab}>
            <TabsList>
              <TabsTrigger value="monitors">Monitors</TabsTrigger>
              <TabsTrigger value="managers">Managers</TabsTrigger>
              <TabsTrigger value="cephfs">CephFS</TabsTrigger>
              <TabsTrigger value="flags">Flags</TabsTrigger>
            </TabsList>

            <TabsContent value="monitors">
              <div className="flex justify-end mb-3">
                <Button variant="outline" size="sm" onClick={() => setCreateMonDialog(true)} disabled={!tabNode}>
                  <Plus className="mr-2 h-4 w-4" />
                  Create Monitor
                </Button>
              </div>
              {monitors.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">No data found.</div>
              ) : (
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b">
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Name</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Address</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Version</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Quorum</th>
                      <th className="py-2 px-3 text-right font-medium text-muted-foreground">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {monitors.map((mon) => (
                      <tr key={mon.name} className="border-b last:border-0 hover:bg-muted/50">
                        <td className="py-2 px-3">{mon.name}</td>
                        <td className="py-2 px-3">{mon.address}</td>
                        <td className="py-2 px-3">{mon.version ?? '—'}</td>
                        <td className="py-2 px-3">
                          {mon.quorum === true ? (
                            <Badge className="bg-green-100 text-green-800 text-xs px-2 py-0.5 rounded">
                              In Quorum
                            </Badge>
                          ) : (
                            <Badge className="bg-red-100 text-red-800 text-xs px-2 py-0.5 rounded">
                              Not in Quorum
                            </Badge>
                          )}
                        </td>
                        <td className="py-2 px-3">
                          <div className="flex items-center justify-end gap-1">
                            <button
                              className="rounded-md p-1 hover:bg-accent"
                              title="Start"
                              aria-label={`Start Monitor ${mon.name}`}
                              disabled={actionPending}
                              onClick={() => runServiceAction('mon', mon.name, 'start')}
                            >
                              <Play className="h-4 w-4" />
                            </button>
                            <button
                              className="rounded-md p-1 hover:bg-accent"
                              title="Stop"
                              aria-label={`Stop Monitor ${mon.name}`}
                              disabled={actionPending}
                              onClick={() => runServiceAction('mon', mon.name, 'stop')}
                            >
                              <Square className="h-4 w-4" />
                            </button>
                            <button
                              className="rounded-md p-1 hover:bg-accent"
                              title="Restart"
                              aria-label={`Restart Monitor ${mon.name}`}
                              disabled={actionPending}
                              onClick={() => runServiceAction('mon', mon.name, 'restart')}
                            >
                              <RotateCcw className="h-4 w-4" />
                            </button>
                            <button
                              className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                              title="Destroy"
                              aria-label={`Destroy Monitor ${mon.name}`}
                              disabled={actionPending}
                              onClick={() => handleDestroyMonitor(mon)}
                            >
                              <Trash2 className="h-4 w-4" />
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </TabsContent>

            <TabsContent value="managers">
              <div className="flex justify-end mb-3">
                <Button variant="outline" size="sm" onClick={() => setCreateMgrDialog(true)} disabled={!tabNode}>
                  <Plus className="mr-2 h-4 w-4" />
                  Create Manager
                </Button>
              </div>
              {managers.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">No data found.</div>
              ) : (
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b">
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Name</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Address</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">State</th>
                      <th className="py-2 px-3 text-right font-medium text-muted-foreground">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {managers.map((mgr) => (
                      <tr key={mgr.name} className="border-b last:border-0 hover:bg-muted/50">
                        <td className="py-2 px-3">{mgr.name}</td>
                        <td className="py-2 px-3">{mgr.addr ?? '—'}</td>
                        <td className="py-2 px-3">
                          {mgr.state === 'active' ? (
                            <Badge className="bg-green-100 text-green-800 text-xs px-2 py-0.5 rounded">
                              active
                            </Badge>
                          ) : (
                            <Badge className="bg-gray-100 text-gray-700 text-xs px-2 py-0.5 rounded">
                              {mgr.state ?? 'standby'}
                            </Badge>
                          )}
                        </td>
                        <td className="py-2 px-3">
                          <div className="flex items-center justify-end gap-1">
                            <button
                              className="rounded-md p-1 hover:bg-accent"
                              title="Start"
                              aria-label={`Start Manager ${mgr.name}`}
                              disabled={actionPending}
                              onClick={() => runServiceAction('mgr', mgr.name, 'start')}
                            >
                              <Play className="h-4 w-4" />
                            </button>
                            <button
                              className="rounded-md p-1 hover:bg-accent"
                              title="Stop"
                              aria-label={`Stop Manager ${mgr.name}`}
                              disabled={actionPending}
                              onClick={() => runServiceAction('mgr', mgr.name, 'stop')}
                            >
                              <Square className="h-4 w-4" />
                            </button>
                            <button
                              className="rounded-md p-1 hover:bg-accent"
                              title="Restart"
                              aria-label={`Restart Manager ${mgr.name}`}
                              disabled={actionPending}
                              onClick={() => runServiceAction('mgr', mgr.name, 'restart')}
                            >
                              <RotateCcw className="h-4 w-4" />
                            </button>
                            <button
                              className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                              title="Destroy"
                              aria-label={`Destroy Manager ${mgr.name}`}
                              disabled={actionPending}
                              onClick={() => handleDestroyManager(mgr)}
                            >
                              <Trash2 className="h-4 w-4" />
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </TabsContent>

            <TabsContent value="cephfs">
              {cephFsList.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">No data found.</div>
              ) : (
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b">
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Name</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Metadata Pool</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Data Pool</th>
                    </tr>
                  </thead>
                  <tbody>
                    {cephFsList.map((fs) => (
                      <tr key={fs.name} className="border-b last:border-0 hover:bg-muted/50">
                        <td className="py-2 px-3">{fs.name}</td>
                        <td className="py-2 px-3">{fs.metadataPool ?? '—'}</td>
                        <td className="py-2 px-3">{fs.dataPool ?? '—'}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </TabsContent>

            <TabsContent value="flags">
              {cephFlags === null ? (
                <div className="text-center py-8 text-muted-foreground">No data found.</div>
              ) : cephFlags.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">No flags set.</div>
              ) : (
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b">
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Flag</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Enabled</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Description</th>
                    </tr>
                  </thead>
                  <tbody>
                    {cephFlags.map((flag) => (
                      <tr key={flag.name} className="border-b last:border-0 hover:bg-muted/50">
                        <td className="py-2 px-3 font-mono">{flag.name}</td>
                        <td className="py-2 px-3">
                          <Switch
                            aria-label={flag.name}
                            checked={!!flag.value}
                            onCheckedChange={(checked) => handleToggleFlag(flag, checked)}
                          />
                        </td>
                        <td className="py-2 px-3 text-muted-foreground">{flag.description ?? '—'}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>

      <Dialog open={createMonDialog} onOpenChange={setCreateMonDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create Ceph Monitor</DialogTitle>
          </DialogHeader>
          <div className="space-y-3 py-2">
            <div className="space-y-1">
              <Label htmlFor="mon-node">Node</Label>
              <Input id="mon-node" value={tabNode} disabled />
            </div>
            <div className="space-y-1">
              <Label htmlFor="mon-id">Monitor ID</Label>
              <Input
                id="mon-id"
                value={createMonId}
                onChange={(e) => setCreateMonId(e.target.value)}
                placeholder={tabNode}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateMonDialog(false)} disabled={actionPending}>
              Cancel
            </Button>
            <Button onClick={handleCreateMonitor} disabled={actionPending || !createMonId.trim()}>
              Create
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={createMgrDialog} onOpenChange={setCreateMgrDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create Ceph Manager</DialogTitle>
          </DialogHeader>
          <div className="space-y-3 py-2">
            <div className="space-y-1">
              <Label htmlFor="mgr-node">Node</Label>
              <Input id="mgr-node" value={tabNode} disabled />
            </div>
            <div className="space-y-1">
              <Label htmlFor="mgr-id">Manager ID</Label>
              <Input
                id="mgr-id"
                value={createMgrId}
                onChange={(e) => setCreateMgrId(e.target.value)}
                placeholder={tabNode}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateMgrDialog(false)} disabled={actionPending}>
              Cancel
            </Button>
            <Button onClick={handleCreateManager} disabled={actionPending || !createMgrId.trim()}>
              Create
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
