// WHERE Clause Builder for Visual Query Builder

import { Button, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui';
import { Plus, Trash2 } from 'lucide-react';
import type { Node } from 'reactflow';

export interface WhereCondition {
  id: string;
  tableId: string;
  tableName: string;
  columnName: string;
  operator: string;
  value: string;
}

const OPERATORS = ['=', '!=', '<', '<=', '>', '>=', 'LIKE', 'IN', 'BETWEEN', 'IS NULL', 'IS NOT NULL'];

interface WhereClauseBuilderProps {
  nodes: Node[];
  conditions: WhereCondition[];
  onConditionsChange: (conditions: WhereCondition[]) => void;
}

export function WhereClauseBuilder({ nodes, conditions, onConditionsChange }: WhereClauseBuilderProps) {
  const addCondition = () => {
    const firstNode = nodes[0];
    if (!firstNode) return;
    const firstColumn = firstNode.data.columns?.[0];

    const newCondition: WhereCondition = {
      id: crypto.randomUUID(),
      tableId: firstNode.id,
      tableName: firstNode.data.tableName,
      columnName: firstColumn?.name || '',
      operator: '=',
      value: '',
    };
    onConditionsChange([...conditions, newCondition]);
  };

  const updateCondition = (id: string, updates: Partial<WhereCondition>) => {
    onConditionsChange(
      conditions.map((c) => (c.id === id ? { ...c, ...updates } : c))
    );
  };

  const removeCondition = (id: string) => {
    onConditionsChange(conditions.filter((c) => c.id !== id));
  };

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between mb-2">
        <h3 className="font-semibold text-sm">WHERE Conditions</h3>
        <Button onClick={addCondition} size="sm" variant="outline" disabled={nodes.length === 0}>
          <Plus className="w-3 h-3 mr-1" />
          Add Condition
        </Button>
      </div>

      {conditions.length === 0 ? (
        <div className="text-xs text-muted-foreground p-2">
          No conditions. Click "Add Condition" to filter results.
        </div>
      ) : (
        <div className="space-y-2 max-h-40 overflow-y-auto">
          {conditions.map((cond) => {
            const tableNode = nodes.find((n) => n.id === cond.tableId);
            const columns = tableNode?.data.columns || [];
            const operatorNeedsValue = !['IS NULL', 'IS NOT NULL'].includes(cond.operator);

            return (
              <div key={cond.id} className="flex gap-1 items-center text-xs">
                <Select
                  value={cond.tableId}
                  onValueChange={(v) => {
                    const node = nodes.find((n) => n.id === v);
                    if (node) {
                      updateCondition(cond.id, {
                        tableId: v,
                        tableName: node.data.tableName,
                        columnName: node.data.columns?.[0]?.name || '',
                      });
                    }
                  }}
                >
                  <SelectTrigger className="w-24 h-7 text-xs">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {nodes.map((n) => (
                      <SelectItem key={n.id} value={n.id}>
                        {n.data.alias || n.data.tableName}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>

                <Select
                  value={cond.columnName}
                  onValueChange={(v) => updateCondition(cond.id, { columnName: v })}
                >
                  <SelectTrigger className="w-28 h-7 text-xs">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {columns.map((col: { name: string }) => (
                      <SelectItem key={col.name} value={col.name}>
                        {col.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>

                <Select
                  value={cond.operator}
                  onValueChange={(v) => updateCondition(cond.id, { operator: v })}
                >
                  <SelectTrigger className="w-24 h-7 text-xs">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {OPERATORS.map((op) => (
                      <SelectItem key={op} value={op}>
                        {op}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>

                {operatorNeedsValue && (
                  <Input
                    value={cond.value}
                    onChange={(e) => updateCondition(cond.id, { value: e.target.value })}
                    placeholder="value"
                    className="h-7 text-xs flex-1"
                  />
                )}

                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => removeCondition(cond.id)}
                  className="h-7 w-7 p-0"
                >
                  <Trash2 className="w-3 h-3" />
                </Button>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
