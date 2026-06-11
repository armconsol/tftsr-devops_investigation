import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal, Trash2 } from 'lucide-react';

interface FirewallRuleInfo {
  ruleNum: number;
  action: string;
  protocol: string;
  source: string;
  destination: string;
  port?: string;
  enabled: boolean;
}

interface FirewallRuleListProps {
  rules: FirewallRuleInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onEdit?: (rule: FirewallRuleInfo) => void;
  onDelete?: (rule: FirewallRuleInfo) => void;
  onEnable?: (rule: FirewallRuleInfo) => void;
  onDisable?: (rule: FirewallRuleInfo) => void;
  onMoveUp?: (rule: FirewallRuleInfo) => void;
  onMoveDown?: (rule: FirewallRuleInfo) => void;
}

export function FirewallRuleList({
  rules,
  onRefresh,
  isLoading,
  onEdit,
  onDelete,
  onEnable,
  onDisable,
  onMoveUp,
  onMoveDown,
}: FirewallRuleListProps) {
  const enabledCount = rules.filter((r) => r.enabled).length;
  const disabledCount = rules.filter((r) => !r.enabled).length;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Firewall Rules</CardTitle>
        <div className="flex space-x-2">
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-green-500">●</span>
            <span>{enabledCount} Enabled</span>
          </div>
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-gray-500">●</span>
            <span>{disabledCount} Disabled</span>
          </div>
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          <Button size="sm">
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
                <TableHead className="w-[40px]">#</TableHead>
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
                <TableRow key={rule.ruleNum}>
                  <TableCell className="font-medium">{rule.ruleNum}</TableCell>
                  <TableCell>{rule.action}</TableCell>
                  <TableCell>{rule.protocol}</TableCell>
                  <TableCell>{rule.source}</TableCell>
                  <TableCell>{rule.destination}</TableCell>
                  <TableCell>{rule.port || '-'}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      rule.enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                    }`}>
                      {rule.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-1">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onMoveUp?.(rule)}
                        title="Move Up"
                      >
                        <span className="h-4 w-4 text-xs">⬆️</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onMoveDown?.(rule)}
                        title="Move Down"
                      >
                        <span className="h-4 w-4 text-xs">⬇️</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onEdit?.(rule)}
                        title="Edit"
                      >
                        <span className="h-4 w-4 text-xs">✏️</span>
                      </button>
                      <button
                        className={`rounded-md p-1 hover:bg-accent ${
                          rule.enabled ? 'text-green-600' : 'text-gray-600'
                        }`}
                        onClick={() => rule.enabled ? onDisable?.(rule) : onEnable?.(rule)}
                        title={rule.enabled ? 'Disable' : 'Enable'}
                      >
                        {rule.enabled ? (
                          <span className="h-4 w-4 text-xs">⏸️</span>
                        ) : (
                          <span className="h-4 w-4 text-xs">▶️</span>
                        )}
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onDelete?.(rule)}
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
