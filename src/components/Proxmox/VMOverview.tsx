import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';

interface VMInfo {
  id: string;
  name: string;
  vmid: number;
  node: string;
  status: string;
  cpu: number;
  memory: number;
  disk: number;
  uptime?: string;
}

interface VMOverviewProps {
  vm: VMInfo;
  onRefresh?: () => void;
  isLoading?: boolean;
  onPowerAction?: (action: 'start' | 'stop' | 'reboot' | 'shutdown' | 'resume' | 'suspend') => void;
  onConsole?: () => void;
}

export function VMOverview({ vm, onRefresh, isLoading, onPowerAction, onConsole }: VMOverviewProps) {
  const statusColors = {
    running: 'bg-green-100 text-green-800',
    stopped: 'bg-gray-100 text-gray-800',
    suspended: 'bg-yellow-100 text-yellow-800',
    paused: 'bg-orange-100 text-orange-800',
    error: 'bg-red-100 text-red-800',
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">{vm.name}</h1>
          <p className="text-muted-foreground">VM ID: {vm.vmid} • Node: {vm.node}</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          <Button size="sm" onClick={onConsole}>
            Console
          </Button>
          {vm.status === 'running' && (
            <>
              <Button variant="outline" size="sm" onClick={() => onPowerAction?.('stop')}>
                Stop
              </Button>
              <Button variant="outline" size="sm" onClick={() => onPowerAction?.('reboot')}>
                Reboot
              </Button>
              <Button variant="outline" size="sm" onClick={() => onPowerAction?.('shutdown')}>
                Shutdown
              </Button>
              <Button variant="outline" size="sm" onClick={() => onPowerAction?.('suspend')}>
                Suspend
              </Button>
            </>
          )}
          {vm.status === 'stopped' && (
            <Button size="sm" onClick={() => onPowerAction?.('start')}>
              Start
            </Button>
          )}
          {vm.status === 'suspended' && (
            <Button size="sm" onClick={() => onPowerAction?.('resume')}>
              Resume
            </Button>
          )}
        </div>
      </div>

      <Tabs value="overview" onValueChange={() => {}}>
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="configuration">Configuration</TabsTrigger>
          <TabsTrigger value="hardware">Hardware</TabsTrigger>
          <TabsTrigger value="snapshots">Snapshots</TabsTrigger>
          <TabsTrigger value="metrics">Metrics</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm">Status</CardTitle>
              </CardHeader>
              <CardContent>
                <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${statusColors[vm.status as keyof typeof statusColors] || statusColors.stopped}`}>
                  {vm.status}
                </span>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm">CPU Cores</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{vm.cpu}</div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm">Memory</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{vm.memory} MB</div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm">Disk</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{vm.disk} GB</div>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Quick Actions</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2">
                <Button variant="outline" size="sm" onClick={() => onPowerAction?.('start')}>Start</Button>
                <Button variant="outline" size="sm" onClick={() => onPowerAction?.('stop')}>Stop</Button>
                <Button variant="outline" size="sm" onClick={() => onPowerAction?.('reboot')}>Reboot</Button>
                <Button variant="outline" size="sm" onClick={() => onPowerAction?.('shutdown')}>Shutdown</Button>
                <Button variant="outline" size="sm" onClick={() => onPowerAction?.('suspend')}>Suspend</Button>
                <Button variant="outline" size="sm" onClick={() => onPowerAction?.('resume')}>Resume</Button>
                <Button variant="outline" size="sm">Clone</Button>
                <Button variant="outline" size="sm">Migrate</Button>
                <Button variant="outline" size="sm">Snapshot</Button>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="configuration">
          <Card>
            <CardHeader>
              <CardTitle>Configuration</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-muted-foreground">VM ID</div>
                    <div className="font-medium">{vm.vmid}</div>
                  </div>
                  <div>
                    <div className="text-sm text-muted-foreground">Node</div>
                    <div className="font-medium">{vm.node}</div>
                  </div>
                  <div>
                    <div className="text-sm text-muted-foreground">Status</div>
                    <div className="font-medium">{vm.status}</div>
                  </div>
                  <div>
                    <div className="text-sm text-muted-foreground">Uptime</div>
                    <div className="font-medium">{vm.uptime || 'N/A'}</div>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="hardware">
          <Card>
            <CardHeader>
              <CardTitle>Hardware Configuration</CardTitle>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Device</TableHead>
                    <TableHead>Type</TableHead>
                    <TableHead>Size</TableHead>
                    <TableHead>Status</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell className="font-medium">Disk 0</TableCell>
                    <TableCell>virtio</TableCell>
                    <TableCell>{vm.disk} GB</TableCell>
                    <TableCell>connected</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className="font-medium">Network 0</TableCell>
                    <TableCell>virtio</TableCell>
                    <TableCell>-</TableCell>
                    <TableCell>connected</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className="font-medium">CPU</TableCell>
                    <TableCell>host</TableCell>
                    <TableCell>{vm.cpu} cores</TableCell>
                    <TableCell>active</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className="font-medium">Memory</TableCell>
                    <TableCell>size</TableCell>
                    <TableCell>{vm.memory} MB</TableCell>
                    <TableCell>active</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="snapshots">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>Snapshots</CardTitle>
              <Button size="sm">
                <span className="mr-2 h-4 w-4">+</span>
                Create
              </Button>
            </CardHeader>
            <CardContent>
              <div className="text-sm text-muted-foreground">
                No snapshots found for this VM
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="metrics">
          <Card>
            <CardHeader>
              <CardTitle>Resource Metrics</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-sm text-muted-foreground">
                Metrics data will be displayed here
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
