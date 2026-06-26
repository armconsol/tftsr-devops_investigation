import React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Button } from '@/components/ui/index';

interface MonitorInfo {
  name: string;
  host: string;
  address: string;
  status: 'in_quorum' | 'out_of_quorum';
}

interface MonitorListProps {
  monitors: MonitorInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
}

export function MonitorList({
  monitors,
  onRefresh,
  isLoading,
}: MonitorListProps) {
  const inQuorumCount = monitors.filter((m) => m.status === 'in_quorum').length;
  const outOfQuorumCount = monitors.filter((m) => m.status === 'out_of_quorum').length;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Ceph Monitors</CardTitle>
        <div className="flex space-x-2">
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-green-500">●</span>
            <span>{inQuorumCount} In Quorum</span>
          </div>
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-red-500">●</span>
            <span>{outOfQuorumCount} Out of Quorum</span>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={onRefresh}
            disabled={isLoading}
          >
            <span className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`}>↻</span>
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Host</TableHead>
                <TableHead>Address</TableHead>
                <TableHead>Status</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {monitors.map((monitor) => (
                <TableRow key={monitor.name}>
                  <TableCell className="font-medium">{monitor.name}</TableCell>
                  <TableCell>{monitor.host}</TableCell>
                  <TableCell>{monitor.address}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      monitor.status === 'in_quorum' ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                    }`}>
                      {monitor.status === 'in_quorum' ? 'In Quorum' : 'Out of Quorum'}
                    </span>
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
