import React from 'react';
import { WidgetContainer } from './WidgetContainer';

interface ResourceTreeWidgetProps {
  onRefresh?: () => void;
  isLoading?: boolean;
}

export function ResourceTreeWidget({
  onRefresh,
  isLoading,
}: ResourceTreeWidgetProps) {
  return (
    <WidgetContainer
      title="Resource Tree"
      onRefresh={onRefresh}
      isLoading={isLoading}
      size="large"
    >
      <div className="text-center text-muted-foreground py-8">
        <p className="mb-2">Resource tree view coming soon</p>
        <p className="text-xs">
          This widget will display a hierarchical view of all resources
        </p>
      </div>
    </WidgetContainer>
  );
}
