import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { ContainerOverview } from '@/components/Proxmox';

interface ContainerInfo {
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

export function ProxmoxContainersPage() {
  const containers: ContainerInfo[] = [
    { id: '1', name: 'nginx-proxy', vmid: 200, node: 'pve1', status: 'running', cpu: 2, memory: 2048, disk: 20, uptime: '1d 8h' },
    { id: '2', name: 'redis-cache', vmid: 201, node: 'pve2', status: 'running', cpu: 1, memory: 1024, disk: 10, uptime: '3d 2h' },
    { id: '3', name: 'monitoring', vmid: 202, node: 'pve1', status: 'stopped', cpu: 2, memory: 4096, disk: 30 },
  ];
  const [selectedContainer, setSelectedContainer] = useState<ContainerInfo | null>(null);

  const handlePowerAction = (_action: string) => {
    // Power action handler
  };

  const handleConsole = () => {
    // Console handler
  };

  const handleContainerSelect = (container: ContainerInfo) => {
    setSelectedContainer(container);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Containers</h1>
          <p className="text-muted-foreground">Manage LXC containers</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm">
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      {selectedContainer ? (
        <ContainerOverview
          container={selectedContainer}
          onRefresh={() => {}}
          onPowerAction={handlePowerAction}
          onConsole={handleConsole}
        />
      ) : (
        <div className="grid grid-cols-1 gap-4">
          {containers.map((container) => (
            <Card key={container.id} className="cursor-pointer hover:shadow-md" onClick={() => handleContainerSelect(container)}>
              <CardHeader>
                <CardTitle>{container.name}</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-4 gap-4 text-sm">
                  <div>
                    <div className="text-muted-foreground">CT ID</div>
                    <div className="font-medium">{container.vmid}</div>
                  </div>
                  <div>
                    <div className="text-muted-foreground">Node</div>
                    <div className="font-medium">{container.node}</div>
                  </div>
                  <div>
                    <div className="text-muted-foreground">Status</div>
                    <div className="font-medium">{container.status}</div>
                  </div>
                  <div>
                    <div className="text-muted-foreground">Resources</div>
                    <div className="font-medium">{container.cpu} CPU / {container.memory}MB RAM</div>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
