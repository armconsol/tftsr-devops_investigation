import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal } from 'lucide-react';

interface FirewallRuleInfo {
  id: string;
  rule: number;
  action: string;
  protocol: string;
  source: string;
  destination: string;
  port?: string;
  status: string;
}

interface FirewallRuleListProps {
  rules: FirewallRuleInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onNewRule?: () => void;
  onEnable?: (rule: FirewallRuleInfo) => void;
  onDisable?: (rule: FirewallRuleInfo) => void;
  onEdit?: (rule: FirewallRuleInfo) => void;
  onDelete?: (rule: FirewallRuleInfo) => void;
  onMove?: (rule: FirewallRuleInfo, direction: 'up' | 'down') => void;
}

export function FirewallRuleList({
  rules,
  onRefresh,
  isLoading,
  onNewRule,
  onEnable,
  onDisable,
  onEdit,
  onDelete,
  onMove,
}: FirewallRuleListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Firewall Rules</CardTitle>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          <Button size="sm" onClick={onNewRule}>
            <span className="mr-2 h-4 w-4">+</span>
            New Rule
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Rule #</TableHead>
                <TableHead>Action</TableHead>
                <TableHead>Protocol</TableHead>
                <TableHead>Source</TableHead>
                <TableHead>Destination</TableHead>
                <TableHead>Port</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {rules.map((rule) => (
                <TableRow key={rule.id}>
                  <TableCell className="font-medium">{rule.rule}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      rule.action === 'ACCEPT' ? 'bg-green-100 text-green-800' :
                      rule.action === 'DROP' ? 'bg-red-100 text-red-800' :
                      'bg-yellow-100 text-yellow-800'
                    }`}>
                      {rule.action}
                    </span>
                  </TableCell>
                  <TableCell>{rule.protocol}</TableCell>
                  <TableCell>{rule.source}</TableCell>
                  <TableCell>{rule.destination}</TableCell>
                  <TableCell>{rule.port || '-'}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      rule.status === 'enabled' ? 'bg-green-100 text-green-800' :
                      'bg-gray-100 text-gray-800'
                    }`}>
                      {rule.status}
                    </span>
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onMove?.(rule, 'up')}
                        title="Move Up"
                      >
                        <span className="h-4 w-4 text-xs">⬆️</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onMove?.(rule, 'down')}
                        title="Move Down"
                      >
                        <span className="h-4 w-4 text-xs">⬇️</span>
                      </button>
                      {rule.status === 'enabled' ? (
                        <button
                          className="rounded-md p-1 hover:bg-accent"
                          onClick={() => onDisable?.(rule)}
                          title="Disable"
                        >
                          <span className="h-4 w-4 text-xs">⏸️</span>
                        </button>
                      ) : (
                        <button
                          className="rounded-md p-1 hover:bg-accent"
                          onClick={() => onEnable?.(rule)}
                          title="Enable"
                        >
                          <span className="h-4 w-4 text-xs">▶️</span>
                        </button>
                      )}
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onEdit?.(rule)}
                        title="Edit"
                      >
                        <span className="h-4 w-4 text-xs">✏️</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onDelete?.(rule)}
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
