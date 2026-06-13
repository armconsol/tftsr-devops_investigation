import React, { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { FirewallRuleList } from '@/components/Proxmox';
import { listProxmoxClusters, listFirewallRules } from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
import { toast } from 'sonner';

export function ProxmoxFirewallPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [selectedClusterId, setSelectedClusterId] = useState<string>('');
  const [nodeInputValue, setNodeInputValue] = useState('localhost');
  const [nodeId, setNodeId] = useState('localhost');
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [rules, setRules] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);

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

  const loadRules = useCallback(async (clusterId: string, nId: string) => {
    if (!clusterId) return;
    setIsLoading(true);
    try {
      const data = await listFirewallRules(clusterId, nId);
      setRules(data);
    } catch (err) {
      console.error('Failed to load firewall rules:', err);
      toast.error('Failed to load firewall rules');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (selectedClusterId) loadRules(selectedClusterId, nodeId);
  }, [selectedClusterId, nodeId, loadRules]);

  const applyNodeId = () => {
    setNodeId(nodeInputValue.trim() || 'localhost');
  };

  if (clusters.length === 0 && !isLoading) {
    return (
      <div className="space-y-4">
        <div>
          <h1 className="text-2xl font-bold">Firewall</h1>
          <p className="text-muted-foreground">Configure firewall rules</p>
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
          <h1 className="text-2xl font-bold">Firewall</h1>
          <p className="text-muted-foreground">Configure firewall rules</p>
        </div>
      </div>

      <div className="flex items-center gap-3 flex-wrap">
        {clusters.length > 0 && (
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">Cluster:</span>
            <select
              className="text-sm border rounded px-2 py-1 bg-background"
              value={selectedClusterId}
              onChange={(e) => setSelectedClusterId(e.target.value)}
            >
              {clusters.map((c) => (
                <option key={c.id} value={c.id}>{c.name}</option>
              ))}
            </select>
          </div>
        )}
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Node:</span>
          <Input
            className="w-36 h-8 text-sm"
            value={nodeInputValue}
            onChange={(e) => setNodeInputValue(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') applyNodeId(); }}
            placeholder="localhost"
          />
          <Button variant="outline" size="sm" onClick={applyNodeId}>Apply</Button>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => loadRules(selectedClusterId, nodeId)}
        >
          <RefreshCw className="mr-2 h-4 w-4" />
          Refresh
        </Button>
      </div>

      <FirewallRuleList
        rules={rules}
        onRefresh={() => loadRules(selectedClusterId, nodeId)}
      />
    </div>
  );
}
