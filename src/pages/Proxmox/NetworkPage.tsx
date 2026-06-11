import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';

export function ProxmoxNetworkPage() {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Network</h1>
          <p className="text-muted-foreground">Configure network interfaces and bridges</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm">
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Network Interfaces</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-sm text-muted-foreground">Network interface configuration coming soon</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Bridges</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-sm text-muted-foreground">Bridge configuration coming soon</div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
