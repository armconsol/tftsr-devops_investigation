import React from 'react';
import { WidgetContainer } from './WidgetContainer';
import { Card, CardContent, CardHeader } from '@/components/ui/index';
import { AlertCircle, CheckCircle } from 'lucide-react';

interface SDNZoneInfo {
  id: string;
  name: string;
  type: string;
  fabric: string;
  status: 'available' | 'error' | 'pending' | 'unknown';
  vni?: number;
  routeTarget?: string;
}

interface SDNWidgetProps {
  zones: SDNZoneInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
}

export function SDNWidget({ zones, onRefresh, isLoading }: SDNWidgetProps) {
  const availableCount = zones.filter((z) => z.status === 'available').length;
  const errorCount = zones.filter((z) => z.status === 'error').length;

  return (
    <WidgetContainer
      title="SDN"
      onRefresh={onRefresh}
      isLoading={isLoading}
      size="medium"
    >
      <div className="grid grid-cols-2 gap-4 mb-4">
        <div className="text-center">
          <div className="text-2xl font-bold">{availableCount}</div>
          <div className="text-xs text-muted-foreground">Available</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold">{errorCount}</div>
          <div className="text-xs text-muted-foreground">Errors</div>
        </div>
      </div>
      <div className="space-y-2">
        {zones.map((zone) => (
          <Card key={zone.id}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <div className="flex items-center space-x-2">
                {zone.status === 'available' ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : (
                  <AlertCircle className="h-4 w-4 text-yellow-500" />
                )}
                <span className="font-medium">{zone.name}</span>
              </div>
              <span className="text-xs text-muted-foreground capitalize">
                {zone.type}
              </span>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="flex justify-between text-xs">
                <span>Fabric</span>
                <span>{zone.fabric}</span>
              </div>
              <div className="flex justify-between text-xs">
                <span>Status</span>
                <span className="capitalize">{zone.status}</span>
              </div>
              {zone.vni && (
                <div className="flex justify-between text-xs">
                  <span>VNI</span>
                  <span>{zone.vni}</span>
                </div>
              )}
              {zone.routeTarget && (
                <div className="flex justify-between text-xs">
                  <span>Route Target</span>
                  <span>{zone.routeTarget}</span>
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </WidgetContainer>
  );
}
