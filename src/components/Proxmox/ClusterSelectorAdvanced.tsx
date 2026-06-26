import React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui/index';

export interface ClusterSelectorProps {
  clusters: { id: string; name: string; type: string; status: string }[];
  selectedIds: string[];
  onToggleSelect?: (id: string) => void;
  onSelectAll?: () => void;
  onClear?: () => void;
  onAddCluster?: () => void;
}

export function ClusterSelector({
  clusters,
  selectedIds,
  onToggleSelect,
  onSelectAll,
  onClear,
  onAddCluster,
}: ClusterSelectorProps) {
  const allSelected = clusters.length > 0 && selectedIds.length === clusters.length;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Cluster Selector</CardTitle>
        <div className="flex space-x-2">
          <Button
            variant={allSelected ? 'default' : 'outline'}
            size="sm"
            onClick={onSelectAll}
          >
            <span className="mr-2 h-4 w-4">✅</span>
            All
          </Button>
          <Button variant="outline" size="sm" onClick={onClear}>
            Clear
          </Button>
          {onAddCluster && (
            <Button size="sm" onClick={onAddCluster}>
              <span className="mr-2 h-4 w-4">+</span>
              Add
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          {clusters.map((cluster) => {
            const isSelected = selectedIds.includes(cluster.id);
            return (
              <div
                key={cluster.id}
                className={`flex items-center justify-between p-3 rounded-md border ${
                  isSelected ? 'border-primary bg-primary/10' : 'border-border'
                }`}
              >
                <div className="flex items-center space-x-3">
                  <input
                    type="checkbox"
                    checked={isSelected}
                    onChange={() => onToggleSelect?.(cluster.id)}
                    className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                  />
                  <div>
                    <div className="font-medium">{cluster.name}</div>
                    <div className="text-xs text-muted-foreground">
                      {cluster.type} • {cluster.status}
                    </div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}
