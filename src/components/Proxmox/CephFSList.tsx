import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal } from 'lucide-react';

interface CephFSInfo {
  id: string;
  name: string;
  pool: string;
  dataPool?: string;
  metadataPool?: string;
  status: string;
}

interface CephFSListProps {
  cephfs: CephFSInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onEdit?: (cephfs: CephFSInfo) => void;
  onDelete?: (cephfs: CephFSInfo) => void;
}

export function CephFSList({
  cephfs,
  onRefresh,
  isLoading,
  onEdit,
  onDelete,
}: CephFSListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Ceph Filesystems</CardTitle>
        <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
          Refresh
        </Button>
        <Button size="sm">
          <span className="mr-2 h-4 w-4">+</span>
          New Filesystem
        </Button>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Pool</TableHead>
                <TableHead>Data Pool</TableHead>
                <TableHead>Metadata Pool</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {cephfs.map((fs) => (
                <TableRow key={fs.id}>
                  <TableCell className="font-medium">{fs.name}</TableCell>
                  <TableCell>{fs.pool}</TableCell>
                  <TableCell>{fs.dataPool || '-'}</TableCell>
                  <TableCell>{fs.metadataPool || '-'}</TableCell>
                  <TableCell>
                    <span className="inline-flex items-center rounded-full px-2 py-1 text-xs font-medium bg-green-100 text-green-800">
                      {fs.status}
                    </span>
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onEdit?.(fs)}
                        title="Edit"
                      >
                        <span className="h-4 w-4 text-xs">✏️</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onDelete?.(fs)}
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
