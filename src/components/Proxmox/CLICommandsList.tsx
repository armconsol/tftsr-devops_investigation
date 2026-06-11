import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal } from 'lucide-react';

interface CLICommand {
  id: string;
  name: string;
  category: string;
  description: string;
  example: string;
}

interface CLICommandsListProps {
  commands: CLICommand[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onRun?: (command: CLICommand) => void;
}

export function CLICommandsList({
  commands,
  onRefresh,
  isLoading,
  onRun,
}: CLICommandsListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>CLI Commands</CardTitle>
        <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
          Refresh
        </Button>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Category</TableHead>
                <TableHead>Description</TableHead>
                <TableHead>Example</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {commands.map((cmd) => (
                <TableRow key={cmd.id}>
                  <TableCell className="font-medium">{cmd.name}</TableCell>
                  <TableCell>{cmd.category}</TableCell>
                  <TableCell>{cmd.description}</TableCell>
                  <TableCell className="font-mono text-xs">{cmd.example}</TableCell>
                  <TableCell className="text-right">
                    <button
                      className="rounded-md p-1 hover:bg-accent"
                      onClick={() => onRun?.(cmd)}
                      title="Run"
                    >
                      <span className="h-4 w-4 text-xs">▶️</span>
                    </button>
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
