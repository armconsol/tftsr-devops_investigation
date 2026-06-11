import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal } from 'lucide-react';

interface UpdateInfo {
  id: string;
  name: string;
  version: string;
  remote: string;
  node?: string;
  category: string;
  installed: string;
  available: string;
  status: 'up-to-date' | 'available' | 'error';
}

interface UpdatesListProps {
  updates: UpdateInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onInstall?: (update: UpdateInfo) => void;
}

export function UpdatesList({
  updates,
  onRefresh,
  isLoading,
  onInstall,
}: UpdatesListProps) {
  const upToDateCount = updates.filter((u) => u.status === 'up-to-date').length;
  const availableCount = updates.filter((u) => u.status === 'available').length;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Updates</CardTitle>
        <div className="flex space-x-2">
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-green-500">●</span>
            <span>{upToDateCount} Up-to-date</span>
          </div>
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-yellow-500">●</span>
            <span>{availableCount} Available</span>
          </div>
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Version</TableHead>
                <TableHead>Remote</TableHead>
                <TableHead>Node</TableHead>
                <TableHead>Category</TableHead>
                <TableHead>Installed</TableHead>
                <TableHead>Available</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {updates.map((update) => (
                <TableRow key={update.id}>
                  <TableCell className="font-medium">{update.name}</TableCell>
                  <TableCell>{update.version}</TableCell>
                  <TableCell>{update.remote}</TableCell>
                  <TableCell>{update.node || '-'}</TableCell>
                  <TableCell>{update.category}</TableCell>
                  <TableCell>{update.installed}</TableCell>
                  <TableCell>{update.available}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      update.status === 'up-to-date' ? 'bg-green-100 text-green-800' :
                      update.status === 'error' ? 'bg-red-100 text-red-800' :
                      'bg-yellow-100 text-yellow-800'
                    }`}>
                      {update.status}
                    </span>
                  </TableCell>
                  <TableCell className="text-right">
                    {update.status === 'available' && (
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onInstall?.(update)}
                        title="Install"
                      >
                        <span className="h-4 w-4 text-xs">⬇️</span>
                      </button>
                    )}
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
