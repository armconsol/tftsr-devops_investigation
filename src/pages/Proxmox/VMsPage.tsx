import React, { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { VMList } from '@/components/Proxmox';
import { listProxmoxClusters, listProxmoxVms } from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
import { toast } from 'sonner';

export function ProxmoxVMsPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [selectedClusterId, setSelectedClusterId] = useState<string>('');
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [vms, setVms] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedVMs, setSelectedVMs] = useState<Set<string>>(new Set());

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

  const loadVms = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setIsLoading(true);
    try {
      const data = await listProxmoxVms(clusterId);
      setVms(data);
    } catch (err) {
      console.error('Failed to load VMs:', err);
      toast.error('Failed to load VMs');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (selectedClusterId) loadVms(selectedClusterId);
  }, [selectedClusterId, loadVms]);

  if (clusters.length === 0 && !isLoading) {
    return (
      <div className="space-y-4">
        <div>
          <h1 className="text-2xl font-bold">Virtual Machines</h1>
          <p className="text-muted-foreground">Manage QEMU/KVM virtual machines</p>
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
          <h1 className="text-2xl font-bold">Virtual Machines</h1>
          <p className="text-muted-foreground">Manage QEMU/KVM virtual machines</p>
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
          <Button variant="outline" size="sm" onClick={() => loadVms(selectedClusterId)}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      <VMList
        vms={vms}
        onRefresh={() => loadVms(selectedClusterId)}
        onVMAction={(_vm, _action) => { toast.info('VM action — not yet implemented'); }}
        onSnapshotAction={(_vm, _action) => { toast.info('Snapshot action — not yet implemented'); }}
        onMigrate={(_vm) => { toast.info('Migrate — not yet implemented'); }}
        onClone={(_vm) => { toast.info('Clone — not yet implemented'); }}
        onDelete={(_vm) => { toast.info('Delete — not yet implemented'); }}
        selectedVMs={selectedVMs}
        onToggleSelect={(vm) => {
          setSelectedVMs((prev) => {
            const next = new Set(prev);
            const id = String(vm.vmid);
            if (next.has(id)) next.delete(id); else next.add(id);
            return next;
          });
        }}
      />
    </div>
  );
}
