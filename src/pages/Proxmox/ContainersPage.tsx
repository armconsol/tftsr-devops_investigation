import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { ContainerOverview } from '@/components/Proxmox';
import {
  listProxmoxClusters,
  listProxmoxContainers,
  startProxmoxContainer,
  stopProxmoxContainer,
  rebootProxmoxContainer,
  shutdownProxmoxContainer,
} from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
import { toast } from 'sonner';

export function ProxmoxContainersPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [selectedClusterId, setSelectedClusterId] = useState<string>('');
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [containers, setContainers] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [selectedContainer, setSelectedContainer] = useState<any | null>(null);

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
              <CardHeader>
                <CardTitle>{container.name ?? `CT ${container.vmid}`}</CardTitle>
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
    </div>
  );
}
