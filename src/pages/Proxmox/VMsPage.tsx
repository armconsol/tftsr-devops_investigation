import React from 'react';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { VMList } from '@/components/Proxmox';

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
}

export function ProxmoxVMsPage() {
  const vms: VMInfo[] = [
    { id: '1', name: 'web-server-01', vmid: 100, node: 'pve1', status: 'running', cpu: 4, memory: 8192, memoryTotal: 8192, disk: 100, diskTotal: 100, uptime: '2d 4h' },
    { id: '2', name: 'db-server-01', vmid: 101, node: 'pve2', status: 'running', cpu: 8, memory: 16384, memoryTotal: 16384, disk: 500, diskTotal: 500, uptime: '5d 12h' },
    { id: '3', name: 'dev-vm', vmid: 102, node: 'pve1', status: 'stopped', cpu: 2, memory: 4096, memoryTotal: 4096, disk: 50, diskTotal: 50 },
  ];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Virtual Machines</h1>
          <p className="text-muted-foreground">Manage QEMU/KVM virtual machines</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm">
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      <VMList
        vms={vms}
        onRefresh={() => {}}
        onVMAction={(_vm, _action) => {
          // VM action handler
        }}
        onSnapshotAction={(_vm, _action) => {
          // Snapshot action handler
        }}
        onMigrate={(_vm) => {
          // Migrate handler
        }}
        onClone={(_vm) => {
          // Clone handler
        }}
        onDelete={(_vm) => {
          // Delete handler
        }}
        selectedVMs={new Set()}
        onToggleSelect={(_vm) => {
          // VM select handler
        }}
      />
    </div>
  );
}
