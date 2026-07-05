import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal, Trash2 } from 'lucide-react';
import { formatBytes } from '@/lib/format';

interface PoolInfo {
  id: string;
  name: string;
  type: string;
  size: number;
  minSize: number;
  used: number;
  available: number;
  total: number;
  usedPercent: number;
}

interface PoolListProps {
  pools: PoolInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onSetQuota?: (pool: PoolInfo) => void;
  onDelete?: (pool: PoolInfo) => void;
  onEdit?: (pool: PoolInfo) => void;
}

export function PoolList({
  pools,
  onRefresh,
  isLoading,
  onSetQuota,
  onDelete,
  onEdit,
}: PoolListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Ceph Pools</CardTitle>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          <Button size="sm">
            <span className="mr-2 h-4 w-4">+</span>
            New Pool
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Size</TableHead>
                <TableHead>Min Size</TableHead>
                <TableHead>Used</TableHead>
                <TableHead>Available</TableHead>
                <TableHead>% Used</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {pools.map((pool) => (
                <TableRow key={pool.id}>
                  <TableCell className="font-medium">{pool.name}</TableCell>
                  <TableCell>{pool.type}</TableCell>
                  <TableCell>{pool.size}</TableCell>
                  <TableCell>{pool.minSize}</TableCell>
                  <TableCell>{formatBytes(pool.used)}</TableCell>
                  <TableCell>{formatBytes(pool.available)}</TableCell>
                  <TableCell>
                    <div className="flex items-center space-x-2">
                      <div className="h-2 w-24 bg-slate-200 rounded-full overflow-hidden">
                        <div
                          className={`h-full rounded-full ${
                            pool.usedPercent > 80 ? 'bg-red-500' :
                            pool.usedPercent > 60 ? 'bg-yellow-500' :
                            'bg-green-500'
                          }`}
                          style={{ width: `${pool.usedPercent}%` }}
                        />
                      </div>
                      <span className="text-xs">{pool.usedPercent.toFixed(1)}%</span>
                    </div>
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onEdit?.(pool)}
                        title="Edit"
                      >
                        <span className="h-4 w-4 text-xs">✏️</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onSetQuota?.(pool)}
                        title="Set Quota"
                      >
                        <span className="h-4 w-4 text-xs">📊</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onDelete?.(pool)}
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
