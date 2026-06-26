import React from 'react';
import { WidgetContainer } from './WidgetContainer';
import { Card, CardContent, CardHeader } from '@/components/ui/index';
import { Progress } from '@/components/ui/index';
import { AlertCircle, CheckCircle, XCircle } from 'lucide-react';

interface NodeInfo {
  id: string;
  name: string;
  status: 'online' | 'offline' | 'error';
  cpu: number;
  memory: number;
  memoryTotal: number;
  disk: number;
  diskTotal: number;
}

interface NodesWidgetProps {
  nodes: NodeInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
}

export function NodesWidget({ nodes, onRefresh, isLoading }: NodesWidgetProps) {
  const onlineCount = nodes.filter((n) => n.status === 'online').length;
  const offlineCount = nodes.filter((n) => n.status !== 'online').length;

  return (
    <WidgetContainer
      title="Nodes"
      onRefresh={onRefresh}
      isLoading={isLoading}
      size="large"
    >
      <div className="grid grid-cols-3 gap-4 mb-4">
        <div className="text-center">
          <div className="text-2xl font-bold">{onlineCount}</div>
          <div className="text-xs text-muted-foreground">Online</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold">{offlineCount}</div>
          <div className="text-xs text-muted-foreground">Offline</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold">{nodes.length}</div>
          <div className="text-xs text-muted-foreground">Total</div>
        </div>
      </div>
      <div className="space-y-2">
        {nodes.map((node) => (
          <Card key={node.id}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <div className="flex items-center space-x-2">
                {node.status === 'online' ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : node.status === 'offline' ? (
                  <XCircle className="h-4 w-4 text-red-500" />
                ) : (
                  <AlertCircle className="h-4 w-4 text-yellow-500" />
                )}
                <span className="font-medium">{node.name}</span>
              </div>
              <span className="text-xs text-muted-foreground capitalize">
                {node.status}
              </span>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="flex justify-between text-xs">
                <span>CPU</span>
                <span>{node.cpu}%</span>
              </div>
              <Progress value={node.cpu} className="h-1" />
              <div className="flex justify-between text-xs">
                <span>Memory</span>
                <span>{Math.round((node.memory / node.memoryTotal) * 100)}%</span>
              </div>
              <Progress value={(node.memory / node.memoryTotal) * 100} className="h-1" />
              <div className="flex justify-between text-xs">
                <span>Disk</span>
                <span>{Math.round((node.disk / node.diskTotal) * 100)}%</span>
              </div>
              <Progress value={(node.disk / node.diskTotal) * 100} className="h-1" />
            </CardContent>
          </Card>
        ))}
      </div>
    </WidgetContainer>
  );
}
