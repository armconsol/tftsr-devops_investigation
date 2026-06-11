import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal, Trash2 } from 'lucide-react';

interface HAGroupInfo {
  id: string;
  name: string;
  resources: number;
  managed: number;
  failed: number;
  status: string;
}

interface HAGroupsListProps {
  groups: HAGroupInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onEdit?: (group: HAGroupInfo) => void;
  onDelete?: (group: HAGroupInfo) => void;
  onEnable?: (group: HAGroupInfo) => void;
  onDisable?: (group: HAGroupInfo) => void;
}

export function HAGroupsList({
  groups,
  onRefresh,
  isLoading,
  onEdit,
  onDelete,
  onEnable,
  onDisable,
}: HAGroupsListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>HA Groups</CardTitle>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          <Button size="sm">
            <span className="mr-2 h-4 w-4">+</span>
            New Group
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Resources</TableHead>
                <TableHead>Managed</TableHead>
                <TableHead>Failed</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {groups.map((group) => (
                <TableRow key={group.id}>
                  <TableCell className="font-medium">{group.name}</TableCell>
                  <TableCell>{group.resources}</TableCell>
                  <TableCell>{group.managed}</TableCell>
                  <TableCell>{group.failed}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      group.status === 'healthy' ? 'bg-green-100 text-green-800' :
                      group.status === 'error' ? 'bg-red-100 text-red-800' :
                      'bg-yellow-100 text-yellow-800'
                    }`}>
                      {group.status}
                    </span>
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onEdit?.(group)}
                        title="Edit"
                      >
                        <span className="h-4 w-4 text-xs">✏️</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => group.managed > 0 ? onDisable?.(group) : onEnable?.(group)}
                        title={group.managed > 0 ? 'Disable' : 'Enable'}
                      >
                        {group.managed > 0 ? (
                          <span className="h-4 w-4 text-xs">⏸️</span>
                        ) : (
                          <span className="h-4 w-4 text-xs">▶️</span>
                        )}
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onDelete?.(group)}
                        title="Delete"
                      >
                        <Trash2 className="h-4 w-4" />
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
