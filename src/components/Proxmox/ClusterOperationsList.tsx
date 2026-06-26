import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal } from 'lucide-react';

interface ClusterOperationInfo {
  id: string;
  name: string;
  type: string;
  status: string;
  node?: string;
  started?: string;
  ended?: string;
  progress?: number;
}

interface ClusterOperationsListProps {
  operations: ClusterOperationInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onCancel?: (op: ClusterOperationInfo) => void;
}

export function ClusterOperationsList({
  operations,
  onRefresh,
  isLoading,
  onCancel,
}: ClusterOperationsListProps) {
  const runningCount = operations.filter((o) => o.status === 'running').length;
  const completedCount = operations.filter((o) => o.status === 'completed').length;
  const failedCount = operations.filter((o) => o.status === 'failed').length;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Cluster Operations</CardTitle>
        <div className="flex space-x-2">
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-yellow-500">●</span>
            <span>{runningCount} Running</span>
          </div>
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-green-500">●</span>
            <span>{completedCount} Completed</span>
          </div>
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-red-500">●</span>
            <span>{failedCount} Failed</span>
          </div>
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Node</TableHead>
                <TableHead>Started</TableHead>
                <TableHead>Ended</TableHead>
                <TableHead>Progress</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {operations.map((op) => (
                <TableRow key={op.id}>
                  <TableCell className="font-medium">{op.name}</TableCell>
                  <TableCell>{op.type}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      op.status === 'running' ? 'bg-yellow-100 text-yellow-800' :
                      op.status === 'completed' ? 'bg-green-100 text-green-800' :
                      'bg-red-100 text-red-800'
                    }`}>
                      {op.status}
                    </span>
                  </TableCell>
                  <TableCell>{op.node || '-'}</TableCell>
                  <TableCell>{op.started || '-'}</TableCell>
                  <TableCell>{op.ended || '-'}</TableCell>
                  <TableCell>
                    {op.progress !== undefined && (
                      <div className="w-full max-w-[100px]">
                        <div className="h-2 w-full rounded-full bg-gray-200">
                          <div
                            className="h-2 rounded-full bg-primary"
                            style={{ width: `${op.progress}%` }}
                          />
                        </div>
                        <div className="text-xs text-center mt-1">{op.progress}%</div>
                      </div>
                    )}
                  </TableCell>
                  <TableCell className="text-right">
                    {op.status === 'running' && (
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onCancel?.(op)}
                        title="Cancel"
                      >
                        <span className="h-4 w-4 text-xs">⏹️</span>
                      </button>
                    )}
                    <button
                      className="rounded-md p-1 hover:bg-accent"
                      title="More"
                    >
                      <MoreHorizontal className="h-4 w-4" />
                    </button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  );
}
