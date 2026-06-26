import React from 'react';
import { WidgetContainer } from './WidgetContainer';
import { Card, CardContent, CardHeader } from '@/components/ui/index';
import { AlertCircle, CheckCircle, XCircle } from 'lucide-react';

interface RemoteInfo {
  id: string;
  name: string;
  type: 've' | 'pbs';
  url: string;
  status: 'online' | 'offline' | 'error';
  lastCheck?: string;
}

interface RemotesWidgetProps {
  remotes: RemoteInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
}

export function RemotesWidget({ remotes, onRefresh, isLoading }: RemotesWidgetProps) {
  const onlineCount = remotes.filter((r) => r.status === 'online').length;
  const offlineCount = remotes.filter((r) => r.status !== 'online').length;

  return (
    <WidgetContainer
      title="Remotes"
      onRefresh={onRefresh}
      isLoading={isLoading}
      size="medium"
    >
      <div className="grid grid-cols-2 gap-4 mb-4">
        <div className="text-center">
          <div className="text-2xl font-bold">{onlineCount}</div>
          <div className="text-xs text-muted-foreground">Online</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold">{offlineCount}</div>
          <div className="text-xs text-muted-foreground">Offline</div>
        </div>
      </div>
      <div className="space-y-2">
        {remotes.map((remote) => (
          <Card key={remote.id}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <div className="flex items-center space-x-2">
                {remote.status === 'online' ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : remote.status === 'offline' ? (
                  <XCircle className="h-4 w-4 text-red-500" />
                ) : (
                  <AlertCircle className="h-4 w-4 text-yellow-500" />
                )}
                <span className="font-medium">{remote.name}</span>
              </div>
              <span className="text-xs text-muted-foreground">
                {remote.type === 've' ? 'PVE' : 'PBS'}
              </span>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="flex justify-between text-xs">
                <span>URL</span>
                <span className="truncate max-w-[150px]">{remote.url}</span>
              </div>
              <div className="flex justify-between text-xs">
                <span>Status</span>
                <span className="capitalize">{remote.status}</span>
              </div>
              {remote.lastCheck && (
                <div className="flex justify-between text-xs">
                  <span>Last Check</span>
                  <span>{remote.lastCheck}</span>
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </WidgetContainer>
  );
}
