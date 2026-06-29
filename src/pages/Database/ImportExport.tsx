// Data Import/Export Page

import { useState } from 'react';
import { Button, Input, Label, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Upload, Download, FileText, Table } from 'lucide-react';
import { ColumnMapper, type ColumnMapping } from '@/components/Database/ColumnMapper';
import { useDatabaseStore } from '@/stores/databaseStore';
import { importCsvDataCmd, importJsonDataCmd, exportQueryResultsCmd, previewCsvFileCmd, previewJsonFileCmd } from '@/lib/tauriCommands';
import { open, save } from '@tauri-apps/api/dialog';
import { toast } from 'sonner';

interface PreviewData {
  columns: string[];
  rows: any[][];
  totalRows: number;
}

export function ImportExport() {
  const { activeConnectionId, queryResults } = useDatabaseStore();
  const [importFile, setImportFile] = useState<string | null>(null);
  const [targetTable, setTargetTable] = useState('');
  const [importFormat, setImportFormat] = useState<'csv' | 'json'>('csv');
  const [exportFormat, setExportFormat] = useState<'csv' | 'json' | 'sql'>('csv');
  const [preview, setPreview] = useState<PreviewData | null>(null);
  const [columnMappings, setColumnMappings] = useState<ColumnMapping[]>([]);
  const [importing, setImporting] = useState(false);
  const [exporting, setExporting] = useState(false);

  const handleSelectFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: importFormat.toUpperCase(),
            extensions: [importFormat],
          },
        ],
      });

      if (selected && typeof selected === 'string') {
        setImportFile(selected);
        await loadPreview(selected);
      }
    } catch (error) {
      toast.error('Failed to select file: ' + String(error));
    }
  };

  const loadPreview = async (filePath: string) => {
    try {
      if (importFormat === 'csv') {
        const result = await previewCsvFileCmd(filePath, 100);
        setPreview(result);
      } else {
        const result = await previewJsonFileCmd(filePath, 100);
        setPreview(result);
      }
    } catch (error) {
      toast.error('Failed to preview file: ' + String(error));
      setPreview(null);
    }
  };

  const handleImport = async () => {
    if (!activeConnectionId || !importFile || !targetTable) {
      toast.error('Please fill in all required fields');
      return;
    }

    setImporting(true);
    try {
      if (importFormat === 'csv') {
        const result = await importCsvDataCmd(
          activeConnectionId,
          importFile,
          targetTable,
          columnMappings
        );
        toast.success(`Imported ${result.rows_imported} rows`);
      } else {
        const result = await importJsonDataCmd(
          activeConnectionId,
          importFile,
          targetTable,
          columnMappings
        );
        toast.success(`Imported ${result.rows_imported} rows`);
      }

      // Reset form
      setImportFile(null);
      setTargetTable('');
      setPreview(null);
      setColumnMappings([]);
    } catch (error) {
      toast.error('Import failed: ' + String(error));
    } finally {
      setImporting(false);
    }
  };

  const handleExport = async () => {
    if (!queryResults) {
      toast.error('No query results to export');
      return;
    }

    try {
      const filePath = await save({
        filters: [
          {
            name: exportFormat.toUpperCase(),
            extensions: [exportFormat],
          },
        ],
        defaultPath: `export.${exportFormat}`,
      });

      if (filePath) {
        setExporting(true);
        await exportQueryResultsCmd(queryResults, filePath, exportFormat);
        toast.success('Export completed');
      }
    } catch (error) {
      toast.error('Export failed: ' + String(error));
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-6">Import/Export Data</h1>

      <Tabs defaultValue="import">
        <TabsList>
          <TabsTrigger value="import">
            <Upload className="w-4 h-4 mr-2" />
            Import
          </TabsTrigger>
          <TabsTrigger value="export">
            <Download className="w-4 h-4 mr-2" />
            Export
          </TabsTrigger>
        </TabsList>

        <TabsContent value="import" className="space-y-4 mt-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>File Format</Label>
              <Select
                value={importFormat}
                onValueChange={(value: any) => {
                  setImportFormat(value);
                  setImportFile(null);
                  setPreview(null);
                }}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="csv">CSV</SelectItem>
                  <SelectItem value="json">JSON</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label>Target Table</Label>
              <Input
                value={targetTable}
                onChange={(e) => setTargetTable(e.target.value)}
                placeholder="table_name"
              />
            </div>
          </div>

          <div>
            <Button onClick={handleSelectFile} variant="outline">
              <FileText className="w-4 h-4 mr-2" />
              Select {importFormat.toUpperCase()} File
            </Button>
            {importFile && (
              <p className="text-sm text-muted-foreground mt-2">
                Selected: {importFile.split('/').pop()}
              </p>
            )}
          </div>

          {preview && (
            <>
              <div className="border rounded-lg p-4">
                <h3 className="font-semibold mb-2">Preview (first 100 rows)</h3>
                <div className="overflow-x-auto">
                  <table className="w-full text-sm">
                    <thead>
                      <tr className="border-b">
                        {preview.columns.map((col, i) => (
                          <th key={i} className="p-2 text-left font-semibold">
                            {col}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {preview.rows.slice(0, 10).map((row, i) => (
                        <tr key={i} className="border-b">
                          {row.map((cell, j) => (
                            <td key={j} className="p-2">
                              {String(cell)}
                            </td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                <p className="text-sm text-muted-foreground mt-2">
                  Total rows: {preview.totalRows}
                </p>
              </div>

              <div>
                <h3 className="font-semibold mb-2">Column Mapping</h3>
                <ColumnMapper
                  sourceColumns={preview.columns}
                  targetColumns={preview.columns.map((col) => ({
                    name: col,
                    data_type: 'text',
                  }))}
                  onMappingsChange={setColumnMappings}
                />
              </div>
            </>
          )}

          <div className="flex justify-end">
            <Button
              onClick={handleImport}
              disabled={!importFile || !targetTable || importing || !activeConnectionId}
            >
              {importing ? 'Importing...' : 'Import Data'}
            </Button>
          </div>
        </TabsContent>

        <TabsContent value="export" className="space-y-4 mt-4">
          <div>
            <Label>Export Format</Label>
            <Select value={exportFormat} onValueChange={(value: any) => setExportFormat(value)}>
              <SelectTrigger className="w-48">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="csv">CSV</SelectItem>
                <SelectItem value="json">JSON</SelectItem>
                <SelectItem value="sql">SQL (INSERT statements)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {queryResults && (
            <div className="border rounded-lg p-4 bg-muted">
              <div className="flex items-center gap-2 mb-2">
                <Table className="w-5 h-5" />
                <span className="font-semibold">Current Query Results</span>
              </div>
              <p className="text-sm text-muted-foreground">
                {queryResults.total_rows} rows • {queryResults.columns.length} columns
              </p>
            </div>
          )}

          <div className="flex justify-end">
            <Button onClick={handleExport} disabled={!queryResults || exporting}>
              <Download className="w-4 h-4 mr-2" />
              {exporting ? 'Exporting...' : `Export as ${exportFormat.toUpperCase()}`}
            </Button>
          </div>

          {!queryResults && (
            <p className="text-sm text-muted-foreground text-center py-8">
              Execute a query first to export results
            </p>
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}
