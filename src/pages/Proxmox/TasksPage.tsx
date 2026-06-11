import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { ClusterOperationsList } from '@/components/Proxmox';

export function ProxmoxTasksPage() {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Tasks & Operations</h1>
          <p className="text-muted-foreground">Monitor cluster operations and tasks</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm">
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Task Summary</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-sm text-muted-foreground">Task summary widget coming soon</div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Cluster Operations</CardTitle>
        </CardHeader>
        <CardContent>
          <ClusterOperationsList
            operations={[]}
            onRefresh={() => {}}
          />
        </CardContent>
      </Card>
    </div>
  );
}
