import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal } from 'lucide-react';

interface ConnectionInfo {
  id: string;
  remote: string;
  node: string;
  endpoint: string;
  status: 'connected' | 'connecting' | 'disconnected' | 'error';
  lastConnected?: string;
  latency?: number;
}

interface ConnectionListProps {
  connections: ConnectionInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onReconnect?: (conn: ConnectionInfo) => void;
  onDisconnect?: (conn: ConnectionInfo) => void;
}

export function ConnectionList({
  connections,
  onRefresh,
  isLoading,
  onReconnect,
  onDisconnect,
}: ConnectionListProps) {
  const connectedCount = connections.filter((c) => c.status === 'connected').length;
  const disconnectedCount = connections.filter((c) => c.status === 'disconnected').length;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Connection Cache</CardTitle>
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
          <Button variant="outline" size="sm" onClick={() => onReconnect?.({ id: 'all', remote: '', node: '', endpoint: '', status: 'disconnected' })}>
            Reconnect All
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Remote</TableHead>
                <TableHead>Node</TableHead>
                <TableHead>Endpoint</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Last Connected</TableHead>
                <TableHead>Latency</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {connections.map((conn) => (
                <TableRow key={conn.id}>
                  <TableCell className="font-medium">{conn.remote}</TableCell>
                  <TableCell>{conn.node}</TableCell>
                  <TableCell>{conn.endpoint}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      conn.status === 'connected' ? 'bg-green-100 text-green-800' :
                      conn.status === 'connecting' ? 'bg-yellow-100 text-yellow-800' :
                      conn.status === 'error' ? 'bg-red-100 text-red-800' :
                      'bg-gray-100 text-gray-800'
                    }`}>
                      {conn.status}
                    </span>
                  </TableCell>
                  <TableCell>{conn.lastConnected || '-'}</TableCell>
                  <TableCell>{conn.latency ? `${conn.latency}ms` : '-'}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onReconnect?.(conn)}
                        title="Reconnect"
                      >
                        <span className="h-4 w-4 text-xs">🔄</span>
                      </button>
                      {conn.status === 'connected' && (
                        <button
                          className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                          onClick={() => onDisconnect?.(conn)}
                          title="Disconnect"
                        >
                          <span className="h-4 w-4 text-xs">🔌</span>
                        </button>
                      )}
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
