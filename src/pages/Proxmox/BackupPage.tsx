import React from 'react';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { BackupJobList } from '@/components/Proxmox';

export function ProxmoxBackupPage() {
  const jobs = [
    { id: '1', name: 'Daily VM Backup', node: 'pve1', schedule: '0 2 * * *', status: 'idle' as const, enabled: true },
    { id: '2', name: 'Weekly PBS Backup', node: 'pbs1', schedule: '0 3 * * 0', status: 'success' as const, lastRun: '2024-01-01', enabled: true },
  ];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Backup Jobs</h1>
          <p className="text-muted-foreground">Manage Proxmox Backup Server jobs</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm">
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      <BackupJobList
        jobs={jobs}
        onRefresh={() => {}}
      />
    </div>
  );
}
