import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Badge } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { listClusterTasks, listProxmoxClusters, ClusterTask } from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';

function taskBadgeVariant(exitstatus?: string): 'default' | 'destructive' | 'secondary' {
  if (!exitstatus) return 'secondary';
  return exitstatus === 'OK' ? 'default' : 'destructive';
}

function taskBadgeLabel(exitstatus?: string): string {
  if (!exitstatus) return 'running';
  return exitstatus;
}

function formatTimestamp(epoch: number): string {
  if (!epoch) return '-';
  return new Date(epoch * 1000).toLocaleString();
}

export function ProxmoxTasksPage() {
  const [tasks, setTasks] = useState<ClusterTask[]>([]);
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [clusterId, setClusterId] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadTasks = useCallback(async (cId: string) => {
    if (!cId) return;
    setLoading(true);
    setError(null);
    try {
      const t = await listClusterTasks(cId, 100);
      setTasks(t);
    } catch (e) {
      setError(String(e));
      console.error(e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        setClusters(cls);
        if (cls.length > 0) {
          setClusterId(cls[0].id);
        }
      })
      .catch(console.error);
  }, []);

  useEffect(() => {
    if (clusterId) void loadTasks(clusterId);
    else setTasks([]);
  }, [clusterId, loadTasks]);

  const runningCount = tasks.filter((t) => !t.exitstatus).length;
  const completedCount = tasks.filter((t) => t.exitstatus === 'OK').length;
  const failedCount = tasks.filter(
    (t) => t.exitstatus && t.exitstatus !== 'OK'
  ).length;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Tasks</h1>
          <p className="text-muted-foreground">Cluster task log and operations</p>
        </div>
        <div className="flex items-center gap-2">
          {clusters.length > 1 && (
            <Select value={clusterId} onValueChange={setClusterId}>
              <SelectTrigger className="h-8 w-48 text-sm">
                <SelectValue placeholder="Select datacenter" />
              </SelectTrigger>
              <SelectContent>
                {clusters.map((c) => (
                  <SelectItem key={c.id} value={c.id}>{c.name}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
          <Button
            variant="outline"
            size="sm"
            onClick={() => void loadTasks(clusterId)}
            disabled={loading || !clusterId}
          >
            <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {error && (
        <div className="rounded border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </div>
      )}

      <div className="grid grid-cols-3 gap-4">
        <Card>
          <CardContent className="pt-4">
            <div className="text-2xl font-bold text-yellow-500">{runningCount}</div>
            <div className="text-sm text-muted-foreground">Running</div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-4">
            <div className="text-2xl font-bold text-green-500">{completedCount}</div>
            <div className="text-sm text-muted-foreground">Completed</div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-4">
            <div className="text-2xl font-bold text-red-500">{failedCount}</div>
            <div className="text-sm text-muted-foreground">Failed</div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Task Log</CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="text-sm text-muted-foreground">Loading tasks...</div>
          ) : tasks.length === 0 ? (
            <div className="text-sm text-muted-foreground">
              {clusterId ? 'No tasks found.' : 'No cluster configured.'}
            </div>
          ) : (
            <div className="space-y-0">
              {tasks.map((t, i) => (
                <div
                  key={`${t.upid}-${i}`}
                  className="flex flex-wrap items-center gap-3 border-b py-2 text-sm last:border-0"
                >
                  <Badge variant={taskBadgeVariant(t.exitstatus)}>
                    {taskBadgeLabel(t.exitstatus)}
                  </Badge>
                  <span className="font-medium">{t.type}</span>
                  <span className="text-muted-foreground">{t.node}</span>
                  <span className="text-xs text-muted-foreground">{t.user}</span>
                  <span className="text-xs text-muted-foreground">
                    {formatTimestamp(t.starttime)}
                  </span>
                  <span className="ml-auto max-w-xs truncate font-mono text-xs text-muted-foreground">
                    {t.upid}
                  </span>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
