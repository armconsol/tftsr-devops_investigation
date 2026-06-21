import React, { useState, useEffect, useCallback, useRef } from 'react';
import { clsx } from 'clsx';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Checkbox } from '@/components/ui/index';
import { MoreHorizontal, Play, Square, RotateCcw, Power, PlayCircle, Pause, X, MoveRight, Copy } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { confirm } from '@tauri-apps/plugin-dialog';
import { toast } from 'sonner';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Checkbox as UICheckbox } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { AlertCircle } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/index';
import type { ClusterInfo } from '@/lib/domain';

interface VMInfo {
  id: string;
  vmid: number;
  name: string;
  node: string;
  status: 'running' | 'stopped' | 'paused';
  cpu: number;
  memory: number;
  memoryTotal: number;
  disk: number;
  diskTotal: number;
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
}: VMListProps) {
  const [migrationVM, setMigrationVM] = useState<VMInfo | null>(null);
  const [targetNode, setTargetNode] = useState<string>('');
  const [targetCluster, setTargetCluster] = useState<string>('');
  const [onlineMigration, setOnlineMigration] = useState(true);
  const [maxDowntime, setMaxDowntime] = useState(30);

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
      disk: vm.disk || 0,
      diskTotal: vm.max_disk || vm.diskTotal || 0,
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
          await invoke('start_proxmox_vm', {
            clusterId,
            nodeId: vm.node,
            vmId: vm.vmid,
          });
          toast.success(`VM ${vm.name} started`);
          onRefresh?.();
          break;
        case 'stop':
          await invoke('stop_proxmox_vm', {
            clusterId,
            nodeId: vm.node,
            vmId: vm.vmid,
          });
          toast.success(`VM ${vm.name} stopped`);
          onRefresh?.();
          break;
        case 'reboot':
          await invoke('reboot_proxmox_vm', {
            clusterId,
            nodeId: vm.node,
            vmId: vm.vmid,
          });
          toast.success(`VM ${vm.name} rebooting`);
          onRefresh?.();
          break;
        case 'shutdown':
          await invoke('shutdown_proxmox_vm', {
            clusterId,
            nodeId: vm.node,
            vmId: vm.vmid,
          });
          toast.success(`VM ${vm.name} shutting down`);
          onRefresh?.();
          break;
        case 'resume':
          await invoke('resume_proxmox_vm', {
            clusterId,
            nodeId: vm.node,
            vmId: vm.vmid,
          });
          toast.success(`VM ${vm.name} resumed`);
          onRefresh?.();
          break;
        case 'suspend':
          await invoke('suspend_proxmox_vm', {
            clusterId,
            nodeId: vm.node,
            vmId: vm.vmid,
          });
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

  const handleSnapshotAction = useCallback((vm: VMInfo, action: 'create' | 'list' | 'rollback' | 'delete') => {
    toast.info(`Snapshot ${action} for ${vm.name} - not yet implemented`);
  }, []);

  const handleMigrate = useCallback((vm: VMInfo) => {
    setMigrationVM(vm);
    const availableNodes = vms
      .map((v) => v.node)
      .filter((node, index, self) => self.indexOf(node) === index && node !== vm.node);
    setTargetNode(availableNodes[0] || '');
    setTargetCluster(clusterId);
  }, [vms, clusterId]);

  const submitMigration = useCallback(async () => {
    if (!migrationVM || !targetNode) {
      toast.error('Please select a target node');
      return;
    }

    const sourceCluster = clusterId;
    const destCluster = targetCluster || clusterId;

    try {
      await invoke('migrate_vm', {
        clusterId: sourceCluster,
        nodeId: migrationVM.node,
        vmId: migrationVM.vmid,
        targetNode,
        targetCluster: destCluster,
      });

      toast.success(`VM ${migrationVM.name} migration started to ${targetNode}${destCluster !== sourceCluster ? ` (cluster: ${destCluster})` : ''}`);
      setMigrationVM(null);
      setTargetNode('');
      setTargetCluster('');
      onRefresh?.();
    } catch (error) {
      console.error('Failed to migrate VM:', error);
      toast.error(`Failed to migrate VM ${migrationVM.name}: ${error}`);
    }
  }, [migrationVM, targetNode, targetCluster, clusterId, onRefresh]);

  const handleClone = useCallback(async (vm: VMInfo) => {
    if (!clusterId) {
      toast.error('No cluster selected');
      return;
    }
    try {
      const nextVmid = Math.max(...vms.map((v) => v.vmid), 100) + 1;
      const newVmidStr = window.prompt(`Enter new VM ID for ${vm.name}:`, `${nextVmid}`);
      if (!newVmidStr) {
        toast.info('Clone cancelled');
        return;
      }
      const newVmid = parseInt(newVmidStr);
      if (isNaN(newVmid) || newVmid < 100) {
        toast.error('Invalid VM ID. Must be >= 100');
        return;
      }
      const newName = window.prompt(`Enter name for cloned VM:`, `${vm.name}-clone`);
      if (!newName) {
        toast.info('Clone cancelled');
        return;
      }

      await invoke('clone_vm', {
        clusterId,
        nodeId: vm.node,
        vmId: vm.vmid,
        newVmid,
        name: newName,
      });

      toast.success(`VM ${vm.name} cloned successfully to VM ${newVmid}`);
      onRefresh?.();
    } catch (error) {
      console.error('Failed to clone VM:', error);
      toast.error(`Failed to clone VM ${vm.name}: ${error}`);
    }
  }, [clusterId, vms, onRefresh]);

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
      await invoke('delete_vm', {
        clusterId,
        nodeId: vm.node,
        vmId: vm.vmid,
      });

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
                <TableHead>Disk</TableHead>
                <TableHead>Uptime</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {vms.map((vm) => {
                const cpuPercent = vm.cpu > 0 ? Math.min(vm.cpu * 100, 100) : 0;
                const memoryPercent = vm.memoryTotal > 0 ? (vm.memory / vm.memoryTotal) * 100 : 0;
                const diskPercent = vm.diskTotal > 0 ? (vm.disk / vm.diskTotal) * 100 : 0;

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
                    <TableCell>
                      {vm.diskTotal > 0 ? (
                        <div className="flex items-center gap-2">
                          <div className="flex-1 h-2 bg-gray-200 rounded-full overflow-hidden">
                            <div
                              className="h-full bg-green-500"
                              style={{ width: `${diskPercent}%` }}
                            />
                          </div>
                          <span className="text-xs text-muted-foreground">
                            {formatBytes(vm.disk)} / {formatBytes(vm.diskTotal)}
                          </span>
                        </div>
                      ) : (
                        <span className="text-muted-foreground">-</span>
                      )}
                    </TableCell>
                    <TableCell>{formatUptime(vm.uptime || 0)}</TableCell>
                    <TableCell className="text-right">
                      <VMActionMenu
                        vm={vm}
                        onVMAction={handleVMAction}
                        onSnapshotAction={handleSnapshotAction}
                        onMigrate={handleMigrate}
                        onClone={handleClone}
                        onDelete={handleDelete}
                      />
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
        onClose={() => { setMigrationVM(null); setTargetNode(''); setTargetCluster(''); }}
        onSubmit={submitMigration}
        availableNodes={vms}
        clusters={clusters}
        currentClusterId={clusterId}
        targetNode={targetNode}
        onTargetNodeChange={setTargetNode}
        targetCluster={targetCluster}
        onTargetClusterChange={setTargetCluster}
        online={onlineMigration}
        onOnlineChange={setOnlineMigration}
        maxDowntime={maxDowntime}
        onMaxDowntimeChange={setMaxDowntime}
      />
    </Card>
  );
}

interface VMActionMenuProps {
  vm: VMInfo;
  onVMAction: (vm: VMInfo, action: 'start' | 'stop' | 'reboot' | 'shutdown' | 'resume' | 'suspend') => void;
  onSnapshotAction: (vm: VMInfo, action: 'create' | 'list' | 'rollback' | 'delete') => void;
  onMigrate: (vm: VMInfo) => void;
  onClone: (vm: VMInfo) => void;
  onDelete: (vm: VMInfo) => void;
}

function VMActionMenu({
  vm,
  onVMAction,
  onSnapshotAction,
  onMigrate,
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
  availableNodes: VMInfo[];
  clusters: ClusterInfo[];
  currentClusterId: string;
  targetNode: string;
  onTargetNodeChange: (node: string) => void;
  targetCluster: string;
  onTargetClusterChange: (clusterId: string) => void;
  online: boolean;
  onOnlineChange: (online: boolean) => void;
  maxDowntime: number;
  onMaxDowntimeChange: (downtime: number) => void;
}

function MigrationDialog({
  vm,
  isOpen,
  onClose,
  onSubmit,
  availableNodes,
  clusters,
  currentClusterId,
  targetNode,
  onTargetNodeChange,
  targetCluster,
  onTargetClusterChange,
  online,
  onOnlineChange,
  maxDowntime,
  onMaxDowntimeChange,
}: MigrationDialogProps) {
  if (!vm) return null;

  const isCrossCluster = targetCluster && targetCluster !== currentClusterId;

  const availableTargets = availableNodes
    .map((v) => v.node)
    .filter((node, index, self) => self.indexOf(node) === index && node !== vm.node);

  const canSubmitMigration = () => {
    if (!targetNode) return false;
    if (isCrossCluster) return true;
    return availableTargets.length > 0;
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
            {isCrossCluster ? (
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
                    {availableTargets.map((node) => (
                      <SelectItem key={node} value={node}>
                        {node}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {availableTargets.length === 0 && (
                  <p className="text-xs text-muted-foreground">
                    No other nodes available in this cluster
                  </p>
                )}
              </>
            )}
          </div>

          <div className="space-y-2">
            <div className="flex items-center space-x-2">
              <UICheckbox
                id="online"
                checked={online}
                onCheckedChange={(checked) => onOnlineChange(checked as boolean)}
              />
              <Label htmlFor="online">Live Migration</Label>
            </div>
            <p className="text-xs text-muted-foreground">
              {online ? 'Keep VM running during migration' : 'VM will be stopped during migration'}
            </p>
          </div>

          {online && (
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
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button
            onClick={onSubmit}
            disabled={!canSubmitMigration()}
          >
            Start Migration
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
