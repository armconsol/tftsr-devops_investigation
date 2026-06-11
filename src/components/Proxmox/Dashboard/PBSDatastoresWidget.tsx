import React from 'react';
import { WidgetContainer } from './WidgetContainer';
import { Card, CardContent, CardHeader } from '@/components/ui/index';
import { Progress } from '@/components/ui/index';
import { AlertCircle, CheckCircle } from 'lucide-react';

interface DatastoreInfo {
  id: string;
  name: string;
  node: string;
  type: string;
  status: 'online' | 'under_maintenance' | 'error';
  used: number;
  available: number;
  total: number;
}

interface PBSDatastoresWidgetProps {
  datastores: DatastoreInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
}

export function PBSDatastoresWidget({
  datastores,
  onRefresh,
  isLoading,
}: PBSDatastoresWidgetProps) {
  const onlineCount = datastores.filter((d) => d.status === 'online').length;
  const maintenanceCount = datastores.filter(
    (d) => d.status === 'under_maintenance'
  ).length;

  return (
    <WidgetContainer
      title="Datastores"
      onRefresh={onRefresh}
      isLoading={isLoading}
      size="large"
    >
      <div className="grid grid-cols-2 gap-4 mb-4">
        <div className="text-center">
          <div className="text-2xl font-bold">{onlineCount}</div>
          <div className="text-xs text-muted-foreground">Online</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold">{maintenanceCount}</div>
          <div className="text-xs text-muted-foreground">Maintenance</div>
        </div>
      </div>
      <div className="space-y-2">
        {datastores.map((datastore) => (
          <Card key={datastore.id}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <div className="flex items-center space-x-2">
                {datastore.status === 'online' ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : (
                  <AlertCircle className="h-4 w-4 text-yellow-500" />
                )}
                <span className="font-medium">{datastore.name}</span>
              </div>
              <span className="text-xs text-muted-foreground">
                {datastore.type}
              </span>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="flex justify-between text-xs">
                <span>Node</span>
                <span>{datastore.node}</span>
              </div>
              <div className="flex justify-between text-xs">
                <span>Status</span>
                <span className="capitalize">{datastore.status.replace('_', ' ')}</span>
              </div>
              <div className="flex justify-between text-xs">
                <span>Usage</span>
                <span>{Math.round((datastore.used / datastore.total) * 100)}%</span>
              </div>
              <Progress
                value={(datastore.used / datastore.total) * 100}
                className="h-1"
              />
              <div className="flex justify-between text-xs">
                <span>Used</span>
                <span>{(datastore.used / (1024 * 1024 * 1024)).toFixed(2)} GB</span>
              </div>
              <div className="flex justify-between text-xs">
                <span>Available</span>
                <span>{(datastore.available / (1024 * 1024 * 1024)).toFixed(2)} GB</span>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </WidgetContainer>
  );
}
