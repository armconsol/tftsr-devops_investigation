import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal } from 'lucide-react';

interface CephManagerInfo {
  id: string;
  name: string;
  daemon: string;
  host: string;
  status: string;
}

interface CephManagersListProps {
  managers: CephManagerInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
}

export function CephManagersList({
  managers,
  onRefresh,
  isLoading,
}: CephManagersListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Ceph Managers</CardTitle>
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
                <TableHead>Daemon</TableHead>
                <TableHead>Host</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {managers.map((mgr) => (
                <TableRow key={mgr.id}>
                  <TableCell className="font-medium">{mgr.name}</TableCell>
                  <TableCell>{mgr.daemon}</TableCell>
                  <TableCell>{mgr.host}</TableCell>
                  <TableCell>
                    <span className="inline-flex items-center rounded-full px-2 py-1 text-xs font-medium bg-green-100 text-green-800">
                      {mgr.status}
                    </span>
                  </TableCell>
                  <TableCell className="text-right">
                    <button
                      className="rounded-md p-1 hover:bg-accent"
                      title="More"
                    >
                      <MoreHorizontal className="h-4 w-4" />
                    </button>
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
