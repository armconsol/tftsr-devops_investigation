import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal } from 'lucide-react';

interface AclInfo {
  id: string;
  path: string;
  type: 'user' | 'group' | 'role';
  principal: string;
  roles: string[];
  propagate: boolean;
}

interface AclListProps {
  acls: AclInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onAdd?: () => void;
  onEdit?: (acl: AclInfo) => void;
  onDelete?: (acl: AclInfo) => void;
}

export function AclList({
  acls,
  onRefresh,
  isLoading,
  onAdd,
  onEdit,
  onDelete,
}: AclListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Access Control Lists (ACL)</CardTitle>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          {onAdd && (
            <Button size="sm" onClick={onAdd}>
              <span className="mr-2 h-4 w-4">+</span>
              New ACL
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Path</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Principal</TableHead>
                <TableHead>Roles</TableHead>
                <TableHead>Propagate</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {acls.map((acl) => (
                <TableRow key={acl.id}>
                  <TableCell className="font-mono text-xs">{acl.path}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      acl.type === 'user' ? 'bg-blue-100 text-blue-800' :
                      acl.type === 'group' ? 'bg-purple-100 text-purple-800' :
                      'bg-orange-100 text-orange-800'
                    }`}>
                      {acl.type}
                    </span>
                  </TableCell>
                  <TableCell>{acl.principal}</TableCell>
                  <TableCell>
                    <div className="flex flex-wrap gap-1">
                      {acl.roles.map((role) => (
                        <span key={role} className="rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-800">
                          {role}
                        </span>
                      ))}
                    </div>
                  </TableCell>
                  <TableCell>{acl.propagate ? 'Yes' : 'No'}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onEdit?.(acl)}
                        title="Edit"
                      >
                        <span className="h-4 w-4 text-xs">✏️</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onDelete?.(acl)}
                        title="Delete"
                      >
                        <span className="h-4 w-4 text-xs">🗑️</span>
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
