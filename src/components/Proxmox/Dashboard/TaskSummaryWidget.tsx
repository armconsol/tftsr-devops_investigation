import React from 'react';
import { WidgetContainer } from './WidgetContainer';
import { Card, CardContent, CardHeader } from '@/components/ui/index';
import { AlertCircle, CheckCircle } from 'lucide-react';

interface TaskInfo {
  id: string;
  type: string;
  status: 'running' | 'success' | 'failed' | 'pending';
  node: string;
  startTime?: string;
  endTime?: string;
  description: string;
}

interface TaskSummaryWidgetProps {
  tasks: TaskInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
}

export function TaskSummaryWidget({ tasks, onRefresh, isLoading }: TaskSummaryWidgetProps) {
  const runningCount = tasks.filter((t) => t.status === 'running').length;
  const successCount = tasks.filter((t) => t.status === 'success').length;
  const failedCount = tasks.filter((t) => t.status === 'failed').length;

  return (
    <WidgetContainer
      title="Task Summary"
      onRefresh={onRefresh}
      isLoading={isLoading}
      size="medium"
    >
      <div className="grid grid-cols-3 gap-4 mb-4">
        <div className="text-center">
          <div className="text-2xl font-bold">{runningCount}</div>
          <div className="text-xs text-muted-foreground">Running</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold">{successCount}</div>
          <div className="text-xs text-muted-foreground">Success</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold">{failedCount}</div>
          <div className="text-xs text-muted-foreground">Failed</div>
        </div>
      </div>
      <div className="space-y-2">
        {tasks.slice(0, 5).map((task) => (
          <Card key={task.id}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <div className="flex items-center space-x-2">
                {task.status === 'running' ? (
                  <div className="h-2 w-2 rounded-full bg-blue-500 animate-pulse" />
                ) : task.status === 'success' ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : (
                  <AlertCircle className="h-4 w-4 text-red-500" />
                )}
                <span className="font-medium">{task.type}</span>
              </div>
              <span className="text-xs text-muted-foreground capitalize">
                {task.status}
              </span>
            </CardHeader>
            <CardContent className="space-y-1">
              <div className="flex justify-between text-xs">
                <span>Node</span>
                <span>{task.node}</span>
              </div>
              <div className="text-xs text-muted-foreground truncate">
                {task.description}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
      {tasks.length > 5 && (
        <div className="text-center text-xs text-muted-foreground mt-2">
          +{tasks.length - 5} more tasks
        </div>
      )}
    </WidgetContainer>
  );
}
