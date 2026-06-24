import React, { useState, useEffect, useCallback, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { clsx } from 'clsx';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Checkbox } from '@/components/ui/index';
import { MoreHorizontal, Play, Square, RotateCcw, Power, PlayCircle, Pause, X, MoveRight, Copy, Settings, Monitor } from 'lucide-react';
import { confirm } from '@tauri-apps/plugin-dialog';
import { toast } from 'sonner';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter, DialogDescription } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Checkbox as UICheckbox } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { AlertCircle } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/index';
import type { ClusterInfo } from '@/lib/domain';
import type { ProxmoxSnapshot } from '@/lib/proxmoxClient';
import {
  cloneVm,
  deleteVm,
  startProxmoxVm,
  stopProxmoxVm,
  rebootProxmoxVm,
  shutdownProxmoxVm,
  suspendProxmoxVm,
  resumeProxmoxVm,
  listProxmoxSnapshots,
  createProxmoxSnapshot,
  deleteProxmoxSnapshot,
  rollbackProxmoxSnapshot,
  listProxmoxNodes,
  migrateVm,
  startRemoteMigration,
  getTaskStatus,
  deleteUserToken,
} from '@/lib/proxmoxClient';

interface VMInfo {
  id: string;
  vmid: number;
  name: string;
  node: string;
  status: 'running' | 'stopped' | 'paused';
  cpu: number;
  memory: number;
  memoryTotal: number;
  uptime?: number;
  tags?: string[];
}

interface RawVMInfo {
  id: number;
  vmid?: number;
  name?: string;
  node?: string;
  status?: string;
  cpu?: number;
  mem?: number;
  max_mem?: number;
  memory?: number;
  memoryTotal?: number;
  disk?: number;
  max_disk?: number;
  diskTotal?: number;
  uptime?: number;
  tags?: string[];
}

interface VMListProps {
  vms: RawVMInfo[];
  clusterId: string;
  clusters?: ClusterInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onSnapshotAction?: (vm: VMInfo, action: 'create' | 'list' | 'rollback' | 'delete') => void;
  onMigrate?: (vm: VMInfo) => void;
  onClone?: (vm: VMInfo) => void;
  onDelete?: (vm: VMInfo) => void;
  selectedVMs?: Set<string>;
  onToggleSelect?: (vm: VMInfo) => void;
  onViewConfig?: (node: string, vmid: number) => void;
}

