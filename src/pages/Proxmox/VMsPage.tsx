import React, { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/index';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/index';
import { RefreshCw, Plus } from 'lucide-react';
import { VMList, CreateVmDialog } from '@/components/Proxmox';
import { listProxmoxVms, getVmConfig } from '@/lib/proxmoxClient';
import { useProxmoxClusters } from '@/hooks/useProxmoxClusters';
import { toast } from 'sonner';

export function ProxmoxVMsPage() {
  const { clusters, selectedClusterId, setSelectedClusterId } = useProxmoxClusters();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [vms, setVms] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedVMs, setSelectedVMs] = useState<Set<string>>(new Set());
  const [showCreateDialog, setShowCreateDialog] = useState(false);

  const [configVm, setConfigVm] = useState<{ node: string; vmid: number } | null>(null);
  const [vmConfig, setVmConfig] = useState<Record<string, unknown> | null>(null);
  const [configLoading, setConfigLoading] = useState(false);

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

  const handleViewConfig = async (node: string, vmid: number) => {
    setConfigVm({ node, vmid });
    setConfigLoading(true);
    setVmConfig(null);
    try {
      const config = await getVmConfig(selectedClusterId, node, vmid);
      setVmConfig(config as Record<string, unknown>);
    } catch (e) {
      toast.error(String(e));
    } finally {
      setConfigLoading(false);
    }
  };

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
          <Button size="sm" onClick={() => setShowCreateDialog(true)} disabled={!selectedClusterId}>
            <Plus className="mr-2 h-4 w-4" />
            Add VM
          </Button>
        </div>
      </div>

      <VMList
        vms={vms}
        clusterId={selectedClusterId}
        clusters={clusters}
        onRefresh={() => loadVms(selectedClusterId)}
        selectedVMs={selectedVMs}
        onToggleSelect={(vm) => {
          setSelectedVMs((prev) => {
            const next = new Set(prev);
            const id = String(vm.vmid);
            if (next.has(id)) next.delete(id); else next.add(id);
            return next;
          });
        }}
        onViewConfig={(node, vmid) => void handleViewConfig(node, vmid)}
      />

      <CreateVmDialog
        isOpen={showCreateDialog}
        clusterId={selectedClusterId}
        onClose={() => setShowCreateDialog(false)}
        onCreated={() => loadVms(selectedClusterId)}
      />

      <Dialog open={!!configVm} onOpenChange={() => { setConfigVm(null); setVmConfig(null); }}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>VM Config: {configVm?.vmid}</DialogTitle>
          </DialogHeader>
          {configLoading ? (
            <div className="text-sm text-muted-foreground">Loading...</div>
          ) : (
            <pre className="text-xs font-mono bg-muted p-3 rounded max-h-96 overflow-auto">
              {JSON.stringify(vmConfig, null, 2)}
            </pre>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
