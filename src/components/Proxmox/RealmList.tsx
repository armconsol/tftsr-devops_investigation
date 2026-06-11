import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal, Trash2 } from 'lucide-react';

interface RealmInfo {
  id: string;
  type: 'pam' | 'ldap' | 'ad' | 'openid';
  server?: string;
  baseDn?: string;
  status: string;
}

interface RealmListProps {
  realms: RealmInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onEdit?: (realm: RealmInfo) => void;
  onDelete?: (realm: RealmInfo) => void;
  onSync?: (realm: RealmInfo) => void;
}

export function RealmList({
  realms,
  onRefresh,
  isLoading,
  onEdit,
  onDelete,
  onSync,
}: RealmListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Authentication Realms</CardTitle>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          <Button size="sm">
            <span className="mr-2 h-4 w-4">+</span>
            New Realm
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Realm ID</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Server</TableHead>
                <TableHead>Base DN</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {realms.map((realm) => (
                <TableRow key={realm.id}>
                  <TableCell className="font-medium">{realm.id}</TableCell>
                  <TableCell>
                    <span className="inline-flex items-center rounded-full px-2 py-1 text-xs font-medium bg-blue-100 text-blue-800">
                      {realm.type.toUpperCase()}
                    </span>
                  </TableCell>
                  <TableCell>{realm.server || '-'}</TableCell>
                  <TableCell>{realm.baseDn || '-'}</TableCell>
                  <TableCell>
                    <span className="inline-flex items-center rounded-full px-2 py-1 text-xs font-medium bg-green-100 text-green-800">
                      Active
                    </span>
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onEdit?.(realm)}
                        title="Edit"
                      >
                        <span className="h-4 w-4 text-xs">✏️</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onSync?.(realm)}
                        title="Sync Users"
                      >
                        <span className="h-4 w-4 text-xs">🔄</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onDelete?.(realm)}
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
