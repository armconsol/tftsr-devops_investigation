import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Checkbox } from '@/components/ui/index';
import { MoreHorizontal, Play, Square, RotateCcw, Power, PlayCircle, Pause, X } from 'lucide-react';

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
  uptime?: string;
  tags?: string[];
}

interface VMListProps {
  vms: VMInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onVMAction?: (vm: VMInfo, action: 'start' | 'stop' | 'reboot' | 'shutdown' | 'resume' | 'suspend') => void;
  onSnapshotAction?: (vm: VMInfo, action: 'create' | 'list' | 'rollback' | 'delete') => void;
  onMigrate?: (vm: VMInfo) => void;
  onClone?: (vm: VMInfo) => void;
  onDelete?: (vm: VMInfo) => void;
  selectedVMs?: Set<string>;
  onToggleSelect?: (vm: VMInfo) => void;
}

export function VMList({
  vms,
  onRefresh,
  isLoading,
  onVMAction,
  onSnapshotAction,
  onMigrate,
  onClone,
  onDelete,
  selectedVMs = new Set<string>(),
  onToggleSelect,
}: VMListProps) {
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
              {vms.map((vm) => (
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
                  <TableCell>{vm.cpu}%</TableCell>
                  <TableCell>{Math.round((vm.memory / vm.memoryTotal) * 100)}%</TableCell>
                  <TableCell>{Math.round((vm.disk / vm.diskTotal) * 100)}%</TableCell>
                  <TableCell>{vm.uptime || '-'}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <VMActionMenu
                        vm={vm}
                        onVMAction={onVMAction}
                        onSnapshotAction={onSnapshotAction}
                        onMigrate={onMigrate}
                        onClone={onClone}
                        onDelete={onDelete}
                      />
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  );
}

interface VMActionMenuProps {
  vm: VMInfo;
  onVMAction?: (vm: VMInfo, action: 'start' | 'stop' | 'reboot' | 'shutdown' | 'resume' | 'suspend') => void;
  onSnapshotAction?: (vm: VMInfo, action: 'create' | 'list' | 'rollback' | 'delete') => void;
  onMigrate?: (vm: VMInfo) => void;
  onClone?: (vm: VMInfo) => void;
  onDelete?: (vm: VMInfo) => void;
}

function VMActionMenu({
  vm,
  onVMAction,
  onSnapshotAction,
  onMigrate,
  onClone,
  onDelete,
}: VMActionMenuProps) {
  const [isOpen, setIsOpen] = React.useState(false);

  return (
    <div className="relative">
      <Button
        variant="ghost"
        size="sm"
        onClick={() => setIsOpen(!isOpen)}
      >
        <MoreHorizontal className="h-4 w-4" />
      </Button>
      {isOpen && (
        <div className="absolute right-0 top-8 z-50 w-48 rounded-md border bg-popover p-2 shadow-md">
          <div className="space-y-1">
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={() => {
                onVMAction?.(vm, 'start');
                setIsOpen(false);
              }}
            >
              <Play className="mr-2 h-4 w-4" />
              Start
            </button>
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={() => {
                onVMAction?.(vm, 'stop');
                setIsOpen(false);
              }}
            >
              <Square className="mr-2 h-4 w-4" />
              Stop
            </button>
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={() => {
                onVMAction?.(vm, 'reboot');
                setIsOpen(false);
              }}
            >
              <RotateCcw className="mr-2 h-4 w-4" />
              Reboot
            </button>
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={() => {
                onVMAction?.(vm, 'shutdown');
                setIsOpen(false);
              }}
            >
              <Power className="mr-2 h-4 w-4" />
              Shutdown
            </button>
            {vm.status === 'paused' && (
              <button
                className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                onClick={() => {
                  onVMAction?.(vm, 'resume');
                  setIsOpen(false);
                }}
              >
                <PlayCircle className="mr-2 h-4 w-4" />
                Resume
              </button>
            )}
            {vm.status === 'running' && (
              <button
                className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                onClick={() => {
                  onVMAction?.(vm, 'suspend');
                  setIsOpen(false);
                }}
              >
                <Pause className="mr-2 h-4 w-4" />
                Suspend
              </button>
            )}
            <div className="h-px bg-border my-1" />
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={() => {
                onSnapshotAction?.(vm, 'create');
                setIsOpen(false);
              }}
            >
              <span className="mr-2 h-4 w-4">📸</span>
              Create Snapshot
            </button>
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={() => {
                onSnapshotAction?.(vm, 'list');
                setIsOpen(false);
              }}
            >
              <span className="mr-2 h-4 w-4">📋</span>
              List Snapshots
            </button>
            <div className="h-px bg-border my-1" />
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={() => {
                onMigrate?.(vm);
                setIsOpen(false);
              }}
            >
              <span className="mr-2 h-4 w-4">🚚</span>
              Migrate
            </button>
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-accent"
              onClick={() => {
                onClone?.(vm);
                setIsOpen(false);
              }}
            >
              <span className="mr-2 h-4 w-4">📋</span>
              Clone
            </button>
            <div className="h-px bg-border my-1" />
            <button
              className="flex w-full items-center rounded-md px-2 py-1.5 text-sm hover:bg-red-100 hover:text-red-600"
              onClick={() => {
                onDelete?.(vm);
                setIsOpen(false);
              }}
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
