import React, { useState, useEffect, useCallback } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/index';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { RefreshCw, Power, RotateCcw } from 'lucide-react';
import {
  listProxmoxClusters,
  getNodeStatus,
  listAptUpdates,
  listAptRepositories,
  getSyslog,
  listClusterTasks,
  getNodeJournal,
  getNodeReport,
  rebootNode,
  shutdownNode,
} from '@/lib/proxmoxClient';
import type {
  NodeStatus,
  AptPackage,
  AptRepository,
  SyslogEntry,
  ClusterTask,
} from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
import { toast } from 'sonner';
import { useProxmoxNodes } from '@/hooks/useProxmoxNodes';

export function ProxmoxAdminPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [clusterId, setClusterId] = useState('');
  const [nodeStatus, setNodeStatus] = useState<NodeStatus | null>(null);
  const [aptUpdates, setAptUpdates] = useState<AptPackage[]>([]);
  const [aptRepos, setAptRepos] = useState<AptRepository[]>([]);
  const [syslog, setSyslog] = useState<SyslogEntry[]>([]);
  const [tasks, setTasks] = useState<ClusterTask[]>([]);
  const [journal, setJournal] = useState<string[]>([]);
  const [report, setReport] = useState<string>('');
  const [activeTab, setActiveTab] = useState('status');
  const [tabError, setTabError] = useState<string | null>(null);
  const [confirmAction, setConfirmAction] = useState<'reboot' | 'shutdown' | null>(null);

  const { nodeNames, selectedNode, setSelectedNode } = useProxmoxNodes(clusterId);

  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        setClusters(cls);
        if (cls.length > 0) setClusterId(cls[0].id);
      })
      .catch((err: unknown) => console.error('Failed to load clusters:', err));
  }, []);

  const loadTabData = useCallback(
    async (tab: string, cId: string, nId: string) => {
      if (!cId || !nId) return;
      setTabError(null);
      try {
        switch (tab) {
          case 'status':
            setNodeStatus(await getNodeStatus(cId, nId));
            break;
          case 'updates':
            setAptUpdates(await listAptUpdates(cId, nId));
            break;
          case 'repositories':
            setAptRepos(await listAptRepositories(cId, nId));
            break;
          case 'syslog':
            setSyslog(await getSyslog(cId, nId));
            break;
          case 'tasks':
            setTasks(await listClusterTasks(cId));
            break;
          case 'journal':
            setJournal(await getNodeJournal(cId, nId, 200));
            break;
          case 'report':
            setReport(await getNodeReport(cId, nId));
            break;
        }
      } catch (e) {
        setTabError(String(e));
      }
    },
    []
  );

  useEffect(() => {
    if (selectedNode) void loadTabData(activeTab, clusterId, selectedNode);
  }, [activeTab, clusterId, selectedNode, loadTabData]);

  const handleConfirmAction = async () => {
    if (!confirmAction) return;
    try {
      if (confirmAction === 'reboot') {
        await rebootNode(clusterId, selectedNode);
        toast.success('Node reboot initiated');
      } else {
        await shutdownNode(clusterId, selectedNode);
        toast.success('Node shutdown initiated');
      }
    } catch (e) {
      toast.error(String(e));
    } finally {
      setConfirmAction(null);
    }
  };

  const formatBytes = (bytes: number | undefined | null) => {
    if (bytes == null || Number.isNaN(bytes)) return '—';
    return bytes >= 1073741824
      ? `${(bytes / 1073741824).toFixed(1)} GB`
      : `${Math.round(bytes / 1048576)} MB`;
  };

  const formatUptime = (seconds: number | undefined | null) => {
    if (seconds == null || Number.isNaN(seconds)) return '—';
    const d = Math.floor(seconds / 86400);
    const h = Math.floor((seconds % 86400) / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    return d > 0 ? `${d}d ${h}h ${m}m` : `${h}h ${m}m`;
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Administration</h1>
          <p className="text-muted-foreground">Node management, updates, and system monitoring</p>
        </div>
      </div>

      <div className="flex items-center gap-3 flex-wrap">
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Cluster:</span>
          <select
            className="text-sm border rounded px-2 py-1 bg-background"
            value={clusterId}
            onChange={(e) => setClusterId(e.target.value)}
          >
            {clusters.length === 0 && <option value="">No clusters</option>}
            {clusters.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Node:</span>
          <select
            className="text-sm border rounded px-2 py-1 bg-background"
            value={selectedNode}
            onChange={(e) => setSelectedNode(e.target.value)}
          >
            {nodeNames.length === 0 && <option value="">No nodes</option>}
            {nodeNames.map((n) => (
              <option key={n} value={n}>{n}</option>
            ))}
          </select>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => void loadTabData(activeTab, clusterId, selectedNode)}
        >
          <RefreshCw className="mr-2 h-4 w-4" />
          Refresh
        </Button>
      </div>

      {tabError && <div className="text-destructive text-sm">{tabError}</div>}

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="status">Node Status</TabsTrigger>
          <TabsTrigger value="updates">Updates</TabsTrigger>
          <TabsTrigger value="repositories">Repositories</TabsTrigger>
          <TabsTrigger value="syslog">System Log</TabsTrigger>
          <TabsTrigger value="tasks">Tasks</TabsTrigger>
          <TabsTrigger value="journal">Journal</TabsTrigger>
          <TabsTrigger value="report">Report</TabsTrigger>
        </TabsList>

        <TabsContent value="status">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>Node Status</CardTitle>
            </CardHeader>
            <CardContent>
              {nodeStatus ? (
                <>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="text-muted-foreground">CPU:</span>{' '}
                      {((nodeStatus.cpu ?? 0) * 100).toFixed(1)}%
                    </div>
                    <div>
                      <span className="text-muted-foreground">Memory:</span>{' '}
                      {formatBytes(nodeStatus.memory?.used)} / {formatBytes(nodeStatus.memory?.total)}
                    </div>
                    <div>
                      <span className="text-muted-foreground">Swap:</span>{' '}
                      {formatBytes(nodeStatus.swap?.used)} / {formatBytes(nodeStatus.swap?.total)}
                    </div>
                    <div>
                      <span className="text-muted-foreground">Disk:</span>{' '}
                      {formatBytes(nodeStatus.disk?.used)} / {formatBytes(nodeStatus.disk?.total)}
                    </div>
                    <div>
                      <span className="text-muted-foreground">Uptime:</span>{' '}
                      {formatUptime(nodeStatus.uptime)}
                    </div>
                    <div>
                      <span className="text-muted-foreground">Version:</span>{' '}
                      {nodeStatus.version}
                    </div>
                    {(nodeStatus.loadAvg?.length ?? 0) > 0 && (
                      <div className="col-span-2">
                        <span className="text-muted-foreground">Load Avg:</span>{' '}
                        {(nodeStatus.loadAvg ?? []).map((v) => v.toFixed(2)).join(' / ')}
                      </div>
                    )}
                  </div>
                  <div className="flex gap-2 mt-4 pt-4 border-t border-destructive/20">
                    <Button
                      variant="destructive"
                      size="sm"
                      onClick={() => setConfirmAction('reboot')}
                    >
                      <RotateCcw className="mr-2 h-4 w-4" />
                      Reboot Node
                    </Button>
                    <Button
                      variant="destructive"
                      size="sm"
                      onClick={() => setConfirmAction('shutdown')}
                    >
                      <Power className="mr-2 h-4 w-4" />
                      Shutdown Node
                    </Button>
                  </div>
                </>
              ) : (
                <div className="text-muted-foreground text-sm">Loading node status...</div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="updates">
          <Card>
            <CardHeader>
              <CardTitle>Available Updates ({aptUpdates.length})</CardTitle>
            </CardHeader>
            <CardContent>
              {aptUpdates.length === 0 ? (
                <div className="text-muted-foreground text-sm">No updates available</div>
              ) : (
                <div className="space-y-1">
                  {aptUpdates.map((pkg, i) => (
                    <div
                      key={`${pkg.package}-${i}`}
                      className="flex items-center justify-between p-2 border rounded text-sm"
                    >
                      <span className="font-mono">{pkg.package}</span>
                      <span className="text-muted-foreground">
                        {pkg.version}
                        {pkg.newVersion ? ` → ${pkg.newVersion}` : ''}
                      </span>
                      {pkg.description && (
                        <span className="text-xs text-muted-foreground truncate max-w-xs ml-2">
                          {pkg.description}
                        </span>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="repositories">
          <Card>
            <CardHeader>
              <CardTitle>APT Repositories</CardTitle>
            </CardHeader>
            <CardContent>
              {aptRepos.length === 0 ? (
                <div className="text-muted-foreground text-sm">No repositories found</div>
              ) : (
                <div className="space-y-2">
                  {aptRepos.map((repo, i) => (
                    <div key={i} className="p-3 border rounded text-sm">
                      <div className="font-mono text-xs">
                        {repo.types.join(' ')} {repo.uris.join(' ')} {repo.suites.join(' ')}{' '}
                        {repo.components.join(' ')}
                      </div>
                      <div className="flex items-center gap-2 mt-1">
                        <span
                          className={
                            repo.enabled
                              ? 'text-xs text-green-600'
                              : 'text-xs text-muted-foreground'
                          }
                        >
                          {repo.enabled ? 'Enabled' : 'Disabled'}
                        </span>
                        {repo.comment && (
                          <span className="text-xs text-muted-foreground">{repo.comment}</span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="syslog">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>System Log</CardTitle>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void loadTabData('syslog', clusterId, selectedNode)}
              >
                <RefreshCw className="mr-2 h-4 w-4" />
                Refresh
              </Button>
            </CardHeader>
            <CardContent>
              <div className="font-mono text-xs space-y-0.5 max-h-96 overflow-y-auto">
                {syslog.length === 0 ? (
                  <div className="text-muted-foreground">No log entries</div>
                ) : (
                  syslog.map((entry) => (
                    <div key={entry.n} className="text-muted-foreground">
                      {entry.t} {entry.msg}
                    </div>
                  ))
                )}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="tasks">
          <Card>
            <CardHeader>
              <CardTitle>Recent Tasks</CardTitle>
            </CardHeader>
            <CardContent>
              {tasks.length === 0 ? (
                <div className="text-muted-foreground text-sm">No tasks found</div>
              ) : (
                <div className="space-y-1">
                  {tasks.map((t) => (
                    <div
                      key={t.upid}
                      className="flex items-center gap-2 p-2 border rounded text-sm"
                    >
                      <span className="font-mono text-xs text-muted-foreground truncate max-w-xs">
                        {t.upid}
                      </span>
                      <span>{t.type}</span>
                      <span className="text-muted-foreground">{t.node}</span>
                      <span
                        className={
                          t.exitstatus === 'OK' ? 'text-green-600' : 'text-destructive'
                        }
                      >
                        {t.exitstatus ?? 'running'}
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="journal">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>System Journal</CardTitle>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void loadTabData('journal', clusterId, selectedNode)}
              >
                <RefreshCw className="mr-2 h-4 w-4" />
                Refresh
              </Button>
            </CardHeader>
            <CardContent>
              <pre className="text-xs font-mono bg-muted p-3 rounded max-h-96 overflow-y-auto whitespace-pre-wrap">
                {journal.join('\n') || 'No journal entries'}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="report">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>Node Report</CardTitle>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void loadTabData('report', clusterId, selectedNode)}
              >
                <RefreshCw className="mr-2 h-4 w-4" />
                Refresh
              </Button>
            </CardHeader>
            <CardContent>
              <pre className="text-xs font-mono bg-muted p-3 rounded max-h-96 overflow-y-auto whitespace-pre-wrap">
                {report || 'No report available'}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      <Dialog open={!!confirmAction} onOpenChange={() => setConfirmAction(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              Confirm {confirmAction === 'reboot' ? 'Reboot' : 'Shutdown'}
            </DialogTitle>
          </DialogHeader>
          <p className="text-sm text-muted-foreground py-2">
            Are you sure you want to {confirmAction} node <strong>{selectedNode}</strong>? This will
            interrupt all running workloads on this node.
          </p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmAction(null)}>Cancel</Button>
            <Button variant="destructive" onClick={() => void handleConfirmAction()}>
              {confirmAction === 'reboot' ? 'Reboot' : 'Shutdown'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
