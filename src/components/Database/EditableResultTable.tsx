// Editable Virtual Table for Query Results — supports inline CRUD editing

import { useMemo, useState, useCallback } from 'react';
import { Button } from '@/components/ui';
import { Save, X, AlertCircle } from 'lucide-react';
import { toast } from 'sonner';
import type { QueryResult } from '@/lib/tauriCommands';
import { updateTableRowsCmd } from '@/lib/tauriCommands';

interface EditableResultTableProps {
  result: QueryResult;
  height?: number;
  connectionId: string | null;
  tableName?: string;
  database?: string;
  onRefresh?: () => void;
}

const ROW_HEIGHT = 35;
const HEADER_HEIGHT = 40;

type ChangesMap = Map<number, Map<string, unknown>>;

export function EditableResultTable({
  result,
  height = 400,
  connectionId,
  tableName,
  database,
  onRefresh,
}: EditableResultTableProps) {
  const [changes, setChanges] = useState<ChangesMap>(new Map());
  const [editingCell, setEditingCell] = useState<{ row: number; col: number } | null>(null);
  const [editValue, setEditValue] = useState<string>('');
  const [saving, setSaving] = useState(false);

  const columnWidths = useMemo(() => {
    const widths: Record<string, number> = {};
    result.columns.forEach((col, idx) => {
      const maxContentWidth = Math.max(
        col.name.length * 8,
        ...result.rows.slice(0, 100).map((row) => {
          const cellValue = row[idx];
          return String(cellValue ?? '').length * 7;
        })
      );
      widths[col.name] = Math.min(Math.max(maxContentWidth, 100), 300);
    });
    return widths;
  }, [result]);

  const pkColumns = useMemo(
    () => result.columns.filter((c) => c.primary_key).map((c) => c.name),
    [result.columns]
  );

  const hasChanges = changes.size > 0;
  const canEdit = Boolean(connectionId && tableName && pkColumns.length > 0);

  const startEdit = (rowIndex: number, colIndex: number) => {
    if (!canEdit) {
      toast.warning(
        pkColumns.length === 0
          ? 'Cannot edit: table has no primary key'
          : 'Cannot edit: missing table context'
      );
      return;
    }
    const colName = result.columns[colIndex].name;
    const existingChange = changes.get(rowIndex)?.get(colName);
    const currentValue = existingChange !== undefined ? existingChange : result.rows[rowIndex][colIndex];
    setEditingCell({ row: rowIndex, col: colIndex });
    setEditValue(currentValue === null ? '' : String(currentValue));
  };

  const commitEdit = useCallback(() => {
    if (!editingCell) return;
    const colName = result.columns[editingCell.col].name;
    const originalValue = result.rows[editingCell.row][editingCell.col];
    const originalStr = originalValue === null || originalValue === undefined ? '' : String(originalValue);

    const newChanges = new Map(changes);
    if (editValue === originalStr) {
      // No change relative to original
      const rowMap = newChanges.get(editingCell.row);
      if (rowMap) {
        rowMap.delete(colName);
        if (rowMap.size === 0) newChanges.delete(editingCell.row);
      }
    } else {
      const rowMap = new Map(newChanges.get(editingCell.row) || new Map());
      rowMap.set(colName, editValue);
      newChanges.set(editingCell.row, rowMap);
    }
    setChanges(newChanges);
    setEditingCell(null);
    setEditValue('');
  }, [editingCell, editValue, changes, result]);

  const cancelEdit = () => {
    setEditingCell(null);
    setEditValue('');
  };

  const discardAll = () => {
    setChanges(new Map());
    toast.info('Changes discarded');
  };

  const coerceValue = (value: string, dataType: string): unknown => {
    const lower = dataType.toLowerCase();
    if (value === '' || value.toUpperCase() === 'NULL') return null;
    if (lower.includes('int') || lower.includes('serial')) {
      const n = parseInt(value, 10);
      return isNaN(n) ? value : n;
    }
    if (
      lower.includes('float') ||
      lower.includes('double') ||
      lower.includes('numeric') ||
      lower.includes('decimal') ||
      lower.includes('real')
    ) {
      const n = parseFloat(value);
      return isNaN(n) ? value : n;
    }
    if (lower.includes('bool') || lower.includes('bit')) {
      const v = value.toLowerCase();
      if (v === 'true' || v === '1' || v === 't') return true;
      if (v === 'false' || v === '0' || v === 'f') return false;
    }
    return value;
  };

  const saveChanges = async () => {
    if (!connectionId || !tableName) {
      toast.error('Missing connection or table context');
      return;
    }
    if (changes.size === 0) {
      toast.info('No changes to save');
      return;
    }

    const updates = Array.from(changes.entries()).map(([rowIndex, colMap]) => {
      const row = result.rows[rowIndex];
      const primary_keys: Record<string, unknown> = {};
      for (const pk of pkColumns) {
        const colIdx = result.columns.findIndex((c) => c.name === pk);
        if (colIdx >= 0) primary_keys[pk] = row[colIdx];
      }
      const column_updates: Record<string, unknown> = {};
      for (const [colName, newValue] of colMap.entries()) {
        const col = result.columns.find((c) => c.name === colName);
        const dataType = col?.data_type || 'TEXT';
        column_updates[colName] = coerceValue(String(newValue), dataType);
      }
      return { primary_keys, column_updates };
    });

    setSaving(true);
    try {
      const res = await updateTableRowsCmd(connectionId, database || '', tableName, updates);
      toast.success(`Saved: ${res.rows_updated} updated, ${res.rows_failed} failed`);
      setChanges(new Map());
      if (onRefresh) onRefresh();
    } catch (error) {
      toast.error('Save failed: ' + String(error));
    } finally {
      setSaving(false);
    }
  };

  const Row = ({ index }: { index: number }) => {
    const row = result.rows[index];
    const rowChanges = changes.get(index);

    return (
      <div
        key={index}
        style={{
          display: 'flex',
          borderBottom: '1px solid var(--border)',
          backgroundColor: index % 2 === 0 ? 'transparent' : 'var(--muted)',
          height: ROW_HEIGHT,
        }}
      >
        {result.columns.map((col, colIndex) => {
          const isEditing = editingCell?.row === index && editingCell?.col === colIndex;
          const changedValue = rowChanges?.get(col.name);
          const isChanged = changedValue !== undefined;
          const displayValue = isChanged ? changedValue : row[colIndex];

          return (
            <div
              key={colIndex}
              onDoubleClick={() => !col.primary_key && startEdit(index, colIndex)}
              style={{
                width: columnWidths[col.name],
                padding: '4px 8px',
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
                fontSize: '13px',
                backgroundColor: isChanged ? 'rgba(250, 204, 21, 0.25)' : undefined,
                cursor: !col.primary_key && canEdit ? 'cell' : 'default',
              }}
              title={col.primary_key ? 'Primary key (read-only)' : String(displayValue ?? '')}
            >
              {isEditing ? (
                <input
                  autoFocus
                  type="text"
                  value={editValue}
                  onChange={(e) => setEditValue(e.target.value)}
                  onBlur={commitEdit}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') commitEdit();
                    else if (e.key === 'Escape') cancelEdit();
                  }}
                  style={{
                    width: '100%',
                    background: 'var(--background)',
                    border: '1px solid var(--primary)',
                    padding: '2px 4px',
                    fontSize: '13px',
                    outline: 'none',
                  }}
                />
              ) : (
                formatCellValue(displayValue)
              )}
            </div>
          );
        })}
      </div>
    );
  };

  return (
    <div className="flex flex-col">
      {/* Edit Toolbar */}
      <div className="flex items-center justify-between p-2 border-b bg-muted/40">
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          {!canEdit && (
            <span className="flex items-center gap-1 text-amber-600">
              <AlertCircle className="w-3 h-3" />
              {pkColumns.length === 0
                ? 'Table has no primary key — editing disabled'
                : 'Editing disabled (missing table context)'}
            </span>
          )}
          {canEdit && (
            <span>Double-click a cell to edit. {hasChanges && `${changes.size} row(s) modified.`}</span>
          )}
        </div>
        <div className="flex gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={discardAll}
            disabled={!hasChanges || saving}
          >
            <X className="w-3 h-3 mr-1" />
            Discard
          </Button>
          <Button
            size="sm"
            onClick={saveChanges}
            disabled={!hasChanges || saving || !canEdit}
          >
            <Save className="w-3 h-3 mr-1" />
            {saving ? 'Saving...' : 'Save Changes'}
          </Button>
        </div>
      </div>

      <div className="border rounded-lg overflow-hidden">
        {/* Header */}
        <div
          style={{
            display: 'flex',
            height: HEADER_HEIGHT,
            backgroundColor: 'var(--muted)',
            borderBottom: '2px solid var(--border)',
            fontWeight: 600,
          }}
        >
          {result.columns.map((col, index) => (
            <div
              key={index}
              style={{
                width: columnWidths[col.name],
                padding: '10px 8px',
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
                fontSize: '13px',
              }}
              title={`${col.name} (${col.data_type})`}
            >
              {col.name}
              {col.primary_key && ' 🔑'}
            </div>
          ))}
        </div>

        {/* Rows */}
        <div
          style={{
            height: Math.max(height - HEADER_HEIGHT, 100),
            overflow: 'auto',
          }}
        >
          {result.rows.map((_, index) => (
            <Row key={index} index={index} />
          ))}
        </div>

        {/* Footer */}
        <div
          className="p-2 text-sm text-muted-foreground border-t"
          style={{ backgroundColor: 'var(--muted)' }}
        >
          {result.total_rows} rows • {result.execution_time_ms}ms
        </div>
      </div>
    </div>
  );
}

function formatCellValue(value: unknown): string {
  if (value === null || value === undefined) return 'NULL';
  if (typeof value === 'boolean') return value ? 'true' : 'false';
  if (typeof value === 'object') return JSON.stringify(value);
  return String(value);
}