function formatUptime(seconds: number): string {
  if (seconds <= 0) return '-';

  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  const parts: string[] = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  if (minutes > 0 || parts.length === 0) parts.push(`${minutes}m`);

  return parts.join(' ');
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function VMList({
  vms: rawVms,
  clusterId,
  clusters = [],
  onRefresh,
  isLoading,
  onSnapshotAction: _onSnapshotAction,
  onMigrate: _onMigrate,
  onClone: _onClone,
  onDelete: _onDelete,
  selectedVMs = new Set<string>(),
  onToggleSelect,
  onViewConfig,
}: VMListProps) {
  const navigate = useNavigate();
  const [migrationVM, setMigrationVM] = useState<VMInfo | null>(null);
  const [targetNode, setTargetNode] = useState<string>('');
  const [targetCluster, setTargetCluster] = useState<string>('');
  const [targetStorage, setTargetStorage] = useState<string>('local-lvm');
  const [targetBridge, setTargetBridge] = useState<string>('vmbr0');
  const [onlineMigration, setOnlineMigration] = useState(true);
  const [maxDowntime, setMaxDowntime] = useState(30);
  const [clusterNodes, setClusterNodes] = useState<string[]>([]);
  const [nodesLoading, setNodesLoading] = useState(false);
  const [migrationRunning, setMigrationRunning] = useState(false);
  const [migrationLog, setMigrationLog] = useState<string[]>([]);
  const [snapshotDialog, setSnapshotDialog] = useState<{
    isOpen: boolean;
    vm: VMInfo | null;
    action: 'create' | 'list' | 'rollback' | 'delete' | null;
    snapshots: ProxmoxSnapshot[];
  }>({ isOpen: false, vm: null, action: null, snapshots: [] });
  const [snapshotName, setSnapshotName] = useState('');
  const [selectedSnapshot, setSelectedSnapshot] = useState('');
  const [cloneDialog, setCloneDialog] = useState<{ isOpen: boolean; vm: VMInfo | null }>({ isOpen: false, vm: null });
  const [cloneVmid, setCloneVmid] = useState('');
  const [cloneName, setCloneName] = useState('');
  const [cloneSubmitting, setCloneSubmitting] = useState(false);

  const vms: VMInfo[] = React.useMemo(() => {
    return rawVms.map((vm) => ({
      id: String(vm.id || vm.vmid),
      vmid: vm.vmid || vm.id,
      name: vm.name || `VM-${vm.vmid || vm.id}`,
      node: vm.node || '',
      status: (vm.status || 'stopped') as 'running' | 'stopped' | 'paused',
      cpu: vm.cpu || 0,
      memory: vm.mem || vm.memory || 0,
      memoryTotal: vm.max_mem || vm.memoryTotal || 0,
      uptime: vm.uptime,
      tags: vm.tags,
    }));
  }, [rawVms]);

  // clusterId comes from props (not captured via closure over state), so it is always
  // current when an action fires even if the user switches clusters mid-session.
  const handleVMAction = useCallback(async (vm: VMInfo, action: string) => {
    if (!clusterId) {
      toast.error('No cluster selected');
      return;
    }
    try {
      switch (action) {
        case 'start':
          await startProxmoxVm(clusterId, vm.node, vm.vmid);
          toast.success(`VM ${vm.name} started`);
          onRefresh?.();
          break;
        case 'stop':
          await stopProxmoxVm(clusterId, vm.node, vm.vmid);
          toast.success(`VM ${vm.name} stopped`);
          onRefresh?.();
          break;
        case 'reboot':
          await rebootProxmoxVm(clusterId, vm.node, vm.vmid);
          toast.success(`VM ${vm.name} rebooting`);
          onRefresh?.();
          break;
        case 'shutdown':
          await shutdownProxmoxVm(clusterId, vm.node, vm.vmid);
          toast.success(`VM ${vm.name} shutting down`);
          onRefresh?.();
          break;
        case 'resume':
          await resumeProxmoxVm(clusterId, vm.node, vm.vmid);
          toast.success(`VM ${vm.name} resumed`);
          onRefresh?.();
          break;
        case 'suspend':
          await suspendProxmoxVm(clusterId, vm.node, vm.vmid);
          toast.success(`VM ${vm.name} suspended`);
          onRefresh?.();
          break;
        default:
          toast.error(`Unknown action: ${action}`);
      }
    } catch (error) {
      console.error(`Failed to ${action} VM ${vm.name}:`, error);
      toast.error(`Failed to ${action} VM ${vm.name}: ${error}`);
    }
  }, [clusterId, onRefresh]);

  const handleSnapshotAction = useCallback(async (vm: VMInfo, action: 'create' | 'list' | 'rollback' | 'delete') => {
    if (action === 'list') {
      try {
        const snapshots = await listProxmoxSnapshots(clusterId, vm.node, vm.vmid);
        setSnapshotDialog({ isOpen: true, vm, action: 'list', snapshots });
      } catch (error) {
        console.error('Failed to list snapshots:', error);
        toast.error(`Failed to list snapshots: ${error}`);
      }
      return;
    }

    if (action === 'rollback' || action === 'delete') {
      try {
        const snapshots = await listProxmoxSnapshots(clusterId, vm.node, vm.vmid);
        if (snapshots.length === 0) {
          toast.error(`No snapshots found for ${vm.name}`);
          return;
        }
        setSnapshotDialog({ isOpen: true, vm, action, snapshots });
      } catch (error) {
        console.error('Failed to list snapshots:', error);
        toast.error(`Failed to list snapshots: ${error}`);
      }
      return;
    }

    if (action === 'create') {
      setSnapshotName('');
      setSnapshotDialog({ isOpen: true, vm, action: 'create', snapshots: [] });
    }
  }, [clusterId]);

  const handleSnapshotSubmit = useCallback(async () => {
    if (!snapshotDialog.vm || !snapshotDialog.action) return;

    const { vm, action } = snapshotDialog;

    try {
      if (action === 'create') {
        if (!snapshotName.trim()) {
          toast.error('Snapshot name is required');
          return;
        }
        await createProxmoxSnapshot(clusterId, vm.node, vm.vmid, snapshotName.trim());
        toast.success(`Snapshot "${snapshotName}" created for ${vm.name}`);
      } else if (action === 'rollback' && selectedSnapshot) {
        if (await confirm(`Are you sure you want to rollback ${vm.name} to "${selectedSnapshot}"? This may cause downtime.`)) {
          await rollbackProxmoxSnapshot(clusterId, vm.node, vm.vmid, selectedSnapshot);
          toast.success(`Rolled back ${vm.name} to "${selectedSnapshot}"`);
        }
      } else if (action === 'delete' && selectedSnapshot) {
        if (await confirm(`Are you sure you want to delete snapshot "${selectedSnapshot}" for ${vm.name}?`)) {
          await deleteProxmoxSnapshot(clusterId, vm.node, vm.vmid, selectedSnapshot);
          toast.success(`Deleted snapshot "${selectedSnapshot}" for ${vm.name}`);
        }
      }
      setSnapshotDialog({ isOpen: false, vm: null, action: null, snapshots: [] });
      setSnapshotName('');
      setSelectedSnapshot('');
      onRefresh?.();
    } catch (error) {
      console.error(`Failed to ${action} snapshot:`, error);
      toast.error(`Failed to ${action} snapshot: ${error}`);
    }
  }, [snapshotDialog, clusterId, snapshotName, selectedSnapshot, onRefresh]);

  const handleSnapshotClose = useCallback(() => {
    setSnapshotDialog({ isOpen: false, vm: null, action: null, snapshots: [] });
    setSnapshotName('');
    setSelectedSnapshot('');
  }, []);

  const handleConsole = useCallback((vm: VMInfo) => {
    if (!clusterId) { toast.error('No cluster selected'); return; }
    navigate(`/proxmox/console/${encodeURIComponent(clusterId)}/${encodeURIComponent(vm.node)}/${vm.vmid}/qemu`);
  }, [clusterId, navigate]);

  const handleMigrate = useCallback(async (vm: VMInfo) => {
    setMigrationVM(vm);
    setTargetCluster(clusterId);
    setNodesLoading(true);
    try {
      const nodeData: { node?: string; status?: string }[] = await listProxmoxNodes(clusterId) as { node?: string; status?: string }[];
      const names = nodeData
        .filter((n) => n.node && n.node !== vm.node)
        .map((n) => n.node as string);
      setClusterNodes(names);
      setTargetNode(names[0] || '');
    } catch {
      const fallback = vms
        .map((v) => v.node)
        .filter((node, idx, self) => self.indexOf(node) === idx && node !== vm.node);
      setClusterNodes(fallback);
      setTargetNode(fallback[0] || '');
    } finally {
      setNodesLoading(false);
    }
  }, [clusterId, vms]);

  const submitMigration = useCallback(async () => {
    if (!migrationVM || !targetNode) {
      toast.error('Please select a target node');
      return;
    }

    const sourceCluster = clusterId;
    const destCluster = targetCluster || clusterId;
    const isCrossCluster = destCluster !== sourceCluster;
    const vm = migrationVM;

    setMigrationRunning(true);
    setMigrationLog([
      `Starting ${isCrossCluster ? 'cross-datacenter ' : ''}migration of ${vm.name} (VM ${vm.vmid}) → ${targetNode}…`,
    ]);

    // Poll the source-node task until it stops, surfacing the real Proxmox
    // exit status instead of an immediate (false) success.
    const pollTask = async (sourceNode: string, upid: string): Promise<string | null> => {
      // Returns null on success, or an error string on failure.
      // eslint-disable-next-line no-constant-condition
      while (true) {
        await new Promise((r) => setTimeout(r, 2000));
        let status;
        try {
          status = await getTaskStatus(sourceCluster, sourceNode, upid);
        } catch (e) {
          setMigrationLog((prev) => [...prev, `Polling task status failed: ${e}`]);
          continue;
        }
        if (typeof status.progress === 'number' && status.progress > 0) {
          setMigrationLog((prev) => [...prev, `Progress: ${status.progress}%`]);
        }
        if (status.status === 'stopped') {
          const exit = (status.exit_status ?? '').toString().trim();
          if (exit === 'OK') return null;
          return exit || 'Task stopped without an exit status';
        }
      }
    };

    try {
      if (isCrossCluster) {
        const started = await startRemoteMigration(
          sourceCluster,
          vm.node,
          vm.vmid,
          destCluster,
          targetNode,
          targetStorage || 'local-lvm',
          targetBridge || 'vmbr0',
          onlineMigration
        );
        setMigrationLog((prev) => [...prev, `Task started: ${started.upid}`]);
        const err = await pollTask(started.source_node || vm.node, started.upid);
        // Clean up the temporary destination token regardless of outcome.
        try {
          await deleteUserToken(started.dest_cluster_id, started.dest_userid, started.dest_tokenname);
        } catch (cleanupErr) {
          console.warn('Failed to delete temporary migration token:', cleanupErr);
        }
        if (err) {
          setMigrationLog((prev) => [...prev, `ERROR: ${err}`]);
          toast.error(`Migration of ${vm.name} failed: ${err}`);
          return;
        }
      } else {
        const task = await migrateVm(sourceCluster, vm.node, vm.vmid, targetNode, destCluster);
        const upid = task.task_id;
        const sourceNode = task.source_node || vm.node;
        if (!upid) {
          throw new Error('Proxmox did not return a migration task id');
        }
        setMigrationLog((prev) => [...prev, `Task started: ${upid}`]);
        const err = await pollTask(sourceNode, upid);
        if (err) {
          setMigrationLog((prev) => [...prev, `ERROR: ${err}`]);
          toast.error(`Migration of ${vm.name} failed: ${err}`);
          return;
        }
      }

      setMigrationLog((prev) => [...prev, 'Migration completed successfully.']);
      toast.success(`VM ${vm.name} migrated to ${targetNode}${isCrossCluster ? ` (cluster: ${destCluster})` : ''}`);
      setMigrationVM(null);
      setTargetNode('');
      setTargetCluster('');
      setMigrationLog([]);
      onRefresh?.();
    } catch (error) {
      console.error('Failed to migrate VM:', error);
      setMigrationLog((prev) => [...prev, `ERROR: ${error}`]);
      toast.error(`Failed to migrate VM ${vm.name}: ${error}`);
    } finally {
      setMigrationRunning(false);
    }
  }, [migrationVM, targetNode, targetCluster, targetStorage, targetBridge, onlineMigration, clusterId, onRefresh]);

  const handleClone = useCallback((vm: VMInfo) => {
    if (!clusterId) { toast.error('No cluster selected'); return; }
    const nextVmid = Math.max(...vms.map((v) => v.vmid), 100) + 1;
    setCloneVmid(String(nextVmid));
    setCloneName(`${vm.name}-clone`);
    setCloneDialog({ isOpen: true, vm });
  }, [clusterId, vms]);

  const handleCloneSubmit = useCallback(async () => {
    if (!cloneDialog.vm || !clusterId) return;
    const vm = cloneDialog.vm;
    const newVmid = parseInt(cloneVmid);
    if (isNaN(newVmid) || newVmid < 100) { toast.error('VM ID must be ≥ 100'); return; }
    if (!cloneName.trim()) { toast.error('Clone name is required'); return; }
    setCloneSubmitting(true);
    try {
      await cloneVm({
        clusterId,
        nodeId: vm.node,
        vmId: vm.vmid,
        newVmId: newVmid,
        name: cloneName.trim(),
      });
      toast.success(`VM ${vm.name} cloned to VM ${newVmid}`);
      setCloneDialog({ isOpen: false, vm: null });
      onRefresh?.();
    } catch (error) {
      toast.error(`Failed to clone VM ${vm.name}: ${error}`);
    } finally {
      setCloneSubmitting(false);
    }
  }, [cloneDialog, clusterId, cloneVmid, cloneName, onRefresh]);

  const handleCloneClose = useCallback(() => {
    setCloneDialog({ isOpen: false, vm: null });
  }, []);

  const handleDelete = useCallback(async (vm: VMInfo) => {
    if (!clusterId) {
      toast.error('No cluster selected');
      return;
    }
    const confirmed = await confirm(`Are you sure you want to delete VM ${vm.name} (VMID: ${vm.vmid})? This action cannot be undone!`, {
      title: 'Delete VM',
      kind: 'warning',
    });

    if (!confirmed) {
      return;
    }

    try {
      await deleteVm(clusterId, vm.node, vm.vmid);

      toast.success(`VM ${vm.name} deleted successfully`);
      onRefresh?.();
    } catch (error) {
      console.error('Failed to delete VM:', error);
      toast.error(`Failed to delete VM ${vm.name}: ${error}`);
    }
  }, [clusterId, onRefresh]);

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Virtual Machines</CardTitle>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-[40px]">
                  <Checkbox
                    checked={vms.length > 0 && vms.every((vm) => selectedVMs.has(vm.id))}
                    onCheckedChange={(checked) => {
                      if (checked) {
                        vms.forEach((vm) => selectedVMs.add(vm.id));
                      } else {
                        vms.forEach((vm) => selectedVMs.delete(vm.id));
                      }
                    }}
                  />
                </TableHead>
                <TableHead>Name</TableHead>
                <TableHead>VM ID</TableHead>
                <TableHead>Node</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>CPU</TableHead>
                <TableHead>Memory</TableHead>
                <TableHead>Uptime</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {vms.map((vm) => {
                const cpuPercent = vm.cpu > 0 ? Math.min(vm.cpu * 100, 100) : 0;
                const memoryPercent = vm.memoryTotal > 0 ? (vm.memory / vm.memoryTotal) * 100 : 0;

                return (
                  <TableRow key={vm.id}>
                    <TableCell>
                      <Checkbox
                        checked={selectedVMs.has(vm.id)}
                        onCheckedChange={() => onToggleSelect?.(vm)}
                      />
                    </TableCell>
                    <TableCell className="font-medium">{vm.name}</TableCell>
                    <TableCell>{vm.vmid}</TableCell>
                    <TableCell>{vm.node}</TableCell>
                    <TableCell>
                      <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                        vm.status === 'running' ? 'bg-green-100 text-green-800' :
                        vm.status === 'stopped' ? 'bg-red-100 text-red-800' :
                        'bg-yellow-100 text-yellow-800'
                      }`}>
                        {vm.status}
                      </span>
                    </TableCell>
                    <TableCell>{cpuPercent.toFixed(2)}%</TableCell>
                    <TableCell>
                      {vm.memoryTotal > 0 ? (
                        <div className="flex items-center gap-2">
                          <div className="flex-1 h-2 bg-gray-200 rounded-full overflow-hidden">
                            <div
                              className="h-full bg-blue-500"
                              style={{ width: `${memoryPercent}%` }}
                            />
                          </div>
                          <span className="text-xs text-muted-foreground">
                            {formatBytes(vm.memory)} / {formatBytes(vm.memoryTotal)}
                          </span>
                        </div>
                      ) : (
                        <span className="text-muted-foreground">-</span>
                      )}
                    </TableCell>
                    <TableCell>{formatUptime(vm.uptime || 0)}</TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        {onViewConfig && (
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-8 w-8 p-0"
                            title="View Config"
                            onClick={() => onViewConfig(vm.node, vm.vmid)}
                          >
                            <Settings className="h-4 w-4" />
                          </Button>
                        )}
                        <VMActionMenu
                          vm={vm}
                          onVMAction={handleVMAction}
                          onSnapshotAction={handleSnapshotAction}
                          onMigrate={handleMigrate}
                          onConsole={handleConsole}
                          onClone={handleClone}
                          onDelete={handleDelete}
                        />
                      </div>
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </div>
      </CardContent>

      <MigrationDialog
        vm={migrationVM}
        isOpen={!!migrationVM}
        onClose={() => { if (!migrationRunning) { setMigrationVM(null); setTargetNode(''); setTargetCluster(''); setClusterNodes([]); setMigrationLog([]); } }}
        onSubmit={submitMigration}
        availableNodeNames={clusterNodes}
        nodesLoading={nodesLoading}
        clusters={clusters}
        currentClusterId={clusterId}
        targetNode={targetNode}
        onTargetNodeChange={setTargetNode}
        targetCluster={targetCluster}
        onTargetClusterChange={setTargetCluster}
        targetStorage={targetStorage}
        onTargetStorageChange={setTargetStorage}
        targetBridge={targetBridge}
        onTargetBridgeChange={setTargetBridge}
        onlineMigration={onlineMigration}
        onOnlineMigrationChange={setOnlineMigration}
        maxDowntime={maxDowntime}
        onMaxDowntimeChange={setMaxDowntime}
        migrationRunning={migrationRunning}
        migrationLog={migrationLog}
      />

      <SnapshotDialog
        isOpen={snapshotDialog.isOpen}
        vm={snapshotDialog.vm}
        action={snapshotDialog.action}
        snapshots={snapshotDialog.snapshots}
        snapshotName={snapshotName}
        selectedSnapshot={selectedSnapshot}
        onSnapshotNameChange={setSnapshotName}
        onSelectedSnapshotChange={setSelectedSnapshot}
        onSubmit={handleSnapshotSubmit}
        onClose={handleSnapshotClose}
      />

      <CloneDialog
        isOpen={cloneDialog.isOpen}
        vm={cloneDialog.vm}
        vmid={cloneVmid}
        name={cloneName}
        submitting={cloneSubmitting}
        onVmidChange={setCloneVmid}
        onNameChange={setCloneName}
        onSubmit={() => void handleCloneSubmit()}
        onClose={handleCloneClose}
      />
    </Card>
  );
}

interface VMActionMenuProps {
  vm: VMInfo;
  onVMAction: (vm: VMInfo, action: 'start' | 'stop' | 'reboot' | 'shutdown' | 'resume' | 'suspend') => void;
  onSnapshotAction: (vm: VMInfo, action: 'create' | 'list' | 'rollback' | 'delete') => void;
  onMigrate: (vm: VMInfo) => void;
  onConsole: (vm: VMInfo) => void;
  onClone: (vm: VMInfo) => void;
  onDelete: (vm: VMInfo) => void;
}

function VMActionMenu({
  vm,
  onVMAction,
  onSnapshotAction,
  onMigrate,
  onConsole,
  onClone,
  onDelete,
}: VMActionMenuProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [flipUpward, setFlipUpward] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const menuContentRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen]);

  // After the menu renders, check whether it overflows the viewport bottom and flip if needed.
  // Done in useEffect (not during render) to avoid the react-hooks/refs ESLint violation.
  useEffect(() => {
    if (!isOpen || !menuContentRef.current) return;
    const rect = menuContentRef.current.getBoundingClientRect();
    setFlipUpward(window.innerHeight - rect.bottom < 20);
  }, [isOpen]);

  const toggleMenu = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsOpen(!isOpen);
  };

  const handleAction = (action: () => void) => (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsOpen(false);
    action();
  };

  return (
    <div className="relative" ref={containerRef}>
      <Button
        variant="ghost"
        size="sm"
        onClick={toggleMenu}
        className="h-8 w-8 p-0"
      >
        <MoreHorizontal className="h-4 w-4" />
      </Button>
      {isOpen && (
        <div
          ref={menuContentRef}
          className={clsx(
            'absolute right-0 z-50 w-48 rounded-md border bg-background shadow-md',
            flipUpward ? 'bottom-full mb-2' : 'top-full mt-2',
          )}
        >
          <div className="space-y-1 p-1">
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={handleAction(() => onConsole(vm))}
            >
              <Monitor className="mr-2 h-4 w-4" />
              Console (VNC)
            </button>
            <div className="h-px bg-border my-1" />
            {vm.status === 'stopped' && (
              <button
                className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                onClick={handleAction(() => onVMAction(vm, 'start'))}
              >
                <Play className="mr-2 h-4 w-4" />
                Start
              </button>
            )}
            {vm.status === 'running' && (
              <>
                <button
                  className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                  onClick={handleAction(() => onVMAction(vm, 'stop'))}
                >
                  <Square className="mr-2 h-4 w-4" />
                  Stop
                </button>
                <button
                  className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                  onClick={handleAction(() => onVMAction(vm, 'reboot'))}
                >
                  <RotateCcw className="mr-2 h-4 w-4" />
                  Reboot
                </button>
                <button
                  className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                  onClick={handleAction(() => onVMAction(vm, 'shutdown'))}
                >
                  <Power className="mr-2 h-4 w-4" />
                  Shutdown
                </button>
                <button
                  className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                  onClick={handleAction(() => onVMAction(vm, 'suspend'))}
                >
                  <Pause className="mr-2 h-4 w-4" />
                  Suspend
                </button>
              </>
            )}
            {vm.status === 'paused' && (
              <button
                className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                onClick={handleAction(() => onVMAction(vm, 'resume'))}
              >
                <PlayCircle className="mr-2 h-4 w-4" />
                Resume
              </button>
            )}
            <div className="h-px bg-border my-1" />
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={handleAction(() => onSnapshotAction(vm, 'create'))}
            >
              📸 Create Snapshot
            </button>
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={handleAction(() => onSnapshotAction(vm, 'list'))}
            >
              📋 List Snapshots
            </button>
            <div className="h-px bg-border my-1" />
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={handleAction(() => onMigrate(vm))}
            >
              <MoveRight className="mr-2 h-4 w-4" />
              Migrate
            </button>
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={handleAction(() => onClone(vm))}
            >
              <Copy className="mr-2 h-4 w-4" />
              Clone
            </button>
            <div className="h-px bg-border my-1" />
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-red-100 hover:text-red-600"
              onClick={handleAction(() => onDelete(vm))}
            >
              <X className="mr-2 h-4 w-4" />
              Delete
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

interface MigrationDialogProps {
  vm: VMInfo | null;
  isOpen: boolean;
  onClose: () => void;
  onSubmit: () => void;
  availableNodeNames: string[];
  nodesLoading: boolean;
  clusters: ClusterInfo[];
  currentClusterId: string;
  targetNode: string;
  onTargetNodeChange: (node: string) => void;
  targetCluster: string;
  onTargetClusterChange: (clusterId: string) => void;
  targetStorage: string;
  onTargetStorageChange: (storage: string) => void;
  targetBridge: string;
  onTargetBridgeChange: (bridge: string) => void;
  onlineMigration: boolean;
  onOnlineMigrationChange: (online: boolean) => void;
  maxDowntime: number;
  onMaxDowntimeChange: (downtime: number) => void;
  migrationRunning: boolean;
  migrationLog: string[];
}

function MigrationDialog({
  vm,
  isOpen,
  onClose,
  onSubmit,
  availableNodeNames,
  nodesLoading,
  clusters,
  currentClusterId,
  targetNode,
  onTargetNodeChange,
  targetCluster,
  onTargetClusterChange,
  targetStorage,
  onTargetStorageChange,
  targetBridge,
  onTargetBridgeChange,
  onlineMigration,
  onOnlineMigrationChange,
  maxDowntime,
  onMaxDowntimeChange,
  migrationRunning,
  migrationLog,
}: MigrationDialogProps) {
  if (!vm) return null;

  const isCrossCluster = targetCluster && targetCluster !== currentClusterId;

  const canSubmitMigration = () => {
    if (migrationRunning) return false;
    if (!targetNode) return false;
    if (isCrossCluster) return true;
    return availableNodeNames.length > 0;
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Migrate {vm.name} (VM {vm.vmid})</DialogTitle>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Live migration requires the same hardware configuration on both nodes. Ensure storage is accessible from both nodes.
            </AlertDescription>
          </Alert>

          {clusters.length > 1 && (
            <div className="space-y-2">
              <Label htmlFor="targetCluster">Target Remote</Label>
              <Select value={targetCluster || currentClusterId} onValueChange={onTargetClusterChange}>
                <SelectTrigger>
                  <SelectValue placeholder="Select target cluster" />
                </SelectTrigger>
                <SelectContent>
                  {clusters.map((c) => (
                    <SelectItem key={c.id} value={c.id}>
                      {c.name}{c.id === currentClusterId ? ' (current)' : ''}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {isCrossCluster && (
                <p className="text-xs text-amber-600">
                  Cross-cluster migration — VM will be moved to a different datacenter.
                </p>
              )}
            </div>
          )}

          <div className="space-y-2">
            <Label htmlFor="targetNode">Target Node</Label>
            {nodesLoading ? (
              <p className="text-sm text-muted-foreground animate-pulse">Loading nodes…</p>
            ) : isCrossCluster ? (
              <>
                <Input
                  id="targetNode"
                  value={targetNode}
                  onChange={(e) => onTargetNodeChange(e.target.value)}
                  placeholder="Enter target node name"
                />
                <p className="text-xs text-muted-foreground">
                  Enter the node name on the destination cluster
                </p>
              </>
            ) : (
              <>
                <Select value={targetNode} onValueChange={onTargetNodeChange}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select target node" />
                  </SelectTrigger>
                  <SelectContent>
                    {availableNodeNames.map((node) => (
                      <SelectItem key={node} value={node}>
                        {node}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {availableNodeNames.length === 0 && (
                  <p className="text-xs text-muted-foreground">
                    No other nodes available in this cluster
                  </p>
                )}
              </>
            )}
          </div>

          {isCrossCluster && (
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-2">
                <Label htmlFor="targetStorage">Target Storage</Label>
                <Input
                  id="targetStorage"
                  value={targetStorage}
                  onChange={(e) => onTargetStorageChange(e.target.value)}
                  placeholder="local-lvm"
                  disabled={migrationRunning}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="targetBridge">Target Bridge</Label>
                <Input
                  id="targetBridge"
                  value={targetBridge}
                  onChange={(e) => onTargetBridgeChange(e.target.value)}
                  placeholder="vmbr0"
                  disabled={migrationRunning}
                />
              </div>
              <p className="col-span-2 text-xs text-muted-foreground">
                Storage and network bridge to use on the destination datacenter.
              </p>
            </div>
          )}

          <div className="space-y-2">
            <div className="flex items-center space-x-2">
              <UICheckbox
                id="onlineMigration"
                checked={onlineMigration}
                onCheckedChange={(checked) => onOnlineMigrationChange(checked as boolean)}
              />
              <Label htmlFor="onlineMigration">Live Migration</Label>
            </div>
            <p className="text-xs text-muted-foreground">
              {onlineMigration ? 'Keep VM running during migration' : 'VM will be stopped during migration'}
            </p>
          </div>

          {onlineMigration && (
            <div className="space-y-2">
              <Label htmlFor="maxDowntime">Max Downtime (ms)</Label>
              <Input
                id="maxDowntime"
                type="number"
                value={maxDowntime}
                onChange={(e) => onMaxDowntimeChange(Number(e.target.value))}
                min={10}
                max={10000}
              />
              <p className="text-xs text-muted-foreground">
                Maximum allowed downtime during live migration
              </p>
            </div>
          )}

          {migrationLog.length > 0 && (
            <div className="space-y-2">
              <Label>Migration Progress</Label>
              <div className="max-h-40 overflow-y-auto rounded-md border bg-muted/50 p-2 font-mono text-xs">
                {migrationLog.map((line, idx) => (
                  <div
                    key={idx}
                    className={clsx(line.startsWith('ERROR') && 'text-destructive')}
                  >
                    {line}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={migrationRunning}>
            {migrationRunning ? 'Close' : 'Cancel'}
          </Button>
          <Button
            onClick={onSubmit}
            disabled={!canSubmitMigration()}
          >
            {migrationRunning ? 'Migrating…' : 'Start Migration'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

// ─── Snapshot Dialog ──────────────────────────────────────────────────────────

interface SnapshotDialogProps {
  isOpen: boolean;
  vm: VMInfo | null;
  action: 'create' | 'list' | 'rollback' | 'delete' | null;
  snapshots: ProxmoxSnapshot[];
  snapshotName: string;
  selectedSnapshot: string;
  onSnapshotNameChange: (value: string) => void;
  onSelectedSnapshotChange: (value: string) => void;
  onSubmit: () => void;
  onClose: () => void;
}

function SnapshotDialog({
  isOpen,
  vm,
  action,
  snapshots,
  snapshotName,
  selectedSnapshot,
  onSnapshotNameChange,
  onSelectedSnapshotChange,
  onSubmit,
  onClose,
}: SnapshotDialogProps) {
  if (!vm) return null;

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {action === 'create' && `Create Snapshot for ${vm.name}`}
            {action === 'list' && `Snapshots for ${vm.name}`}
            {action === 'rollback' && `Rollback ${vm.name}`}
            {action === 'delete' && `Delete Snapshot for ${vm.name}`}
          </DialogTitle>
          <DialogDescription>
            {action === 'create' && 'Enter a name for the new snapshot'}
            {action === 'list' && 'View all snapshots for this VM'}
            {action === 'rollback' && 'Select a snapshot to rollback to'}
            {action === 'delete' && 'Select a snapshot to delete'}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {action === 'create' && (
            <div className="space-y-2">
              <Label htmlFor="snapshot-name">Snapshot Name</Label>
              <Input
                id="snapshot-name"
                value={snapshotName}
                onChange={(e) => onSnapshotNameChange(e.target.value)}
                placeholder="e.g., before-upgrade"
              />
            </div>
          )}

          {(action === 'list' || action === 'rollback' || action === 'delete') && (
            <div className="space-y-2">
              <Label>Available Snapshots</Label>
              {snapshots.length === 0 ? (
                <p className="text-sm text-muted-foreground">No snapshots found</p>
              ) : (
                <Select value={selectedSnapshot} onValueChange={onSelectedSnapshotChange}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select a snapshot" />
                  </SelectTrigger>
                  <SelectContent>
                    {snapshots.map((snap) => (
                      <SelectItem key={snap.snapname} value={snap.snapname}>
                        {snap.snapname}
                        {snap.ctime && (
                          <span className="text-xs text-muted-foreground ml-2">
                            ({new Date(snap.ctime * 1000).toLocaleString()})
                          </span>
                        )}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )}

              {action === 'list' && snapshots.length > 0 && (
                <div className="mt-4 space-y-2">
                  <Label>Snapshot Details</Label>
                  {snapshots.map((snap) => (
                    <div key={snap.snapname} className="p-3 border rounded-lg">
                      <div className="font-medium">{snap.snapname}</div>
                      <div className="text-sm text-muted-foreground">
                        Created: {new Date(snap.ctime * 1000).toLocaleString()}
                        {snap.description && <div>Description: {snap.description}</div>}
                        {snap.parent && <div>Parent: {snap.parent}</div>}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={onSubmit}>
            {action === 'create' && 'Create Snapshot'}
            {action === 'list' && 'Close'}
            {action === 'rollback' && 'Rollback'}
            {action === 'delete' && 'Delete Snapshot'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

// ─── Clone Dialog ─────────────────────────────────────────────────────────────

interface CloneDialogProps {
  isOpen: boolean;
  vm: VMInfo | null;
  vmid: string;
  name: string;
  submitting: boolean;
  onVmidChange: (v: string) => void;
  onNameChange: (v: string) => void;
  onSubmit: () => void;
  onClose: () => void;
}

function CloneDialog({ isOpen, vm, vmid, name, submitting, onVmidChange, onNameChange, onSubmit, onClose }: CloneDialogProps) {
  if (!vm) return null;
  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-sm">
        <DialogHeader>
          <DialogTitle>Clone {vm.name} (VM {vm.vmid})</DialogTitle>
          <DialogDescription>Enter details for the cloned VM.</DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-2">
          <div className="space-y-1">
            <Label htmlFor="clone-vmid">New VM ID</Label>
            <Input
              id="clone-vmid"
              type="number"
              min={100}
              max={999999999}
              value={vmid}
              onChange={(e) => onVmidChange(e.target.value)}
              disabled={submitting}
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="clone-name">New VM Name</Label>
            <Input
              id="clone-name"
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              placeholder={`${vm.name}-clone`}
              disabled={submitting}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={submitting}>Cancel</Button>
          <Button onClick={onSubmit} disabled={submitting || !name.trim()}>
            {submitting ? 'Cloning…' : 'Clone VM'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
