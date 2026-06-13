import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Badge } from '@/components/ui/index';
import { RefreshCw, Network } from 'lucide-react';
import { listNetworkInterfaces, listProxmoxClusters, NetworkInterface } from '@/lib/proxmoxClient';

export function ProxmoxNetworkPage() {
  const [interfaces, setInterfaces] = useState<NetworkInterface[]>([]);
  const [clusterId, setClusterId] = useState('');
  const [nodeId] = useState('localhost');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadInterfaces = useCallback(async (cId: string, nId: string) => {
    if (!cId) return;
    setLoading(true);
    setError(null);
    try {
      const ifaces = await listNetworkInterfaces(cId, nId);
      setInterfaces(ifaces);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        if (cls.length > 0) {
          setClusterId(cls[0].id);
          void loadInterfaces(cls[0].id, nodeId);
        }
      })
      .catch(console.error);
  }, [loadInterfaces, nodeId]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Network</h1>
          <p className="text-muted-foreground">Network interfaces and bridges</p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => void loadInterfaces(clusterId, nodeId)}
          disabled={loading || !clusterId}
        >
          <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {error && (
        <div className="rounded border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </div>
      )}

      <Card>
        <CardHeader>
          <CardTitle>Network Interfaces</CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="text-sm text-muted-foreground">Loading...</div>
          ) : interfaces.length === 0 ? (
            <div className="text-sm text-muted-foreground">
              {clusterId ? 'No network interfaces found.' : 'No cluster configured.'}
            </div>
          ) : (
            <div className="space-y-2">
              {interfaces.map((iface, i) => (
                <div key={`${iface.iface}-${i}`} className="flex items-center gap-3 rounded border p-3">
                  <Network className="h-4 w-4 shrink-0 text-muted-foreground" />
                  <div className="flex-1 min-w-0">
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="font-mono font-medium">{iface.iface}</span>
                      <Badge variant="outline">{iface.type}</Badge>
                      <Badge variant={iface.active ? 'default' : 'secondary'}>
                        {iface.active ? 'Active' : 'Inactive'}
                      </Badge>
                      {iface.autostart && (
                        <Badge variant="outline" className="text-xs">Autostart</Badge>
                      )}
                    </div>
                    {(iface.address || iface.gateway) && (
                      <div className="mt-1 text-xs text-muted-foreground">
                        {iface.address && (
                          <span>
                            {iface.address}
                            {iface.netmask ? `/${iface.netmask}` : ''}
                          </span>
                        )}
                        {iface.gateway && (
                          <span className="ml-2">gw {iface.gateway}</span>
                        )}
                      </div>
                    )}
                    {iface.comments && (
                      <div className="mt-1 text-xs italic text-muted-foreground">
                        {iface.comments}
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
