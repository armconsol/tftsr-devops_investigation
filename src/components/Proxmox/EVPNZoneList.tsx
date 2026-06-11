import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal, Trash2 } from 'lucide-react';

interface EVPNZoneInfo {
  id: string;
  name: string;
  type: string;
  fabric: string;
  status: 'available' | 'error' | 'pending' | 'unknown';
  vni?: number;
  routeTarget?: string;
}

interface EVPNZoneListProps {
  zones: EVPNZoneInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onEdit?: (zone: EVPNZoneInfo) => void;
  onDelete?: (zone: EVPNZoneInfo) => void;
}

export function EVPNZoneList({
  zones,
  onRefresh,
  isLoading,
  onEdit,
  onDelete,
}: EVPNZoneListProps) {
  const availableCount = zones.filter((z) => z.status === 'available').length;
  const errorCount = zones.filter((z) => z.status === 'error').length;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>EVPN Zones</CardTitle>
        <div className="flex space-x-2">
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-green-500">●</span>
            <span>{availableCount} Available</span>
          </div>
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-red-500">●</span>
            <span>{errorCount} Errors</span>
          </div>
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          <Button size="sm">
            <span className="mr-2 h-4 w-4">+</span>
            New Zone
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
                <TableHead>Fabric</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>VNI</TableHead>
                <TableHead>Route Target</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {zones.map((zone) => (
                <TableRow key={zone.id}>
                  <TableCell className="font-medium">{zone.name}</TableCell>
                  <TableCell>{zone.type}</TableCell>
                  <TableCell>{zone.fabric}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      zone.status === 'available' ? 'bg-green-100 text-green-800' :
                      zone.status === 'error' ? 'bg-red-100 text-red-800' :
                      'bg-yellow-100 text-yellow-800'
                    }`}>
                      {zone.status}
                    </span>
                  </TableCell>
                  <TableCell>{zone.vni || '-'}</TableCell>
                  <TableCell>{zone.routeTarget || '-'}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onEdit?.(zone)}
                        title="Edit"
                      >
                        <span className="h-4 w-4 text-xs">✏️</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onDelete?.(zone)}
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
