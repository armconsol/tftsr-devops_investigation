import React, { useState, useRef, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal, Plug, PlugZap, TerminalSquare } from 'lucide-react';

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
  onShell?: (remote: RemoteInfo) => void;
}

function ActionsMenu({
  remote,
  onEdit,
  onDelete,
  onConnect,
  onDisconnect,
  onShell,
}: {
  remote: RemoteInfo;
  onEdit?: (remote: RemoteInfo) => void;
  onDelete?: (remote: RemoteInfo) => void;
  onConnect?: (remote: RemoteInfo) => void;
  onDisconnect?: (remote: RemoteInfo) => void;
  onShell?: (remote: RemoteInfo) => void;
}) {
  const [open, setOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  return (
    <div className="relative" ref={menuRef}>
      <button
        className="rounded-md p-1 hover:bg-accent"
        onClick={() => setOpen((v) => !v)}
        title="More actions"
      >
        <MoreHorizontal className="h-4 w-4" />
      </button>
      {open && (
        <div className="absolute right-0 z-50 mt-1 w-44 rounded-md border bg-background shadow-lg">
          <div className="py-1">
            <button
              className="flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent"
              onClick={() => { setOpen(false); onEdit?.(remote); }}
            >
              Edit
            </button>
            <button
              className="flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent"
              onClick={() => {
                setOpen(false);
                if (remote.status === 'connected') {
                  onDisconnect?.(remote);
                } else {
                  onConnect?.(remote);
                }
              }}
            >
              Test Connection
            </button>
            <button
              className="flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent"
              onClick={() => { setOpen(false); onShell?.(remote); }}
            >
              <TerminalSquare className="h-4 w-4" />
              Console (Shell)
            </button>
            <div className="my-1 h-px bg-border" />
            <button
              className="flex w-full items-center gap-2 px-3 py-2 text-sm text-destructive hover:bg-destructive/10"
              onClick={() => { setOpen(false); onDelete?.(remote); }}
            >
              Delete
            </button>
          </div>
        </div>
      )}
    </div>
  );
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
  onShell,
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
                    <div className="flex items-center justify-end space-x-1">
                      {remote.status === 'connected' ? (
                        <button
                          className="rounded-md p-1 hover:bg-red-100 hover:text-red-600 text-green-600"
                          onClick={() => onDisconnect?.(remote)}
                          title="Disconnect"
                        >
                          <PlugZap className="h-4 w-4" />
                        </button>
                      ) : (
                        <button
                          className="rounded-md p-1 hover:bg-green-100 hover:text-green-600 text-muted-foreground"
                          onClick={() => onConnect?.(remote)}
                          title="Test connection"
                        >
                          <Plug className="h-4 w-4" />
                        </button>
                      )}
                      <ActionsMenu
                        remote={remote}
                        onEdit={onEdit}
                        onDelete={onDelete}
                        onConnect={onConnect}
                        onDisconnect={onDisconnect}
                        onShell={onShell}
                      />
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
