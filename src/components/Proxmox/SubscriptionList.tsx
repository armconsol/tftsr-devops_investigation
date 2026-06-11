import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal } from 'lucide-react';

interface SubscriptionInfo {
  id: string;
  cluster: string;
  status: 'active' | 'mixed' | 'none' | 'unknown';
  level: string;
  socket: number;
  expiry?: string;
  keyId?: string;
}

interface SubscriptionListProps {
  subscriptions: SubscriptionInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onClearKey?: (sub: SubscriptionInfo) => void;
  onAdopt?: (sub: SubscriptionInfo) => void;
}

export function SubscriptionList({
  subscriptions,
  onRefresh,
  isLoading,
  onClearKey,
  onAdopt,
}: SubscriptionListProps) {
  const activeCount = subscriptions.filter((s) => s.status === 'active').length;
  const noneCount = subscriptions.filter((s) => s.status === 'none').length;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Subscriptions</CardTitle>
        <div className="flex space-x-2">
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-green-500">●</span>
            <span>{activeCount} Active</span>
          </div>
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-red-500">●</span>
            <span>{noneCount} None</span>
          </div>
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          <Button size="sm">
            <span className="mr-2 h-4 w-4">+</span>
            Add Key
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Cluster</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Level</TableHead>
                <TableHead>Socket</TableHead>
                <TableHead>Expiry</TableHead>
                <TableHead>Key ID</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {subscriptions.map((sub) => (
                <TableRow key={sub.id}>
                  <TableCell className="font-medium">{sub.cluster}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      sub.status === 'active' ? 'bg-green-100 text-green-800' :
                      sub.status === 'none' ? 'bg-red-100 text-red-800' :
                      'bg-yellow-100 text-yellow-800'
                    }`}>
                      {sub.status}
                    </span>
                  </TableCell>
                  <TableCell>{sub.level}</TableCell>
                  <TableCell>{sub.socket}</TableCell>
                  <TableCell>{sub.expiry || '-'}</TableCell>
                  <TableCell>{sub.keyId || '-'}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onClearKey?.(sub)}
                        title="Clear Key"
                      >
                        <span className="h-4 w-4 text-xs">🗑️</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onAdopt?.(sub)}
                        title="Adopt from Node"
                      >
                        <span className="h-4 w-4 text-xs">📋</span>
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
