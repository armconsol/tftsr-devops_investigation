import React from 'react';
// Card imports removed '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { CertificateList } from '@/components/Proxmox';

export function ProxmoxCertificatesPage() {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Certificates</h1>
          <p className="text-muted-foreground">Manage TLS certificates</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm">
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      <CertificateList
        certificates={[]}
        onRefresh={() => {}}
      />
    </div>
  );
}
