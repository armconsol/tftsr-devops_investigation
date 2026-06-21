import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal } from 'lucide-react';

interface StorageInfo {
  id: string;
  storage: string;
  name: string;
  type: string;
  content: string;
  node?: string;
  size: number;
  used: number;
  available: number;
  status: string;
}

interface StorageListProps {
  storages: StorageInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onEdit?: (storage: StorageInfo) => void;
  onDelete?: (storage: StorageInfo) => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0 || bytes < 0 || isNaN(bytes)) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function StorageList({
  storages,
  onRefresh,
  isLoading,
  onEdit,
  onDelete,
}: StorageListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Storages</CardTitle>
        <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
          Refresh
        </Button>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Content</TableHead>
                <TableHead>Node</TableHead>
                <TableHead>Used</TableHead>
                <TableHead>Total</TableHead>
                <TableHead>Available</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {storages.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={9} className="text-center py-8 text-muted-foreground">
                    No storage configured or unable to fetch storage data
                  </TableCell>
                </TableRow>
              ) : (
                storages.map((storage) => (
                  <TableRow key={storage.id || storage.storage}>
                    <TableCell className="font-medium">{storage.storage || storage.name}</TableCell>
                    <TableCell>{storage.type || '-'}</TableCell>
                    <TableCell>{storage.content || '-'}</TableCell>
                    <TableCell>{storage.node || '-'}</TableCell>
                    <TableCell>{storage.used ? formatBytes(storage.used) : '-'}</TableCell>
                    <TableCell>{storage.size ? formatBytes(storage.size) : '-'}</TableCell>
                    <TableCell>{storage.available ? formatBytes(storage.available) : '-'}</TableCell>
                    <TableCell>
                      <span className="inline-flex items-center rounded-full px-2 py-1 text-xs font-medium bg-green-100 text-green-800">
                        {storage.status || 'available'}
                      </span>
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end space-x-2">
                        <button
                          className="rounded-md p-1 hover:bg-accent"
                          onClick={() => onEdit?.(storage)}
                          title="Edit"
                        >
                          <span className="h-4 w-4 text-xs">✏️</span>
                        </button>
                        <button
                          className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                          onClick={() => onDelete?.(storage)}
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
                ))
              )}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  );
}
