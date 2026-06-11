import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { PoolList, OSDList, CephHealthWidget, MonitorList } from '@/components/Proxmox';

export function ProxmoxCephPage() {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Ceph Storage</h1>
          <p className="text-muted-foreground">Manage Ceph clusters and storage</p>
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
            <CardTitle>Ceph Health</CardTitle>
          </CardHeader>
          <CardContent>
            <CephHealthWidget
              health={{ status: 'HEALTH_OK', summary: 'Cluster healthy', details: [] }}
            />
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Pools</CardTitle>
          </CardHeader>
          <CardContent>
            <PoolList
              pools={[]}
              onRefresh={() => {}}
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>OSDs</CardTitle>
          </CardHeader>
          <CardContent>
            <OSDList
              osds={[]}
              onRefresh={() => {}}
            />
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Monitors</CardTitle>
        </CardHeader>
        <CardContent>
          <MonitorList
            monitors={[]}
            onRefresh={() => {}}
          />
        </CardContent>
      </Card>
    </div>
  );
}
