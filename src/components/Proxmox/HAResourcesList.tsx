import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Play, Trash2, RefreshCw } from 'lucide-react';
import { HaResource } from '@/lib/proxmoxClient';

interface HAResourcesListProps {
  resources: HaResource[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onEnable?: (resource: HaResource) => void;
  onRemove?: (resource: HaResource) => void;
}

export function HAResourcesList({
  resources,
  onRefresh,
  isLoading,
  onEnable,
  onRemove,
}: HAResourcesListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>HA Resources</CardTitle>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Resource ID</TableHead>
                <TableHead>Group</TableHead>
                <TableHead>State</TableHead>
                <TableHead>Max Restart</TableHead>
                <TableHead>Max Relocate</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {resources.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                    No HA resources configured
                  </TableCell>
                </TableRow>
              ) : (
                resources.map((resource) => (
                  <TableRow key={resource.sid}>
                    <TableCell className="font-medium font-mono text-xs">{resource.sid}</TableCell>
                    <TableCell>{resource.group ?? '-'}</TableCell>
                    <TableCell>
                      <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                        resource.state === 'started' ? 'bg-green-100 text-green-800' :
                        resource.state === 'stopped' ? 'bg-gray-100 text-gray-600' :
                        resource.state === 'error' ? 'bg-red-100 text-red-800' :
                        'bg-yellow-100 text-yellow-800'
                      }`}>
                        {resource.state}
                      </span>
                    </TableCell>
                    <TableCell>{resource.maxRestart ?? '-'}</TableCell>
                    <TableCell>{resource.maxRelocate ?? '-'}</TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end space-x-2">
                        <button
                          className="rounded-md p-1 hover:bg-green-100 hover:text-green-600"
                          onClick={() => onEnable?.(resource)}
                          title="Enable"
                        >
                          <Play className="h-4 w-4" />
                        </button>
                        <button
                          className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                          onClick={() => onRemove?.(resource)}
                          title="Remove"
                        >
                          <Trash2 className="h-4 w-4" />
                        </button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  );
}
