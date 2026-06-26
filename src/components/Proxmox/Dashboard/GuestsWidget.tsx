import React from 'react';
import { WidgetContainer } from './WidgetContainer';
import { Card, CardContent, CardHeader } from '@/components/ui/index';
import { Progress } from '@/components/ui/index';
import { AlertCircle, CheckCircle, XCircle } from 'lucide-react';

interface GuestInfo {
  id: string;
  name: string;
  type: 'qemu' | 'lxc';
  status: 'running' | 'stopped' | 'paused';
  cpu: number;
  memory: number;
  memoryTotal: number;
  disk: number;
  diskTotal: number;
  uptime?: string;
}

interface GuestsWidgetProps {
  guests: GuestInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
}

export function GuestsWidget({ guests, onRefresh, isLoading }: GuestsWidgetProps) {
  const runningCount = guests.filter((g) => g.status === 'running').length;
  const stoppedCount = guests.filter((g) => g.status === 'stopped').length;
  const pausedCount = guests.filter((g) => g.status === 'paused').length;

  return (
    <WidgetContainer
      title="Guests"
      onRefresh={onRefresh}
      isLoading={isLoading}
      size="large"
    >
      <div className="grid grid-cols-3 gap-4 mb-4">
        <div className="text-center">
          <div className="text-2xl font-bold">{runningCount}</div>
          <div className="text-xs text-muted-foreground">Running</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold">{stoppedCount}</div>
          <div className="text-xs text-muted-foreground">Stopped</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold">{pausedCount}</div>
          <div className="text-xs text-muted-foreground">Paused</div>
        </div>
      </div>
      <div className="space-y-2">
        {guests.map((guest) => (
          <Card key={guest.id}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <div className="flex items-center space-x-2">
                {guest.status === 'running' ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : guest.status === 'stopped' ? (
                  <XCircle className="h-4 w-4 text-red-500" />
                ) : (
                  <AlertCircle className="h-4 w-4 text-yellow-500" />
                )}
                <span className="font-medium">{guest.name}</span>
              </div>
              <span className="text-xs text-muted-foreground capitalize">
                {guest.type === 'qemu' ? 'VM' : 'CT'}
              </span>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="flex justify-between text-xs">
                <span>Status</span>
                <span className="capitalize">{guest.status}</span>
              </div>
              {guest.uptime && (
                <div className="flex justify-between text-xs">
                  <span>Uptime</span>
                  <span>{guest.uptime}</span>
                </div>
              )}
              <div className="flex justify-between text-xs">
                <span>CPU</span>
                <span>{guest.cpu}%</span>
              </div>
              <Progress value={guest.cpu} className="h-1" />
              <div className="flex justify-between text-xs">
                <span>Memory</span>
                <span>{Math.round((guest.memory / guest.memoryTotal) * 100)}%</span>
              </div>
              <Progress value={(guest.memory / guest.memoryTotal) * 100} className="h-1" />
              <div className="flex justify-between text-xs">
                <span>Disk</span>
                <span>{Math.round((guest.disk / guest.diskTotal) * 100)}%</span>
              </div>
              <Progress value={(guest.disk / guest.diskTotal) * 100} className="h-1" />
            </CardContent>
          </Card>
        ))}
      </div>
    </WidgetContainer>
  );
}
