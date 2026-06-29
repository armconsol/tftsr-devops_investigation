// Virtual Table for Query Results

import { useMemo } from 'react';
import { FixedSizeList as List } from 'react-window';
import type { QueryResult } from '@/stores/databaseStore';

interface ResultTableProps {
  result: QueryResult;
  height?: number;
}

const ROW_HEIGHT = 35;
const HEADER_HEIGHT = 40;

export function ResultTable({ result, height = 400 }: ResultTableProps) {
  const columnWidths = useMemo(() => {
    const widths: Record<string, number> = {};
    result.columns.forEach((col) => {
      const maxContentWidth = Math.max(
        col.name.length * 8,
        ...result.rows.slice(0, 100).map((row) => {
          const cellValue = row[result.columns.indexOf(col)];
          return String(cellValue).length * 7;
        })
      );
      widths[col.name] = Math.min(Math.max(maxContentWidth, 100), 300);
    });
    return widths;
  }, [result]);

  const totalWidth = useMemo(() => {
    return result.columns.reduce((sum, col) => sum + columnWidths[col.name], 0);
  }, [result.columns, columnWidths]);

  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const row = result.rows[index];

    return (
      <div
        style={{
          ...style,
          display: 'flex',
          borderBottom: '1px solid var(--border)',
          backgroundColor: index % 2 === 0 ? 'transparent' : 'var(--muted)',
        }}
      >
        {result.columns.map((col, colIndex) => (
          <div
            key={colIndex}
            style={{
              width: columnWidths[col.name],
              padding: '8px',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
              fontSize: '13px',
            }}
            title={String(row[colIndex])}
          >
            {formatCellValue(row[colIndex])}
          </div>
        ))}
      </div>
    );
  };

  return (
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
      <List
        height={height - HEADER_HEIGHT}
        itemCount={result.rows.length}
        itemSize={ROW_HEIGHT}
        width="100%"
        style={{ overflowX: 'auto' }}
      >
        {Row}
      </List>

      {/* Footer */}
      <div
        className="p-2 text-sm text-muted-foreground border-t"
        style={{ backgroundColor: 'var(--muted)' }}
      >
        {result.total_rows} rows • {result.execution_time_ms}ms
      </div>
    </div>
  );
}

function formatCellValue(value: any): string {
  if (value === null || value === undefined) {
    return 'NULL';
  }
  if (typeof value === 'boolean') {
    return value ? 'true' : 'false';
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}
