import React, { useState, useEffect, useCallback } from 'react';
import { RefreshCw, RotateCcw, Power, AlertTriangle } from 'lucide-react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/index';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/index';
import {
  listProxmoxClusters,
  getNodeStatus,
  getNodeDns,
  updateNodeDns,
  getNodeTime,
  updateNodeTime,
  getNodeJournal,
  getNodeReport,
  rebootNode,
  shutdownNode,
} from '@/lib/proxmoxClient';
import type { NodeStatus, NodeDns, NodeTime } from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
import { toast } from 'sonner';

type ConfirmAction = 'reboot' | 'shutdown' | null;

export function ProxmoxNodeDetailPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [clusterId, setClusterId] = useState('');
  const [nodeInputValue, setNodeInputValue] = useState('localhost');
  const [nodeId, setNodeId] = useState('localhost');
  const [activeTab, setActiveTab] = useState('status');

  // Status
  const [nodeStatus, setNodeStatus] = useState<NodeStatus | null>(null);
  const [isLoadingStatus, setIsLoadingStatus] = useState(false);

  // DNS
  const [nodeDns, setNodeDns] = useState<NodeDns | null>(null);
  const [dnsForm, setDnsForm] = useState({ search: '', dns1: '', dns2: '', dns3: '' });
  const [isLoadingDns, setIsLoadingDns] = useState(false);
  const [isSavingDns, setIsSavingDns] = useState(false);

  // Time
  const [nodeTime, setNodeTime] = useState<NodeTime | null>(null);
  const [timezoneInput, setTimezoneInput] = useState('');
  const [isLoadingTime, setIsLoadingTime] = useState(false);
  const [isSavingTime, setIsSavingTime] = useState(false);

  // Journal
  const [journal, setJournal] = useState<string[]>([]);
  const [isLoadingJournal, setIsLoadingJournal] = useState(false);

  // Report
  const [report, setReport] = useState('');
  const [isLoadingReport, setIsLoadingReport] = useState(false);

  // Admin confirm dialog
  const [confirmAction, setConfirmAction] = useState<ConfirmAction>(null);
  const [isExecutingAction, setIsExecutingAction] = useState(false);

  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        setClusters(cls);
        if (cls.length > 0) setClusterId(cls[0].id);
      })
      .catch((err: unknown) => {
        console.error('Failed to load clusters:', err);
        toast.error('Failed to load clusters');
      });
  }, []);

  const applyNodeId = () => {
    setNodeId(nodeInputValue.trim() || 'localhost');
  };

  // ─── Tab data loaders ────────────────────────────────────────────────────────

  const loadStatus = useCallback(async (cId: string, nId: string) => {
    if (!cId) return;
    setIsLoadingStatus(true);
    try {
      setNodeStatus(await getNodeStatus(cId, nId));
    } catch (err: unknown) {
      console.error('Failed to load node status:', err);
      toast.error('Failed to load node status');
    } finally {
      setIsLoadingStatus(false);
    }
  }, []);

  const loadDns = useCallback(async (cId: string, nId: string) => {
    if (!cId) return;
    setIsLoadingDns(true);
    try {
      const dns = await getNodeDns(cId, nId);
      setNodeDns(dns);
      setDnsForm({
        search: dns.search ?? '',
        dns1: dns.dns1 ?? '',
        dns2: dns.dns2 ?? '',
        dns3: dns.dns3 ?? '',
      });
    } catch (err: unknown) {
      console.error('Failed to load DNS settings:', err);
      toast.error('Failed to load DNS settings');
    } finally {
      setIsLoadingDns(false);
    }
  }, []);

  const loadTime = useCallback(async (cId: string, nId: string) => {
    if (!cId) return;
    setIsLoadingTime(true);
    try {
      const t = await getNodeTime(cId, nId);
      setNodeTime(t);
      setTimezoneInput(t.timezone);
    } catch (err: unknown) {
      console.error('Failed to load time settings:', err);
      toast.error('Failed to load time settings');
    } finally {
      setIsLoadingTime(false);
    }
  }, []);

  const loadJournal = useCallback(async (cId: string, nId: string) => {
    if (!cId) return;
    setIsLoadingJournal(true);
    try {
      setJournal(await getNodeJournal(cId, nId, 100));
    } catch (err: unknown) {
      console.error('Failed to load journal:', err);
      toast.error('Failed to load journal');
    } finally {
      setIsLoadingJournal(false);
    }
  }, []);

  const loadReport = useCallback(async (cId: string, nId: string) => {
    if (!cId) return;
    setIsLoadingReport(true);
    try {
      setReport(await getNodeReport(cId, nId));
    } catch (err: unknown) {
      console.error('Failed to generate report:', err);
      toast.error('Failed to generate report');
    } finally {
      setIsLoadingReport(false);
    }
  }, []);

  useEffect(() => {
    if (!clusterId) return;
    switch (activeTab) {
      case 'status':
        void loadStatus(clusterId, nodeId);
        break;
      case 'dns':
        void loadDns(clusterId, nodeId);
        break;
      case 'time':
        void loadTime(clusterId, nodeId);
        break;
      // journal and report are load-on-demand only
    }
  }, [activeTab, clusterId, nodeId, loadStatus, loadDns, loadTime]);

  // ─── Handlers ────────────────────────────────────────────────────────────────

  const handleSaveDns = async () => {
    if (!clusterId) return;
    setIsSavingDns(true);
    try {
      await updateNodeDns(
        clusterId,
        nodeId,
        dnsForm.search,
        dnsForm.dns1 || undefined,
        dnsForm.dns2 || undefined,
        dnsForm.dns3 || undefined
      );
      toast.success('DNS settings updated');
      await loadDns(clusterId, nodeId);
    } catch (err: unknown) {
      toast.error(`Failed to update DNS: ${String(err)}`);
    } finally {
      setIsSavingDns(false);
    }
  };

  const handleSaveTime = async () => {
    if (!clusterId || !timezoneInput.trim()) return;
    setIsSavingTime(true);
    try {
      await updateNodeTime(clusterId, nodeId, timezoneInput.trim());
      toast.success('Timezone updated');
      await loadTime(clusterId, nodeId);
    } catch (err: unknown) {
      toast.error(`Failed to update timezone: ${String(err)}`);
    } finally {
      setIsSavingTime(false);
    }
  };

  const handleConfirmAction = async () => {
    if (!confirmAction || !clusterId) return;
    setIsExecutingAction(true);
    try {
      if (confirmAction === 'reboot') {
        await rebootNode(clusterId, nodeId);
        toast.success(`Reboot initiated for node ${nodeId}`);
      } else {
        await shutdownNode(clusterId, nodeId);
        toast.success(`Shutdown initiated for node ${nodeId}`);
      }
      setConfirmAction(null);
    } catch (err: unknown) {
      toast.error(`Failed to ${confirmAction} node: ${String(err)}`);
    } finally {
      setIsExecutingAction(false);
    }
  };

  // ─── Formatters ──────────────────────────────────────────────────────────────

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

  const formatEpoch = (epoch: number) => new Date(epoch * 1000).toLocaleString();

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Node Detail</h1>
          <p className="text-muted-foreground">
            Per-node DNS, time, journal, reports, and administration
          </p>
        </div>
      </div>

      {/* Cluster / Node selector */}
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
          <Input
            className="w-36 h-8 text-sm"
            value={nodeInputValue}
            onChange={(e) => setNodeInputValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') applyNodeId();
            }}
            placeholder="localhost"
          />
          <Button variant="outline" size="sm" onClick={applyNodeId}>
            Apply
          </Button>
        </div>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="status">Status</TabsTrigger>
          <TabsTrigger value="dns">DNS</TabsTrigger>
          <TabsTrigger value="time">Time / NTP</TabsTrigger>
          <TabsTrigger value="journal">Journal</TabsTrigger>
          <TabsTrigger value="report">Report</TabsTrigger>
          <TabsTrigger value="admin">Admin</TabsTrigger>
        </TabsList>

        {/* ── Status ──────────────────────────────────────────────────────────── */}
        <TabsContent value="status">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>Node Status</CardTitle>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void loadStatus(clusterId, nodeId)}
                disabled={isLoadingStatus}
              >
                <RefreshCw className={`mr-2 h-4 w-4 ${isLoadingStatus ? 'animate-spin' : ''}`} />
                Refresh
              </Button>
            </CardHeader>
            <CardContent>
              {isLoadingStatus ? (
                <div className="text-muted-foreground text-sm">Loading node status...</div>
              ) : nodeStatus ? (
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
                    <span className="text-muted-foreground">Version:</span> {nodeStatus.version}
                  </div>
                  {(nodeStatus.loadAvg?.length ?? 0) > 0 && (
                    <div className="col-span-2">
                      <span className="text-muted-foreground">Load Avg:</span>{' '}
                      {(nodeStatus.loadAvg ?? []).map((v) => v.toFixed(2)).join(' / ')}
                    </div>
                  )}
                </div>
              ) : (
                <div className="text-muted-foreground text-sm">No status data available</div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* ── DNS ─────────────────────────────────────────────────────────────── */}
        <TabsContent value="dns">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>DNS Settings</CardTitle>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void loadDns(clusterId, nodeId)}
                disabled={isLoadingDns}
              >
                <RefreshCw className={`mr-2 h-4 w-4 ${isLoadingDns ? 'animate-spin' : ''}`} />
                Refresh
              </Button>
            </CardHeader>
            <CardContent>
              {isLoadingDns ? (
                <div className="text-muted-foreground text-sm">Loading DNS settings...</div>
              ) : (
                <div className="space-y-4 max-w-md">
                  {nodeDns === null && (
                    <p className="text-sm text-muted-foreground">
                      No DNS data loaded yet. Click Refresh.
                    </p>
                  )}
                  <div className="space-y-1">
                    <Label htmlFor="dns-search">Search Domain</Label>
                    <Input
                      id="dns-search"
                      value={dnsForm.search}
                      onChange={(e) => setDnsForm((f) => ({ ...f, search: e.target.value }))}
                      placeholder="e.g. example.com"
                    />
                  </div>
                  <div className="space-y-1">
                    <Label htmlFor="dns1">DNS Server 1</Label>
                    <Input
                      id="dns1"
                      value={dnsForm.dns1}
                      onChange={(e) => setDnsForm((f) => ({ ...f, dns1: e.target.value }))}
                      placeholder="e.g. 8.8.8.8"
                    />
                  </div>
                  <div className="space-y-1">
                    <Label htmlFor="dns2">DNS Server 2</Label>
                    <Input
                      id="dns2"
                      value={dnsForm.dns2}
                      onChange={(e) => setDnsForm((f) => ({ ...f, dns2: e.target.value }))}
                      placeholder="e.g. 8.8.4.4"
                    />
                  </div>
                  <div className="space-y-1">
                    <Label htmlFor="dns3">DNS Server 3</Label>
                    <Input
                      id="dns3"
                      value={dnsForm.dns3}
                      onChange={(e) => setDnsForm((f) => ({ ...f, dns3: e.target.value }))}
                      placeholder="Optional"
                    />
                  </div>
                  <Button onClick={() => void handleSaveDns()} disabled={isSavingDns}>
                    {isSavingDns ? 'Saving...' : 'Save DNS Settings'}
                  </Button>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* ── Time / NTP ──────────────────────────────────────────────────────── */}
        <TabsContent value="time">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>Time / NTP</CardTitle>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void loadTime(clusterId, nodeId)}
                disabled={isLoadingTime}
              >
                <RefreshCw className={`mr-2 h-4 w-4 ${isLoadingTime ? 'animate-spin' : ''}`} />
                Refresh
              </Button>
            </CardHeader>
            <CardContent>
              {isLoadingTime ? (
                <div className="text-muted-foreground text-sm">Loading time settings...</div>
              ) : (
                <div className="space-y-4 max-w-md">
                  {nodeTime && (
                    <div className="grid grid-cols-2 gap-3 text-sm p-3 bg-muted rounded">
                      <div>
                        <span className="text-muted-foreground">Local Time:</span>{' '}
                        {formatEpoch(nodeTime.localtime)}
                      </div>
                      <div>
                        <span className="text-muted-foreground">UTC Time:</span>{' '}
                        {formatEpoch(nodeTime.time)}
                      </div>
                      <div className="col-span-2">
                        <span className="text-muted-foreground">Timezone:</span>{' '}
                        {nodeTime.timezone}
                      </div>
                    </div>
                  )}
                  {nodeTime === null && (
                    <p className="text-sm text-muted-foreground">
                      No time data loaded yet. Click Refresh.
                    </p>
                  )}
                  <div className="space-y-1">
                    <Label htmlFor="timezone">Timezone</Label>
                    <Input
                      id="timezone"
                      value={timezoneInput}
                      onChange={(e) => setTimezoneInput(e.target.value)}
                      placeholder="e.g. Europe/London"
                    />
                  </div>
                  <Button
                    onClick={() => void handleSaveTime()}
                    disabled={isSavingTime || !timezoneInput.trim()}
                  >
                    {isSavingTime ? 'Saving...' : 'Update Timezone'}
                  </Button>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* ── Journal ─────────────────────────────────────────────────────────── */}
        <TabsContent value="journal">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>System Journal</CardTitle>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void loadJournal(clusterId, nodeId)}
                disabled={isLoadingJournal}
              >
                <RefreshCw className={`mr-2 h-4 w-4 ${isLoadingJournal ? 'animate-spin' : ''}`} />
                {journal.length === 0 ? 'Load Journal' : 'Reload'}
              </Button>
            </CardHeader>
            <CardContent>
              {isLoadingJournal ? (
                <div className="text-muted-foreground text-sm">Loading journal entries...</div>
              ) : journal.length === 0 ? (
                <div className="text-muted-foreground text-sm py-6 text-center">
                  Click "Load Journal" to fetch the last 100 entries
                </div>
              ) : (
                <pre className="font-mono text-xs bg-muted p-3 rounded max-h-[400px] overflow-y-auto whitespace-pre-wrap break-words">
                  {journal.join('\n')}
                </pre>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* ── Report ──────────────────────────────────────────────────────────── */}
        <TabsContent value="report">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>System Report</CardTitle>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void loadReport(clusterId, nodeId)}
                disabled={isLoadingReport}
              >
                <RefreshCw className={`mr-2 h-4 w-4 ${isLoadingReport ? 'animate-spin' : ''}`} />
                {report === '' ? 'Generate Report' : 'Regenerate'}
              </Button>
            </CardHeader>
            <CardContent>
              {isLoadingReport ? (
                <div className="text-muted-foreground text-sm">Generating report...</div>
              ) : report === '' ? (
                <div className="text-muted-foreground text-sm py-6 text-center">
                  Click "Generate Report" to produce a full system report
                </div>
              ) : (
                <pre className="font-mono text-xs bg-muted p-3 rounded max-h-[500px] overflow-y-auto whitespace-pre-wrap break-words">
                  {report}
                </pre>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* ── Admin ───────────────────────────────────────────────────────────── */}
        <TabsContent value="admin">
          <Card>
            <CardHeader>
              <CardTitle>Node Administration</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center gap-2 p-3 bg-destructive/10 border border-destructive/30 rounded text-sm text-destructive">
                <AlertTriangle className="h-4 w-4 shrink-0" />
                <span>These operations affect the physical node and cannot be undone.</span>
              </div>
              <div className="flex items-center gap-3">
                <Button
                  variant="destructive"
                  onClick={() => setConfirmAction('reboot')}
                  disabled={!clusterId}
                >
                  <RotateCcw className="mr-2 h-4 w-4" />
                  Reboot Node
                </Button>
                <Button
                  variant="destructive"
                  onClick={() => setConfirmAction('shutdown')}
                  disabled={!clusterId}
                >
                  <Power className="mr-2 h-4 w-4" />
                  Shutdown Node
                </Button>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Confirmation Dialog */}
      <Dialog
        open={confirmAction !== null}
        onOpenChange={(open) => {
          if (!open) setConfirmAction(null);
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              Confirm {confirmAction === 'reboot' ? 'Reboot' : 'Shutdown'}
            </DialogTitle>
          </DialogHeader>
          <p className="text-sm text-muted-foreground py-2">
            Are you sure you want to{' '}
            <span className="font-medium text-foreground">{confirmAction}</span> node{' '}
            <span className="font-mono font-medium text-foreground">{nodeId}</span>? This will
            affect all workloads running on this node.
          </p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmAction(null)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={() => void handleConfirmAction()}
              disabled={isExecutingAction}
            >
              {isExecutingAction
                ? 'Executing...'
                : confirmAction === 'reboot'
                  ? 'Reboot'
                  : 'Shutdown'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
