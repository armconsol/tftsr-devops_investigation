import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui';
import { Pencil, Plus, RefreshCw, Trash2 } from 'lucide-react';
import {
  browseTableDataCmd,
  deleteTableRowCmd,
  getTableMetadataCmd,
  insertTableRowCmd,
  updateTableRowCmd,
  type DataValue,
  type SortParams,
  type TableColumnMetadata,
  type TableMetadata,
  type TableRow as BrowserTableRow,
} from '@/lib/tauriCommands';
import { toast } from 'sonner';

interface TableBrowserProps {
  connectionId: string;
  database: string;
  table: string;
}

const PAGE_SIZE_OPTIONS = [10, 25, 50, 100];

export function TableBrowser({ connectionId, database, table }: TableBrowserProps) {
  const [metadata, setMetadata] = useState<TableMetadata | null>(null);
  const [rows, setRows] = useState<BrowserTableRow[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [totalPages, setTotalPages] = useState(0);
  const [pageIndex, setPageIndex] = useState(0);
  const [pageSize, setPageSize] = useState(25);
  const [sort, setSort] = useState<SortParams | undefined>(undefined);
  const [filterText, setFilterText] = useState('');
  const [filterColumn, setFilterColumn] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingRow, setEditingRow] = useState<BrowserTableRow | null>(null);
  const [formValues, setFormValues] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);

  const columns = metadata?.columns || [];
  const primaryKey = metadata?.primary_key;

  const loadMetadata = useCallback(async () => {
    const md = await getTableMetadataCmd(connectionId, database, table);
    setMetadata(md);
    if (md.columns.length > 0) {
      setFilterColumn((prev) => prev || md.columns[0].name);
    } else {
      setFilterColumn(undefined);
    }
  }, [connectionId, database, table]);

  const loadRows = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const filters =
        filterText.trim() && filterColumn
          ? [{ column: filterColumn, operator: 'LIKE' as const, value: filterText.trim() }]
          : undefined;

      const response = await browseTableDataCmd({
        connectionId,
        database,
        table,
        pagination: { limit: pageSize, offset: pageIndex * pageSize },
        sort,
        filters,
      });

      setRows(response.rows);
      setTotalCount(response.total_count);
      setTotalPages(response.total_pages);
    } catch (err) {
      const message = String(err);
      setError(message);
      toast.error('Failed to load table rows: ' + message);
    } finally {
      setLoading(false);
    }
  }, [connectionId, database, table, pageSize, pageIndex, sort, filterText, filterColumn]);

  useEffect(() => {
    void loadMetadata();
  }, [loadMetadata]);

  useEffect(() => {
    void loadRows();
  }, [loadRows]);

  const pageLabel = useMemo(() => {
    if (totalCount === 0) return 'No rows';
    const start = pageIndex * pageSize + 1;
    const end = Math.min((pageIndex + 1) * pageSize, totalCount);
    return `${start}-${end} of ${totalCount}`;
  }, [pageIndex, pageSize, totalCount]);

  const handleSort = (column: string) => {
    setPageIndex(0);
    setSort((prev) => {
      if (!prev || prev.column !== column) {
        return { column, direction: 'ASC' };
      }
      return { column, direction: prev.direction === 'ASC' ? 'DESC' : 'ASC' };
    });
  };

  const openCreateDialog = () => {
    const next: Record<string, string> = {};
    columns.forEach((col) => {
      next[col.name] = '';
    });
    setEditingRow(null);
    setFormValues(next);
    setDialogOpen(true);
  };

  const openEditDialog = (row: BrowserTableRow) => {
    const next: Record<string, string> = {};
    columns.forEach((col) => {
      next[col.name] = dataValueToText(row[col.name]);
    });
    setEditingRow(row);
    setFormValues(next);
    setDialogOpen(true);
  };

  const closeDialog = () => {
    setDialogOpen(false);
    setEditingRow(null);
    setFormValues({});
  };

  const buildRowData = (mode: 'create' | 'edit') => {
    const values: Record<string, DataValue> = {};
    columns.forEach((col) => {
      if (mode === 'edit' && primaryKey && col.name === primaryKey) {
        return;
      }
      values[col.name] = textToDataValue(formValues[col.name] ?? '', col);
    });
    return { values };
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      if (!editingRow) {
        await insertTableRowCmd(connectionId, database, table, buildRowData('create'));
        toast.success('Row inserted');
      } else {
        if (!primaryKey) {
          throw new Error('Cannot update row: primary key not found');
        }
        const pkValue = editingRow[primaryKey];
        if (!pkValue) {
          throw new Error('Cannot update row: primary key value missing');
        }
        await updateTableRowCmd(
          connectionId,
          database,
          table,
          primaryKey,
          pkValue,
          buildRowData('edit')
        );
        toast.success('Row updated');
      }
      closeDialog();
      await loadRows();
    } catch (err) {
      toast.error('Save failed: ' + String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (row: BrowserTableRow) => {
    if (!primaryKey) {
      toast.error('Cannot delete row: table has no primary key');
      return;
    }
    const pkValue = row[primaryKey];
    if (!pkValue) {
      toast.error('Cannot delete row: primary key value missing');
      return;
    }
    if (!confirm('Delete this row?')) {
      return;
    }
    try {
      await deleteTableRowCmd(connectionId, database, table, primaryKey, pkValue);
      toast.success('Row deleted');
      await loadRows();
    } catch (err) {
      toast.error('Delete failed: ' + String(err));
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h2 className="text-xl font-semibold">{table}</h2>
          <p className="text-sm text-muted-foreground">
            {database} • {metadata?.row_count ?? totalCount} rows
            {primaryKey ? ` • PK: ${primaryKey}` : ''}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" onClick={() => void loadRows()} disabled={loading}>
            <RefreshCw className={`w-4 h-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          <Button onClick={openCreateDialog}>
            <Plus className="w-4 h-4 mr-2" />
            Add Row
          </Button>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Select
          value={filterColumn || ''}
          onValueChange={(value) => {
            setFilterColumn(value);
            setPageIndex(0);
          }}
        >
          <SelectTrigger className="w-48">
            <SelectValue placeholder="Filter column" />
          </SelectTrigger>
          <SelectContent>
            {columns.map((col) => (
              <SelectItem key={col.name} value={col.name}>
                {col.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Input
          value={filterText}
          onChange={(e) => {
            setFilterText(e.target.value);
            setPageIndex(0);
          }}
          placeholder="Filter rows (LIKE)"
        />
      </div>

      {error ? (
        <div className="rounded border border-destructive/40 bg-destructive/10 p-3 text-sm text-destructive">
          {error}
        </div>
      ) : null}

      <div className="border rounded-md overflow-auto max-h-[60vh]">
        <Table>
          <TableHeader>
            <TableRow>
              {columns.map((col) => (
                <TableHead
                  key={col.name}
                  className="cursor-pointer select-none"
                  onClick={() => handleSort(col.name)}
                  title={`Sort by ${col.name}`}
                >
                  {col.name}
                  {sort?.column === col.name ? (sort.direction === 'ASC' ? ' ↑' : ' ↓') : ''}
                </TableHead>
              ))}
              <TableHead className="w-[140px]">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {rows.length === 0 ? (
              <TableRow>
                <TableCell colSpan={columns.length + 1} className="text-center text-muted-foreground">
                  {loading ? 'Loading rows...' : 'No rows found'}
                </TableCell>
              </TableRow>
            ) : (
              rows.map((row, idx) => (
                <TableRow key={rowKey(row, primaryKey, idx)}>
                  {columns.map((col) => (
                    <TableCell
                      key={col.name}
                      className="max-w-[280px] truncate"
                      title={dataValueToText(row[col.name] ?? undefined)}
                    >
                      {dataValueToText(row[col.name] ?? undefined)}
                    </TableCell>
                  ))}
                  <TableCell>
                    <div className="flex items-center gap-1">
                      <Button size="sm" variant="outline" onClick={() => openEditDialog(row)}>
                        <Pencil className="w-3 h-3" />
                      </Button>
                      <Button size="sm" variant="destructive" onClick={() => void handleDelete(row)}>
                        <Trash2 className="w-3 h-3" />
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <div className="flex items-center justify-between">
        <span className="text-sm text-muted-foreground">{pageLabel}</span>
        <div className="flex items-center gap-2">
          <Select
            value={String(pageSize)}
            onValueChange={(value) => {
              setPageSize(Number(value));
              setPageIndex(0);
            }}
          >
            <SelectTrigger className="w-24">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {PAGE_SIZE_OPTIONS.map((size) => (
                <SelectItem key={size} value={String(size)}>
                  {size}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button
            variant="outline"
            onClick={() => setPageIndex((prev) => Math.max(prev - 1, 0))}
            disabled={pageIndex === 0 || loading}
          >
            Previous
          </Button>
          <Button
            variant="outline"
            onClick={() => setPageIndex((prev) => prev + 1)}
            disabled={loading || pageIndex + 1 >= Math.max(totalPages, 1)}
          >
            Next
          </Button>
        </div>
      </div>

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>{editingRow ? 'Edit Row' : 'Add Row'}</DialogTitle>
          </DialogHeader>

          <div className="space-y-4 max-h-[60vh] overflow-y-auto pr-1">
            {columns.map((col) => {
              const isPk = primaryKey === col.name;
              return (
                <div key={col.name} className="space-y-1">
                  <Label htmlFor={`col-${col.name}`}>
                    {col.name}
                    {isPk ? ' (primary key)' : ''}
                  </Label>
                  <Input
                    id={`col-${col.name}`}
                    value={formValues[col.name] ?? ''}
                    onChange={(e) =>
                      setFormValues((prev) => ({ ...prev, [col.name]: e.target.value }))
                    }
                    disabled={Boolean(editingRow && isPk)}
                    placeholder={col.data_type}
                  />
                </div>
              );
            })}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={closeDialog} disabled={saving}>
              Cancel
            </Button>
            <Button onClick={() => void handleSave()} disabled={saving}>
              {saving ? 'Saving...' : editingRow ? 'Update Row' : 'Insert Row'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

function rowKey(row: BrowserTableRow, primaryKey: string | undefined, idx: number) {
  if (primaryKey) {
    const value = row[primaryKey];
    if (value !== undefined && value !== null) {
      return `${primaryKey}-${dataValueToText(value)}`;
    }
  }
  return `row-${idx}`;
}

function dataValueToText(value: DataValue | undefined): string {
  if (!value) return '';
  switch (value.type) {
    case 'Null':
      return 'NULL';
    case 'Boolean':
    case 'Integer':
    case 'Float':
      return String(value.value);
    case 'String':
    case 'Date':
    case 'DateTime':
      return value.value;
    case 'Bytes':
      return `<${value.value.length} bytes>`;
    case 'Json':
      return JSON.stringify(value.value);
    case 'Array':
      return `[${value.value.map(dataValueToText).join(', ')}]`;
    default:
      return '';
  }
}

function textToDataValue(input: string, column: TableColumnMetadata): DataValue {
  const trimmed = input.trim();
  if (trimmed === '' || trimmed.toUpperCase() === 'NULL') {
    return { type: 'Null' };
  }

  const type = column.data_type.toLowerCase();
  if (type.includes('int') || type.includes('serial')) {
    const value = Number.parseInt(trimmed, 10);
    if (!Number.isNaN(value)) return { type: 'Integer', value };
  }
  if (
    type.includes('float') ||
    type.includes('double') ||
    type.includes('numeric') ||
    type.includes('decimal') ||
    type.includes('real')
  ) {
    const value = Number.parseFloat(trimmed);
    if (!Number.isNaN(value)) return { type: 'Float', value };
  }
  if (type.includes('bool') || type.includes('bit')) {
    const lower = trimmed.toLowerCase();
    if (lower === 'true' || lower === '1' || lower === 't') return { type: 'Boolean', value: true };
    if (lower === 'false' || lower === '0' || lower === 'f') return { type: 'Boolean', value: false };
  }
  return { type: 'String', value: input };
}
