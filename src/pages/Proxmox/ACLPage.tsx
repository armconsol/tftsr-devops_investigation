import React from 'react';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { AclList } from '@/components/Proxmox';

export function ProxmoxACLPage() {
  const acls = [
    { id: '1', path: '/nodes/pve1', type: 'user' as const, principal: 'admin@pam', roles: ['PVEAdmin'], propagate: true },
    { id: '2', path: '/storage/local', type: 'group' as const, principal: 'admins', roles: ['PVEDataStoreAdmin'], propagate: false },
    { id: '3', path: '/vms/100', type: 'user' as const, principal: 'developer@pam', roles: ['PVEVMUser'], propagate: false },
  ];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Access Control Lists</h1>
          <p className="text-muted-foreground">Manage permissions and access control</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm">
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      <AclList
        acls={acls}
        onRefresh={() => {}}
      />
    </div>
  );
}
