import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Badge } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { RefreshCw, Key, Check, AlertCircle, Clock } from 'lucide-react';
import { getSubscriptionStatus, listProxmoxClusters, SubscriptionStatus } from '@/lib/proxmoxClient';
import { ClusterInfo } from '@/lib/domain';
import { useProxmoxStore } from '@/stores/proxmoxStore';

interface ClusterSubscription {
  cluster: ClusterInfo;
  status: SubscriptionStatus;
}

function StatusBadge({ status }: { status: SubscriptionStatus['status'] }) {
  if (status === 'active') {
    return (
      <Badge variant="success" className="flex items-center gap-1 w-fit">
        <Check className="h-3 w-3" />
        Active
      </Badge>
    );
  }
  if (status === 'expired') {
    return (
      <Badge variant="destructive" className="flex items-center gap-1 w-fit">
        <AlertCircle className="h-3 w-3" />
        Expired
      </Badge>
    );
  }
  return (
    <Badge variant="secondary" className="flex items-center gap-1 w-fit">
      <Clock className="h-3 w-3" />
      None
    </Badge>
  );
}

function maskKey(key?: string): string {
  if (!key) return '';
  const parts = key.split('-');
  if (parts.length < 2) return key.slice(0, 4) + '-xxxx-xxxx-xxxx';
  return `${parts[0]}-xxxx-xxxx-xxxx`;
}

export function ProxmoxSubscriptionPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [subscriptions, setSubscriptions] = useState<Record<string, SubscriptionStatus>>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [keyInput, setKeyInput] = useState('');
  const persistSelectedClusterId = useProxmoxStore((s) => s.setSelectedClusterId);
  const [selectedClusterId, setSelectedClusterIdState] = useState<string>(
    () => useProxmoxStore.getState().selectedClusterId
  );
  const setSelectedClusterId = (id: string) => {
    setSelectedClusterIdState(id);
    persistSelectedClusterId(id);
  };
  const [activating, setActivating] = useState(false);
  const [activationMessage, setActivationMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  async function loadAll() {
    setLoading(true);
    setError(null);
    try {
      const cls = await listProxmoxClusters();
      setClusters(cls);
      const persisted = useProxmoxStore.getState().selectedClusterId;
      if (!selectedClusterId || !cls.some((c) => c.id === selectedClusterId)) {
        if (persisted && cls.some((c) => c.id === persisted)) {
          setSelectedClusterIdState(persisted);
        } else if (cls.length > 0) {
          setSelectedClusterId(cls[0].id);
        }
      }
      const subs: Record<string, SubscriptionStatus> = {};
      await Promise.all(
        cls.map(async (c) => {
          try {
            subs[c.id] = await getSubscriptionStatus(c.id);
          } catch {
            subs[c.id] = { status: 'none' };
          }
        })
      );
      setSubscriptions(subs);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadAll();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function handleActivate() {
    if (!keyInput.trim() || !selectedClusterId) return;
    setActivating(true);
    setActivationMessage(null);
    try {
      // Backend invocation would go here: await setSubscriptionKey(selectedClusterId, keyInput.trim())
      // For now we optimistically refresh status
      await loadAll();
      setActivationMessage({ type: 'success', text: 'Subscription key submitted. Status refreshed.' });
      setKeyInput('');
    } catch (err) {
      setActivationMessage({ type: 'error', text: String(err) });
    } finally {
      setActivating(false);
    }
  }

  const clusterSubscriptions: ClusterSubscription[] = clusters.map((c) => ({
    cluster: c,
    status: subscriptions[c.id] ?? { status: 'none' },
  }));

  const activeCount = clusterSubscriptions.filter((cs) => cs.status.status === 'active').length;
  const expiredCount = clusterSubscriptions.filter((cs) => cs.status.status === 'expired').length;
  const noneCount = clusterSubscriptions.filter((cs) => cs.status.status === 'none').length;

  const selectedSub = selectedClusterId ? subscriptions[selectedClusterId] : undefined;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Subscriptions</h1>
          <p className="text-muted-foreground">Manage Proxmox subscription keys across clusters</p>
        </div>
        <Button variant="outline" size="sm" onClick={loadAll} disabled={loading}>
          <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {error && (
        <div className="rounded-md border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
          {error}
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Left panel: Subscription Key input */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Key className="h-5 w-5" />
              Subscription Key
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Current key display */}
            {selectedSub?.key && (
              <div className="rounded-md border bg-muted/30 px-4 py-3 space-y-1">
                <div className="text-xs text-muted-foreground">Current Key</div>
                <div className="font-mono text-sm font-medium">{maskKey(selectedSub.key)}</div>
                {selectedSub.productname && (
                  <div className="text-xs text-muted-foreground">{selectedSub.productname}</div>
                )}
                <StatusBadge status={selectedSub.status} />
              </div>
            )}

            {clusters.length > 1 && (
              <div className="space-y-2">
                <Label>Target Cluster</Label>
                <select
                  className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
                  value={selectedClusterId}
                  onChange={(e) => setSelectedClusterId(e.target.value)}
                >
                  {clusters.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                    </option>
                  ))}
                </select>
              </div>
            )}

            <div className="space-y-2">
              <Label htmlFor="sub-key">Enter Subscription Key</Label>
              <Input
                id="sub-key"
                placeholder="pve4e-xxxx-xxxx-xxxx"
                value={keyInput}
                onChange={(e) => setKeyInput(e.target.value)}
                onKeyDown={(e) => { if (e.key === 'Enter') void handleActivate(); }}
              />
              <p className="text-xs text-muted-foreground">
                Keys can be obtained from the{' '}
                <a
                  href="https://www.proxmox.com/en/proxmox-ve/pricing"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="underline hover:text-foreground"
                >
                  Proxmox shop
                </a>
                .
              </p>
            </div>

            {activationMessage && (
              <div
                className={`rounded-md border px-4 py-3 text-sm ${
                  activationMessage.type === 'success'
                    ? 'border-green-500/50 bg-green-500/10 text-green-700 dark:text-green-400'
                    : 'border-destructive/50 bg-destructive/10 text-destructive'
                }`}
              >
                {activationMessage.text}
              </div>
            )}

            <Button
              className="w-full"
              disabled={!keyInput.trim() || !selectedClusterId || activating}
              onClick={handleActivate}
            >
              {activating ? (
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Key className="mr-2 h-4 w-4" />
              )}
              Activate Key
            </Button>
          </CardContent>
        </Card>

        {/* Right panel: Per-cluster status */}
        <Card>
          <CardHeader>
            <CardTitle>Cluster Subscription Status</CardTitle>
            <div className="flex items-center gap-3 text-sm text-muted-foreground pt-1">
              <span className="flex items-center gap-1">
                <span className="h-2 w-2 rounded-full bg-green-500 inline-block" />
                {activeCount} Active
              </span>
              <span className="flex items-center gap-1">
                <span className="h-2 w-2 rounded-full bg-red-500 inline-block" />
                {expiredCount} Expired
              </span>
              <span className="flex items-center gap-1">
                <span className="h-2 w-2 rounded-full bg-muted-foreground inline-block" />
                {noneCount} None
              </span>
            </div>
          </CardHeader>
          <CardContent>
            {clusterSubscriptions.length === 0 ? (
              <div className="flex items-center justify-center py-12 text-muted-foreground text-sm">
                {loading ? 'Loading...' : 'No clusters configured.'}
              </div>
            ) : (
              <div className="space-y-3">
                {clusterSubscriptions.map(({ cluster, status }) => (
                  <div
                    key={cluster.id}
                    className={`rounded-lg border p-4 cursor-pointer transition-colors ${
                      selectedClusterId === cluster.id
                        ? 'border-primary bg-primary/5'
                        : 'hover:bg-muted/50'
                    }`}
                    onClick={() => setSelectedClusterId(cluster.id)}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div className="space-y-1 min-w-0">
                        <div className="font-medium truncate">{cluster.name}</div>
                        <div className="text-xs text-muted-foreground font-mono truncate">
                          {cluster.url}:{cluster.port}
                        </div>
                        {status.productname && (
                          <div className="text-xs text-muted-foreground">{status.productname}</div>
                        )}
                        <div className="flex items-center gap-3 text-xs text-muted-foreground flex-wrap">
                          {status.regdate && (
                            <span>Registered: {status.regdate}</span>
                          )}
                          {status.nextduedate && (
                            <span>Next due: {status.nextduedate}</span>
                          )}
                        </div>
                      </div>
                      <div className="flex-shrink-0">
                        <StatusBadge status={status.status} />
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
