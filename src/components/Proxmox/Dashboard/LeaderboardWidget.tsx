import React from 'react';
import { WidgetContainer } from './WidgetContainer';
import { Card, CardContent, CardHeader } from '@/components/ui/index';
import { TrendingUp } from 'lucide-react';

interface TopEntity {
  id: string;
  name: string;
  type: string;
  remote: string;
  value: number;
  unit: string;
}

interface LeaderboardWidgetProps {
  topCpu: TopEntity[];
  topMemory: TopEntity[];
  topStorage: TopEntity[];
  onRefresh?: () => void;
  isLoading?: boolean;
}

export function LeaderboardWidget({
  topCpu,
  topMemory,
  topStorage,
  onRefresh,
  isLoading,
}: LeaderboardWidgetProps) {
  return (
    <WidgetContainer
      title="Leaderboard"
      onRefresh={onRefresh}
      isLoading={isLoading}
      size="large"
    >
      <div className="space-y-4">
        <div>
          <h4 className="text-sm font-medium mb-2 flex items-center">
            <TrendingUp className="h-4 w-4 mr-2 text-blue-500" />
            Top CPU Consumers
          </h4>
          <div className="space-y-2">
            {topCpu.map((entity, index) => (
              <Card key={entity.id}>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <div className="flex items-center space-x-2">
                    <span className="font-bold text-lg">{index + 1}</span>
                    <span className="font-medium">{entity.name}</span>
                    <span className="text-xs text-muted-foreground">
                      ({entity.type})
                    </span>
                  </div>
                  <span className="text-xs text-muted-foreground">
                    {entity.remote}
                  </span>
                </CardHeader>
                <CardContent>
                  <div className="flex justify-between text-sm">
                    <span>CPU</span>
                    <span className="font-bold">{entity.value} {entity.unit}</span>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>

        <div>
          <h4 className="text-sm font-medium mb-2 flex items-center">
            <TrendingUp className="h-4 w-4 mr-2 text-purple-500" />
            Top Memory Consumers
          </h4>
          <div className="space-y-2">
            {topMemory.map((entity, index) => (
              <Card key={entity.id}>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <div className="flex items-center space-x-2">
                    <span className="font-bold text-lg">{index + 1}</span>
                    <span className="font-medium">{entity.name}</span>
                    <span className="text-xs text-muted-foreground">
                      ({entity.type})
                    </span>
                  </div>
                  <span className="text-xs text-muted-foreground">
                    {entity.remote}
                  </span>
                </CardHeader>
                <CardContent>
                  <div className="flex justify-between text-sm">
                    <span>Memory</span>
                    <span className="font-bold">{entity.value} {entity.unit}</span>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>

        <div>
          <h4 className="text-sm font-medium mb-2 flex items-center">
            <TrendingUp className="h-4 w-4 mr-2 text-orange-500" />
            Top Storage Consumers
          </h4>
          <div className="space-y-2">
            {topStorage.map((entity, index) => (
              <Card key={entity.id}>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <div className="flex items-center space-x-2">
                    <span className="font-bold text-lg">{index + 1}</span>
                    <span className="font-medium">{entity.name}</span>
                    <span className="text-xs text-muted-foreground">
                      ({entity.type})
                    </span>
                  </div>
                  <span className="text-xs text-muted-foreground">
                    {entity.remote}
                  </span>
                </CardHeader>
                <CardContent>
                  <div className="flex justify-between text-sm">
                    <span>Storage</span>
                    <span className="font-bold">{entity.value} {entity.unit}</span>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </div>
    </WidgetContainer>
  );
}
