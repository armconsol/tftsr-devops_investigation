import React, { useState, useEffect, useCallback, useRef } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Checkbox } from '@/components/ui/index';
import { MoreHorizontal, Play, Square, RotateCcw, Power, PlayCircle, Pause, X, MoveRight, Copy } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'sonner';

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

interface VMListProps {
  vms: VMInfo[];
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
  vms,
  onRefresh,
  isLoading,
  onSnapshotAction,
  onMigrate,
  onClone,
  onDelete,
  selectedVMs = new Set<string>(),
  onToggleSelect,
}: VMListProps) {
  const [clusterId, setClusterId] = useState<string>('');

  useEffect(() => {
    invoke<string>('get_current_proxmox_cluster').catch(() => {
      // Fallback: try to get first cluster
      invoke<string[]>('list_proxmox_clusters')
        .then((clusters: any[]) => {
          if (clusters.length > 0) {
            setClusterId(clusters[0].id);
          }
        })
        .catch(() => {});
    })
    .then((id) => {
      if (id) setClusterId(id);
    });
  }, []);

  const handleVMAction = useCallback(async (vm: VMInfo, action: string) => {
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

  const handleMigrate = useCallback(async (vm: VMInfo) => {
    try {
      const targetNodes = vms
        .map((v) => v.node)
        .filter((node, index, self) => self.indexOf(node) === index && node !== vm.node);
      
      if (targetNodes.length === 0) {
        toast.error('No target nodes available for migration');
        return;
      }

      const targetNode = await prompt(`Select target node: ${targetNodes.join(', ')}`, targetNodes[0]);
      
      await invoke('migrate_vm', {
        clusterId,
        nodeId: vm.node,
        vmId: vm.vmid,
        targetNode,
        online: vm.status === 'running',
      });
      
      toast.success(`VM ${vm.name} migration started`);
      onRefresh?.();
    } catch (error) {
      console.error('Failed to migrate VM:', error);
      toast.error(`Failed to migrate VM ${vm.name}: ${error}`);
    }
  }, [clusterId, vms, onRefresh]);

  const handleClone = useCallback(async (vm: VMInfo) => {
    try {
      const newVmidStr = await prompt(`Enter new VM ID for ${vm.name}:`, `${vm.vmid + 1}`);
      const newVmid = newVmidStr ? parseInt(newVmidStr) : vm.vmid + 1;
      const newName = await prompt(`Enter name for cloned VM:`, `${vm.name}-clone`);
      
      await invoke('clone_vm', {
        clusterId,
        nodeId: vm.node,
        vmId: vm.vmid,
        newVmid,
        name: newName || `${vm.name}-clone`,
      });
      
      toast.success(`VM ${vm.name} cloned successfully`);
      onRefresh?.();
    } catch (error) {
      console.error('Failed to clone VM:', error);
      toast.error(`Failed to clone VM ${vm.name}: ${error}`);
    }
  }, [clusterId, onRefresh]);

  const handleDelete = useCallback(async (vm: VMInfo) => {
    if (!confirm(`Are you sure you want to delete VM ${vm.name}?`)) {
      return;
    }

    try {
      await invoke('delete_vm', {
        clusterId,
        nodeId: vm.node,
        vmId: vm.vmid,
      });
      
      toast.success(`VM ${vm.name} deleted`);
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
                    checked={vms.every((vm) => selectedVMs.has(vm.id))}
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
  const menuRef = useRef<HTMLDivElement>(null);

  // Close menu when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
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

  const toggleMenu = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsOpen(!isOpen);
  };

  const handleAction = (action: () => void) => (e: React.MouseEvent) => {
    e.stopPropagation();
    action();
    setIsOpen(false);
  };

  // Calculate menu position to avoid overflow
  const getMenuPosition = () => {
    const viewportHeight = window.innerHeight;
    const buttonRect = menuRef.current?.querySelector('button')?.getBoundingClientRect();
    
    if (!buttonRect) return { top: '100%', left: 0 };

    const menuHeight = 400; // approximate menu height
    const spaceBelow = viewportHeight - buttonRect.bottom;
    const spaceAbove = buttonRect.top;

    if (spaceBelow >= menuHeight) {
      return { top: '100%', left: 0 };
    } else if (spaceAbove >= menuHeight) {
      return { bottom: '100%', left: 0 };
    } else {
      // Menu will fit somewhere in the middle
      return { top: '100%', left: 0 };
    }
  };

  const position = getMenuPosition();

  return (
    <div className="relative" ref={menuRef}>
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
          className={`absolute z-50 w-48 rounded-md border bg-background shadow-md ${
            position.bottom ? 'bottom-full mb-2' : 'top-full mt-2'
          }`}
          style={{ right: 0 }}
        >
          <div className="space-y-1 p-1">
            {vm.status === 'stopped' && (
              <button
                className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                onClick={() => onVMAction(vm, 'start')}
              >
                <Play className="mr-2 h-4 w-4" />
                Start
              </button>
            )}
            {vm.status === 'running' && (
              <>
                <button
                  className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                  onClick={() => onVMAction(vm, 'stop')}
                >
                  <Square className="mr-2 h-4 w-4" />
                  Stop
                </button>
                <button
                  className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                  onClick={() => onVMAction(vm, 'reboot')}
                >
                  <RotateCcw className="mr-2 h-4 w-4" />
                  Reboot
                </button>
                <button
                  className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                  onClick={() => onVMAction(vm, 'shutdown')}
                >
                  <Power className="mr-2 h-4 w-4" />
                  Shutdown
                </button>
                <button
                  className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                  onClick={() => onVMAction(vm, 'suspend')}
                >
                  <Pause className="mr-2 h-4 w-4" />
                  Suspend
                </button>
              </>
            )}
            {vm.status === 'paused' && (
              <button
                className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                onClick={() => onVMAction(vm, 'resume')}
              >
                <PlayCircle className="mr-2 h-4 w-4" />
                Resume
              </button>
            )}
            <div className="h-px bg-border my-1" />
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={() => onSnapshotAction(vm, 'create')}
            >
              📸 Create Snapshot
            </button>
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={() => onSnapshotAction(vm, 'list')}
            >
              📋 List Snapshots
            </button>
            <div className="h-px bg-border my-1" />
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={() => onMigrate(vm)}
            >
              <MoveRight className="mr-2 h-4 w-4" />
              Migrate
            </button>
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={() => onClone(vm)}
            >
              <Copy className="mr-2 h-4 w-4" />
              Clone
            </button>
            <div className="h-px bg-border my-1" />
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-red-100 hover:text-red-600"
              onClick={() => onDelete(vm)}
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
