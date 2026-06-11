import React from 'react';
import { WidgetContainer } from './WidgetContainer';

interface MapWidgetProps {
  onRefresh?: () => void;
  isLoading?: boolean;
}

export function MapWidget({ onRefresh, isLoading }: MapWidgetProps) {
  return (
    <WidgetContainer
      title="Map"
      onRefresh={onRefresh}
      isLoading={isLoading}
      size="large"
    >
      <div className="text-center text-muted-foreground py-8">
        <p className="mb-2">Geographic map view coming soon</p>
        <p className="text-xs">
          This widget will display remote locations on a map
        </p>
      </div>
    </WidgetContainer>
  );
}
