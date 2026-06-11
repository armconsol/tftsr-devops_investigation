import React from 'react';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';

export function ProxmoxSDNPage() {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">SDN</h1>
          <p className="text-muted-foreground">Software Defined Networking</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm">
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      <div className="text-sm text-muted-foreground">
        SDN Zone management coming soon
      </div>
    </div>
  );
}
