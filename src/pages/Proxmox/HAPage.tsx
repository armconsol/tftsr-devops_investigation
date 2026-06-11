import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { HAGroupsList, HAResourcesList } from '@/components/Proxmox';

export function ProxmoxHAPage() {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">High Availability</h1>
          <p className="text-muted-foreground">Manage HA groups and resources</p>
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
            <CardTitle>HA Groups</CardTitle>
          </CardHeader>
          <CardContent>
            <HAGroupsList
              groups={[]}
              onRefresh={() => {}}
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>HA Resources</CardTitle>
          </CardHeader>
          <CardContent>
            <HAResourcesList
              resources={[]}
              onRefresh={() => {}}
            />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
