import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal } from 'lucide-react';

interface RemoteInfo {
  id: string;
  name: string;
  type: 'pve' | 'pbs';
  url: string;
  nodeCount?: number;
  status: 'connected' | 'disconnected' | 'error';
  lastConnected?: string;
}

interface RemotesListProps {
  remotes: RemoteInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onAdd?: () => void;
  onEdit?: (remote: RemoteInfo) => void;
  onDelete?: (remote: RemoteInfo) => void;
  onConnect?: (remote: RemoteInfo) => void;
  onDisconnect?: (remote: RemoteInfo) => void;
}

export function RemotesList({
  remotes,
  onRefresh,
  isLoading,
  onAdd,
  onEdit,
  onDelete,
  onConnect,
  onDisconnect,
}: RemotesListProps) {
  const connectedCount = remotes.filter((r) => r.status === 'connected').length;
  const disconnectedCount = remotes.filter((r) => r.status === 'disconnected').length;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Remotes</CardTitle>
        <div className="flex space-x-2">
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-green-500">●</span>
            <span>{connectedCount} Connected</span>
          </div>
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-red-500">●</span>
            <span>{disconnectedCount} Disconnected</span>
          </div>
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          {onAdd && (
            <Button size="sm" onClick={onAdd}>
              <span className="mr-2 h-4 w-4">+</span>
              Add
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>URL</TableHead>
                <TableHead>Nodes</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Last Connected</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {remotes.map((remote) => (
                <TableRow key={remote.id}>
                  <TableCell className="font-medium">{remote.name}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      remote.type === 'pve' ? 'bg-blue-100 text-blue-800' : 'bg-purple-100 text-purple-800'
                    }`}>
                      {remote.type.toUpperCase()}
                    </span>
                  </TableCell>
                  <TableCell>{remote.url}</TableCell>
                  <TableCell>{remote.nodeCount || '-'}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      remote.status === 'connected' ? 'bg-green-100 text-green-800' :
                      remote.status === 'error' ? 'bg-red-100 text-red-800' :
                      'bg-gray-100 text-gray-800'
                    }`}>
                      {remote.status}
                    </span>
                  </TableCell>
                  <TableCell>{remote.lastConnected || '-'}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onEdit?.(remote)}
                        title="Edit"
                      >
                        <span className="h-4 w-4 text-xs">✏️</span>
                      </button>
                      {remote.status === 'connected' ? (
                        <button
                          className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                          onClick={() => onDisconnect?.(remote)}
                          title="Disconnect"
                        >
                          <span className="h-4 w-4 text-xs">🔌</span>
                        </button>
                      ) : (
                        <button
                          className="rounded-md p-1 hover:bg-green-100 hover:text-green-600"
                          onClick={() => onConnect?.(remote)}
                          title="Connect"
                        >
                          <span className="h-4 w-4 text-xs">🔌</span>
                        </button>
                      )}
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onDelete?.(remote)}
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
