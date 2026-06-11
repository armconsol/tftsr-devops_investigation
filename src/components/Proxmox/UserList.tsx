import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal, Trash2 } from 'lucide-react';

interface UserInfo {
  id: string;
  email?: string;
  enabled: boolean;
  lastLogin?: string;
}

interface UserListProps {
  users: UserInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onEdit?: (user: UserInfo) => void;
  onDelete?: (user: UserInfo) => void;
  onEnable?: (user: UserInfo) => void;
  onDisable?: (user: UserInfo) => void;
}

export function UserList({
  users,
  onRefresh,
  isLoading,
  onEdit,
  onDelete,
  onEnable,
  onDisable,
}: UserListProps) {
  const enabledCount = users.filter((u) => u.enabled).length;
  const disabledCount = users.filter((u) => !u.enabled).length;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Users</CardTitle>
        <div className="flex space-x-2">
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-green-500">●</span>
            <span>{enabledCount} Enabled</span>
          </div>
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-gray-500">●</span>
            <span>{disabledCount} Disabled</span>
          </div>
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          <Button size="sm">
            <span className="mr-2 h-4 w-4">+</span>
            New User
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>User ID</TableHead>
                <TableHead>Email</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Last Login</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {users.map((user) => (
                <TableRow key={user.id}>
                  <TableCell className="font-medium">{user.id}</TableCell>
                  <TableCell>{user.email || '-'}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      user.enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                    }`}>
                      {user.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </TableCell>
                  <TableCell>{user.lastLogin || '-'}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onEdit?.(user)}
                        title="Edit"
                      >
                        <span className="h-4 w-4 text-xs">✏️</span>
                      </button>
                      <button
                        className={`rounded-md p-1 hover:bg-accent ${
                          user.enabled ? 'text-green-600' : 'text-gray-600'
                        }`}
                        onClick={() => user.enabled ? onDisable?.(user) : onEnable?.(user)}
                        title={user.enabled ? 'Disable' : 'Enable'}
                      >
                        {user.enabled ? (
                          <span className="h-4 w-4 text-xs">⏸️</span>
                        ) : (
                          <span className="h-4 w-4 text-xs">▶️</span>
                        )}
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onDelete?.(user)}
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
