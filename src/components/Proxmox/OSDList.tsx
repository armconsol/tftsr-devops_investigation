import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { formatBytes } from '@/lib/format';

interface OSDInfo {
  id: number;
  host: string;
  status: 'up' | 'down';
  weight: number;
  size: number;
  used: number;
  avail: number;
  usedPercent: number;
}

interface OSDListProps {
  osds: OSDInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onSetWeight?: (osd: OSDInfo) => void;
  onMarkIn?: (osd: OSDInfo) => void;
  onMarkOut?: (osd: OSDInfo) => void;
  onZap?: (osd: OSDInfo) => void;
}

export function OSDList({
  osds,
  onRefresh,
  isLoading,
  onSetWeight,
  onMarkIn,
  onMarkOut,
  onZap,
}: OSDListProps) {
  const upCount = osds.filter((o) => o.status === 'up').length;
  const downCount = osds.filter((o) => o.status === 'down').length;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Ceph OSDs</CardTitle>
        <div className="flex space-x-2">
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-green-500">●</span>
            <span>{upCount} Up</span>
          </div>
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-red-500">●</span>
            <span>{downCount} Down</span>
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
                <TableHead>ID</TableHead>
                <TableHead>Host</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Weight</TableHead>
                <TableHead>Size</TableHead>
                <TableHead>Used</TableHead>
                <TableHead>Avail</TableHead>
                <TableHead>% Used</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {osds.map((osd) => (
                <TableRow key={osd.id}>
                  <TableCell className="font-medium">osd.{osd.id}</TableCell>
                  <TableCell>{osd.host}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      osd.status === 'up' ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                    }`}>
                      {osd.status}
                    </span>
                  </TableCell>
                  <TableCell>{osd.weight}</TableCell>
                  <TableCell>{formatBytes(osd.size)}</TableCell>
                  <TableCell>{formatBytes(osd.used)}</TableCell>
                  <TableCell>{formatBytes(osd.avail)}</TableCell>
                  <TableCell>
                    <div className="flex items-center space-x-2">
                      <div className="h-2 w-24 bg-slate-200 rounded-full overflow-hidden">
                        <div
                          className={`h-full rounded-full ${
                            osd.usedPercent > 80 ? 'bg-red-500' :
                            osd.usedPercent > 60 ? 'bg-yellow-500' :
                            'bg-green-500'
                          }`}
                          style={{ width: `${osd.usedPercent}%` }}
                        />
                      </div>
                      <span className="text-xs">{osd.usedPercent.toFixed(1)}%</span>
                    </div>
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onSetWeight?.(osd)}
                        title="Set Weight"
                      >
                        <span className="h-4 w-4 text-xs">⚖️</span>
                      </button>
                      {osd.status === 'down' ? (
                        <button
                          className="rounded-md p-1 hover:bg-green-100 hover:text-green-600"
                          onClick={() => onMarkIn?.(osd)}
                          title="Mark In"
                        >
                          <span className="h-4 w-4 text-xs">▶️</span>
                        </button>
                      ) : (
                        <button
                          className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                          onClick={() => onMarkOut?.(osd)}
                          title="Mark Out"
                        >
                          <span className="h-4 w-4 text-xs">⏹️</span>
                        </button>
                      )}
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onZap?.(osd)}
                        title="Zap (Destroy)"
                      >
                        <span className="h-4 w-4 text-xs">💣</span>
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
