import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Badge } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/index';
import { RefreshCw, Search } from 'lucide-react';
import {
  listClusterTasks,
  ClusterTask,
  searchTaskLogs,
  getProxmoxTaskLog,
  type TaskLogSearchResult,
  type TaskLogEntry,
} from '@/lib/proxmoxClient';
import { useProxmoxClusters } from '@/hooks/useProxmoxClusters';
import { toast } from 'sonner';

const MIN_SEARCH_QUERY_LENGTH = 2;

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
  const { clusters, selectedClusterId: clusterId, setSelectedClusterId: setClusterId } = useProxmoxClusters();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [logQuery, setLogQuery] = useState('');
  const [logSearching, setLogSearching] = useState(false);
  const [logResults, setLogResults] = useState<TaskLogSearchResult[] | null>(null);
  const [logDialogTarget, setLogDialogTarget] = useState<{ node: string; upid: string } | null>(null);
  const [logDialogEntries, setLogDialogEntries] = useState<TaskLogEntry[]>([]);
  const [logDialogLoading, setLogDialogLoading] = useState(false);

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
    if (clusterId) void loadTasks(clusterId);
    else setTasks([]);
  }, [clusterId, loadTasks]);

  // Loaded tasks change identity on every refresh, so clear any stale search
  // results rather than showing matches against tasks that are no longer listed.
  useEffect(() => {
    setLogResults(null);
  }, [tasks]);

  const handleSearchLogs = useCallback(async () => {
    const query = logQuery.trim();
    if (query.length < MIN_SEARCH_QUERY_LENGTH || tasks.length === 0) return;
    setLogSearching(true);
    try {
      const targets = tasks.map((t) => ({ node: t.node, upid: t.upid }));
      const results = await searchTaskLogs(clusterId, query, targets);
      setLogResults(results.filter((r) => r.matches.length > 0));
    } catch (e) {
      toast.error(`Failed to search task logs: ${e}`);
    } finally {
      setLogSearching(false);
    }
  }, [clusterId, logQuery, tasks]);

  const handleViewFullLog = useCallback(async (node: string, upid: string) => {
    setLogDialogTarget({ node, upid });
    setLogDialogLoading(true);
    setLogDialogEntries([]);
    try {
      const entries = await getProxmoxTaskLog(clusterId, node, upid);
      setLogDialogEntries(entries);
    } catch (e) {
      toast.error(`Failed to load task log: ${e}`);
    } finally {
      setLogDialogLoading(false);
    }
  }, [clusterId]);

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
          <CardTitle>Search Task Logs</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center gap-2">
            <Input
              placeholder="Search task logs..."
              value={logQuery}
              onChange={(e) => setLogQuery(e.target.value)}
              className="max-w-sm"
            />
            <Button
              variant="outline"
              size="sm"
              onClick={() => void handleSearchLogs()}
              disabled={logSearching || logQuery.trim().length < MIN_SEARCH_QUERY_LENGTH || tasks.length === 0}
            >
              <Search className={`mr-2 h-4 w-4 ${logSearching ? 'animate-pulse' : ''}`} />
              Search Logs
            </Button>
          </div>

          {logResults !== null && (
            logResults.length === 0 ? (
              <div className="text-sm text-muted-foreground">No matching log lines found.</div>
            ) : (
              <div className="space-y-2">
                {logResults.map((result) => (
                  <div key={result.upid} className="rounded border p-2 text-sm">
                    <div className="flex items-center justify-between">
                      <div className="font-mono text-xs text-muted-foreground truncate max-w-md">
                        {result.node} — {result.upid}
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => void handleViewFullLog(result.node, result.upid)}
                      >
                        View Full Log
                      </Button>
                    </div>
                    <div className="mt-1 space-y-0.5 font-mono text-xs">
                      {result.matches.map((entry) => (
                        <div key={entry.n} className="text-amber-700 dark:text-amber-400">
                          {entry.t}
                        </div>
                      ))}
                    </div>
                    {result.error && (
                      <div className="mt-1 text-xs text-destructive">{result.error}</div>
                    )}
                  </div>
                ))}
              </div>
            )
          )}
        </CardContent>
      </Card>

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

      <Dialog open={!!logDialogTarget} onOpenChange={(open) => !open && setLogDialogTarget(null)}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>
              Task Log — {logDialogTarget?.node} ({logDialogTarget?.upid})
            </DialogTitle>
          </DialogHeader>
          <div className="max-h-96 overflow-y-auto rounded-md border bg-muted/50 p-2 font-mono text-xs space-y-0.5">
            {logDialogLoading ? (
              <div className="text-muted-foreground">Loading log...</div>
            ) : logDialogEntries.length === 0 ? (
              <div className="text-muted-foreground">No log entries.</div>
            ) : (
              logDialogEntries.map((entry) => <div key={entry.n}>{entry.t}</div>)
            )}
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
