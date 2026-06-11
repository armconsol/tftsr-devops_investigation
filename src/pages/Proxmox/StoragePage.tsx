import React from 'react';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { StorageList } from '@/components/Proxmox';

export function ProxmoxStoragePage() {
  const storages = [
    { id: '1', name: 'local', type: 'dir', remote: 'local', node: 'pve1', used: '50 GB', total: '500 GB', available: '450 GB', status: 'active' },
    { id: '2', name: 'local-lvm', type: 'lvm', remote: 'local', node: 'pve1', used: '100 GB', total: '1000 GB', available: '900 GB', status: 'active' },
    { id: '3', name: 'nfs-backup', type: 'nfs', remote: 'nfs', node: 'pve2', used: '200 GB', total: '2000 GB', available: '1800 GB', status: 'active' },
  ];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Storage</h1>
          <p className="text-muted-foreground">Manage storage pools and volumes</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm">
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      <StorageList
        storages={storages}
        onRefresh={() => {}}
      />
    </div>
  );
}
