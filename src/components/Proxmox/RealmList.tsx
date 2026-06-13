import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Pencil, Trash2, PlusCircle, RefreshCw } from 'lucide-react';
import { AuthRealm } from '@/lib/proxmoxClient';

interface RealmListProps {
  realms: AuthRealm[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onCreate?: () => void;
  onEdit?: (realm: AuthRealm) => void;
  onDelete?: (realm: AuthRealm) => void;
}

export function RealmList({
  realms,
  onRefresh,
  isLoading,
  onCreate,
  onEdit,
  onDelete,
}: RealmListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Authentication Realms</CardTitle>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
          <Button size="sm" onClick={onCreate}>
            <PlusCircle className="mr-2 h-4 w-4" />
            New Realm
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Realm Name</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Comment</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {realms.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={4} className="text-center text-muted-foreground py-8">
                    No auth realms configured
                  </TableCell>
                </TableRow>
              ) : (
                realms.map((realm) => (
                  <TableRow key={realm.realm}>
                    <TableCell className="font-medium">{realm.realm}</TableCell>
                    <TableCell>
                      <span className="inline-flex items-center rounded-full px-2 py-1 text-xs font-medium bg-blue-100 text-blue-800">
                        {realm.type.toUpperCase()}
                      </span>
                    </TableCell>
                    <TableCell className="text-muted-foreground text-sm">{realm.comment ?? '-'}</TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end space-x-2">
                        <button
                          className="rounded-md p-1 hover:bg-accent"
                          onClick={() => onEdit?.(realm)}
                          title="Edit"
                        >
                          <Pencil className="h-4 w-4" />
                        </button>
                        <button
                          className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                          onClick={() => onDelete?.(realm)}
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
