import React, { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/index';
import { Badge } from '@/components/ui/index';
import {
  listProxmoxClusters,
  listPbsDatastores,
  listPbsSnapshots,
  listPbsTasks,
} from '@/lib/proxmoxClient';
import type { PbsDatastore, PbsSnapshot, PbsTask } from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
import { useProxmoxClusters } from '@/hooks/useProxmoxClusters';
import { toast } from 'sonner';

const isPbsCluster = (c: ClusterInfo) => c.clusterType === 'pbs';

function formatBytes(bytes?: number): string {
  if (bytes === undefined || bytes === null) return '—';
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

export function ProxmoxPBSPage() {
  const [allClusters, setAllClusters] = useState<ClusterInfo[]>([]);
  const { clusters: pbsClusters, selectedClusterId, setSelectedClusterId } =
    useProxmoxClusters(isPbsCluster);
  const [node, setNode] = useState<string>('localhost');
  const [activeTab, setActiveTab] = useState<string>('datastores');

  const [datastores, setDatastores] = useState<PbsDatastore[]>([]);
  const [datastoresLoading, setDatastoresLoading] = useState(false);

  const [selectedStore, setSelectedStore] = useState<string>('');
  const [namespace, setNamespace] = useState<string>('');
  const [snapshots, setSnapshots] = useState<PbsSnapshot[]>([]);
  const [snapshotsLoading, setSnapshotsLoading] = useState(false);

  const [taskNode, setTaskNode] = useState<string>('localhost');
  const [tasks, setTasks] = useState<PbsTask[]>([]);
  const [tasksLoading, setTasksLoading] = useState(false);

  // Fetched separately (unfiltered) only to distinguish "no clusters at all"
  // from "clusters exist but none are PBS" in the empty-state message below.
  useEffect(() => {
    listProxmoxClusters()
      .then(setAllClusters)
      .catch((err) => {
        console.error('Failed to load clusters:', err);
        toast.error('Failed to load clusters');
      });
  }, []);

  useEffect(() => {
    setTaskNode(node);
  }, [node]);

  const loadDatastores = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setDatastoresLoading(true);
    try {
      const ds = await listPbsDatastores(clusterId);
      setDatastores(ds);
      if (ds.length > 0 && !selectedStore) setSelectedStore(ds[0].store);
    } catch (err) {
      console.error('Failed to load datastores:', err);
      toast.error('Failed to load datastores');
    } finally {
      setDatastoresLoading(false);
    }
  }, [selectedStore]);

  useEffect(() => {
    if (selectedClusterId) loadDatastores(selectedClusterId);
  }, [selectedClusterId, loadDatastores]);

  const loadSnapshots = useCallback(async () => {
    if (!selectedClusterId || !selectedStore) {
      toast.error('Select a datastore first');
      return;
    }
    setSnapshotsLoading(true);
    try {
      const snaps = await listPbsSnapshots(selectedClusterId, selectedStore, namespace || undefined);
      setSnapshots(snaps);
    } catch (err) {
      console.error('Failed to load snapshots:', err);
      toast.error('Failed to load snapshots');
    } finally {
      setSnapshotsLoading(false);
    }
  }, [selectedClusterId, selectedStore, namespace]);

  const loadTasks = useCallback(async () => {
    if (!selectedClusterId) return;
    setTasksLoading(true);
    try {
      const t = await listPbsTasks(selectedClusterId, taskNode);
      setTasks(t);
    } catch (err) {
      console.error('Failed to load tasks:', err);
      toast.error('Failed to load tasks');
    } finally {
      setTasksLoading(false);
    }
  }, [selectedClusterId, taskNode]);

  const handleDatastoreRowClick = (store: string) => {
    setSelectedStore(store);
    setActiveTab('snapshots');
  };

  if (allClusters.length > 0 && pbsClusters.length === 0) {
    return (
      <div className="space-y-4">
        <div>
          <h1 className="text-2xl font-bold">PBS Management</h1>
          <p className="text-muted-foreground">Manage Proxmox Backup Server datastores and snapshots</p>
        </div>
        <div className="text-center py-12 text-muted-foreground">
          <p>No PBS clusters configured. Add a PBS cluster in Settings → Proxmox.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">PBS Management</h1>
          <p className="text-muted-foreground">Manage Proxmox Backup Server datastores and snapshots</p>
        </div>
        <div className="flex items-center space-x-2">
          {pbsClusters.length > 1 && (
            <select
              className="rounded-md border px-3 py-1.5 text-sm bg-background"
              value={selectedClusterId}
              onChange={(e) => setSelectedClusterId(e.target.value)}
            >
              {pbsClusters.map((c) => (
                <option key={c.id} value={c.id}>{c.name}</option>
              ))}
            </select>
          )}
          <div className="flex items-center space-x-1">
            <Label htmlFor="nodeInput" className="text-sm whitespace-nowrap">Node:</Label>
            <Input
              id="nodeInput"
              value={node}
              onChange={(e) => setNode(e.target.value)}
              className="w-32 h-8 text-sm"
              placeholder="localhost"
            />
          </div>
        </div>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="datastores">Datastores</TabsTrigger>
          <TabsTrigger value="snapshots">Snapshots</TabsTrigger>
          <TabsTrigger value="tasks">Tasks</TabsTrigger>
        </TabsList>

        <TabsContent value="datastores" className="mt-4">
          {datastoresLoading ? (
            <div className="text-center py-8 text-muted-foreground">Loading...</div>
          ) : datastores.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">No datastores found.</div>
          ) : (
            <table className="w-full text-sm">
              <thead className="border-b">
                <tr>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Datastore Name</th>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Path</th>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Used</th>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Total</th>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Available</th>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Capacity</th>
                </tr>
              </thead>
              <tbody>
                {datastores.map((ds) => {
                  const pct =
                    ds.total && ds.total > 0 && ds.used !== undefined
                      ? Math.min(100, Math.round((ds.used / ds.total) * 100))
                      : null;
                  return (
                    <tr
                      key={ds.store}
                      className="border-b last:border-0 hover:bg-muted/50 cursor-pointer"
                      onClick={() => handleDatastoreRowClick(ds.store)}
                    >
                      <td className="py-2 px-3 font-medium">{ds.store}</td>
                      <td className="py-2 px-3 text-muted-foreground">{ds.path ?? '—'}</td>
                      <td className="py-2 px-3">{formatBytes(ds.used)}</td>
                      <td className="py-2 px-3">{formatBytes(ds.total)}</td>
                      <td className="py-2 px-3">{formatBytes(ds.avail)}</td>
                      <td className="py-2 px-3 w-36">
                        {pct !== null ? (
                          <div className="w-full bg-gray-200 rounded h-2">
                            <div className="bg-blue-500 h-2 rounded" style={{ width: `${pct}%` }} />
                          </div>
                        ) : (
                          '—'
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          )}
        </TabsContent>

        <TabsContent value="snapshots" className="mt-4">
          <div className="flex items-end space-x-3 mb-4">
            <div className="space-y-1">
              <Label htmlFor="storeSelect" className="text-sm">Datastore</Label>
              <select
                id="storeSelect"
                className="rounded-md border px-3 py-1.5 text-sm bg-background"
                value={selectedStore}
                onChange={(e) => setSelectedStore(e.target.value)}
              >
                {datastores.length === 0 && <option value="">— no datastores —</option>}
                {datastores.map((ds) => (
                  <option key={ds.store} value={ds.store}>{ds.store}</option>
                ))}
              </select>
            </div>
            <div className="space-y-1">
              <Label htmlFor="namespaceInput" className="text-sm">Namespace (optional)</Label>
              <Input
                id="namespaceInput"
                value={namespace}
                onChange={(e) => setNamespace(e.target.value)}
                className="h-8 text-sm w-40"
                placeholder="e.g. team/dev"
              />
            </div>
            <Button size="sm" onClick={loadSnapshots} disabled={snapshotsLoading || !selectedStore}>
              Load
            </Button>
          </div>

          {snapshotsLoading ? (
            <div className="text-center py-8 text-muted-foreground">Loading...</div>
          ) : snapshots.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">No snapshots found.</div>
          ) : (
            <table className="w-full text-sm">
              <thead className="border-b">
                <tr>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Backup ID</th>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Type</th>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Timestamp</th>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Size</th>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Verify State</th>
                </tr>
              </thead>
              <tbody>
                {snapshots.map((snap) => (
                  <tr key={`${snap.backupId}-${snap.backupTime}`} className="border-b last:border-0 hover:bg-muted/50">
                    <td className="py-2 px-3 font-medium">{snap.backupId}</td>
                    <td className="py-2 px-3">{snap.backupType}</td>
                    <td className="py-2 px-3">{new Date(snap.backupTime * 1000).toLocaleString()}</td>
                    <td className="py-2 px-3">{formatBytes(snap.size)}</td>
                    <td className="py-2 px-3">{snap.verifyState ?? '—'}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </TabsContent>

        <TabsContent value="tasks" className="mt-4">
          <div className="flex items-end space-x-3 mb-4">
            <div className="space-y-1">
              <Label htmlFor="taskNodeInput" className="text-sm">Node</Label>
              <Input
                id="taskNodeInput"
                value={taskNode}
                onChange={(e) => setTaskNode(e.target.value)}
                className="h-8 text-sm w-40"
                placeholder="localhost"
              />
            </div>
            <Button size="sm" onClick={loadTasks} disabled={tasksLoading}>
              Load
            </Button>
          </div>

          {tasksLoading ? (
            <div className="text-center py-8 text-muted-foreground">Loading...</div>
          ) : tasks.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">No tasks found.</div>
          ) : (
            <table className="w-full text-sm">
              <thead className="border-b">
                <tr>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">UPID</th>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Type</th>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Start Time</th>
                  <th className="py-2 px-3 text-left font-medium text-muted-foreground">Status</th>
                </tr>
              </thead>
              <tbody>
                {tasks.map((task) => {
                  const status = task.status;
                  let statusEl: React.ReactNode;
                  if (!status) {
                    statusEl = <Badge variant="outline" className="text-yellow-600 border-yellow-400">Running</Badge>;
                  } else if (status === 'OK') {
                    statusEl = <Badge className="text-green-600 bg-green-100 border-green-300">OK</Badge>;
                  } else {
                    statusEl = <Badge variant="outline" className="text-red-600 border-red-400">{status}</Badge>;
                  }
                  return (
                    <tr key={task.upid} className="border-b last:border-0 hover:bg-muted/50">
                      <td className="py-2 px-3 font-mono text-xs max-w-xs truncate" title={task.upid}>{task.upid}</td>
                      <td className="py-2 px-3">{task.taskType}</td>
                      <td className="py-2 px-3">{new Date(task.starttime * 1000).toLocaleString()}</td>
                      <td className="py-2 px-3">{statusEl}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}
