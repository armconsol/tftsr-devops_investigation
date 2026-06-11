import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal } from 'lucide-react';

interface HAResourceInfo {
  id: string;
  name: string;
  type: string;
  group: string;
  node: string;
  managed: boolean;
  failed: boolean;
  status: string;
}

interface HAResourcesListProps {
  resources: HAResourceInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onManage?: (resource: HAResourceInfo) => void;
  onUnmanage?: (resource: HAResourceInfo) => void;
  onFailover?: (resource: HAResourceInfo) => void;
}

export function HAResourcesList({
  resources,
  onRefresh,
  isLoading,
  onManage,
  onUnmanage,
  onFailover,
}: HAResourcesListProps) {
  const managedCount = resources.filter((r) => r.managed).length;
  const failedCount = resources.filter((r) => r.failed).length;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>HA Resources</CardTitle>
        <div className="flex space-x-2">
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-green-500">●</span>
            <span>{managedCount} Managed</span>
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
                <TableHead>Group</TableHead>
                <TableHead>Node</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {resources.map((resource) => (
                <TableRow key={resource.id}>
                  <TableCell className="font-medium">{resource.name}</TableCell>
                  <TableCell>{resource.type}</TableCell>
                  <TableCell>{resource.group}</TableCell>
                  <TableCell>{resource.node}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      resource.failed ? 'bg-red-100 text-red-800' :
                      resource.managed ? 'bg-green-100 text-green-800' :
                      'bg-gray-100 text-gray-800'
                    }`}>
                      {resource.failed ? 'Failed' : resource.managed ? 'Managed' : 'Unmanaged'}
                    </span>
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      {resource.managed ? (
                        <button
                          className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                          onClick={() => onUnmanage?.(resource)}
                          title="Unmanage"
                        >
                          <span className="h-4 w-4 text-xs">⏹️</span>
                        </button>
                      ) : (
                        <button
                          className="rounded-md p-1 hover:bg-green-100 hover:text-green-600"
                          onClick={() => onManage?.(resource)}
                          title="Manage"
                        >
                          <span className="h-4 w-4 text-xs">▶️</span>
                        </button>
                      )}
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onFailover?.(resource)}
                        title="Failover"
                      >
                        <span className="h-4 w-4 text-xs">🔄</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        title="More"
                      >
                        <MoreHorizontal className="h-4 w-4" />
                      </button>
                    </div>
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
