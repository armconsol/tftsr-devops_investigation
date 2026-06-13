import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Trash2, Pencil, PlusCircle, RefreshCw } from 'lucide-react';
import { HaGroup } from '@/lib/proxmoxClient';

interface HAGroupsListProps {
  groups: HaGroup[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onCreate?: () => void;
  onEdit?: (group: HaGroup) => void;
  onDelete?: (id: string) => void;
}

export function HAGroupsList({
  groups,
  onRefresh,
  isLoading,
  onCreate,
  onEdit,
  onDelete,
}: HAGroupsListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>HA Groups</CardTitle>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
          <Button size="sm" onClick={onCreate}>
            <PlusCircle className="mr-2 h-4 w-4" />
            Add Group
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Nodes</TableHead>
                <TableHead>Restricted</TableHead>
                <TableHead>No-Quorum Policy</TableHead>
                <TableHead>Comment</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {groups.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                    No HA groups configured
                  </TableCell>
                </TableRow>
              ) : (
                groups.map((group) => (
                  <TableRow key={group.id}>
                    <TableCell className="font-medium">{group.id}</TableCell>
                    <TableCell className="font-mono text-xs">{group.nodes}</TableCell>
                    <TableCell>
                      {group.restricted ? (
                        <span className="inline-flex items-center rounded-full px-2 py-1 text-xs font-medium bg-yellow-100 text-yellow-800">
                          Yes
                        </span>
                      ) : (
                        <span className="inline-flex items-center rounded-full px-2 py-1 text-xs font-medium bg-gray-100 text-gray-600">
                          No
                        </span>
                      )}
                    </TableCell>
                    <TableCell>{group.noQuorumPolicy ?? '-'}</TableCell>
                    <TableCell className="text-muted-foreground text-sm">{group.comment ?? '-'}</TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end space-x-2">
                        <button
                          className="rounded-md p-1 hover:bg-accent"
                          onClick={() => onEdit?.(group)}
                          title="Edit"
                        >
                          <Pencil className="h-4 w-4" />
                        </button>
                        <button
                          className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                          onClick={() => onDelete?.(group.id)}
                          title="Delete"
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
