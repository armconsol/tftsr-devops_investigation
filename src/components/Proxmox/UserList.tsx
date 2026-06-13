import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Pencil, Trash2, PlusCircle, RefreshCw, Play, Pause } from 'lucide-react';
import { ProxmoxUser } from '@/lib/proxmoxClient';

interface UserListProps {
  users: ProxmoxUser[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onCreate?: () => void;
  onEdit?: (user: ProxmoxUser) => void;
  onDelete?: (user: ProxmoxUser) => void;
  onEnable?: (user: ProxmoxUser) => void;
  onDisable?: (user: ProxmoxUser) => void;
}

function formatExpiry(expire?: number): string {
  if (!expire || expire === 0) return 'Never';
  return new Date(expire * 1000).toLocaleDateString();
}

function deriveRealm(userid: string): string {
  const parts = userid.split('@');
  return parts.length > 1 ? parts[1] : '-';
}

export function UserList({
  users,
  onRefresh,
  isLoading,
  onCreate,
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
        <div className="flex items-center space-x-2">
          <div className="flex items-center space-x-1 text-sm text-muted-foreground">
            <span className="text-green-500">●</span>
            <span>{enabledCount} Enabled</span>
          </div>
          <div className="flex items-center space-x-1 text-sm text-muted-foreground">
            <span className="text-gray-400">●</span>
            <span>{disabledCount} Disabled</span>
          </div>
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
          <Button size="sm" onClick={onCreate}>
            <PlusCircle className="mr-2 h-4 w-4" />
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
                <TableHead>Realm</TableHead>
                <TableHead>Name</TableHead>
                <TableHead>Email</TableHead>
                <TableHead>Enabled</TableHead>
                <TableHead>Expire</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {users.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={7} className="text-center text-muted-foreground py-8">
                    No users found
                  </TableCell>
                </TableRow>
              ) : (
                users.map((user) => {
                  const fullName = [user.firstname, user.lastname].filter(Boolean).join(' ') || '-';
                  return (
                    <TableRow key={user.userid}>
                      <TableCell className="font-medium font-mono text-xs">{user.userid}</TableCell>
                      <TableCell>{deriveRealm(user.userid)}</TableCell>
                      <TableCell>{fullName}</TableCell>
                      <TableCell>{user.email ?? '-'}</TableCell>
                      <TableCell>
                        <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                          user.enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'
                        }`}>
                          {user.enabled ? 'Enabled' : 'Disabled'}
                        </span>
                      </TableCell>
                      <TableCell>{formatExpiry(user.expire)}</TableCell>
                      <TableCell className="text-right">
                        <div className="flex items-center justify-end space-x-2">
                          <button
                            className="rounded-md p-1 hover:bg-accent"
                            onClick={() => onEdit?.(user)}
                            title="Edit"
                          >
                            <Pencil className="h-4 w-4" />
                          </button>
                          <button
                            className="rounded-md p-1 hover:bg-accent"
                            onClick={() => user.enabled ? onDisable?.(user) : onEnable?.(user)}
                            title={user.enabled ? 'Disable' : 'Enable'}
                          >
                            {user.enabled ? (
                              <Pause className="h-4 w-4 text-yellow-600" />
                            ) : (
                              <Play className="h-4 w-4 text-green-600" />
                            )}
                          </button>
                          <button
                            className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                            onClick={() => onDelete?.(user)}
                            title="Delete"
                          >
                            <Trash2 className="h-4 w-4" />
                          </button>
                        </div>
                      </TableCell>
                    </TableRow>
                  );
                })
              )}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  );
}
