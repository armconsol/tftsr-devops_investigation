// Bottom Panel with Results, Messages, History, and Visualization Tabs

import { useMemo, useState } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui';
import { EditableResultTable } from './EditableResultTable';
import { DataVisualization } from './DataVisualization';
import { AlertCircle, CheckCircle, History, BarChart3 } from 'lucide-react';
import type { QueryResult, QueryHistory } from '@/lib/tauriCommands';

interface QueryResultsPanelProps {
  queryResult: QueryResult | null;
  executionError: string | null;
  queryHistory: QueryHistory[];
  onHistoryItemClick: (item: QueryHistory) => void;
  connectionId?: string | null;
  database?: string;
  onRefresh?: () => void;
}

/**
 * Heuristically extract the table name from a simple "SELECT ... FROM table" query.
 * Returns null if the query is complex (joins, subqueries, etc.) — we won't try to edit.
 */
function extractTableName(_history: QueryHistory[]): string | undefined {
  // Currently we rely on the consumer to pass tableName via context. Heuristics
  // could be added here in the future. Returning undefined disables editing.
  return undefined;
}

export function QueryResultsPanel({
  queryResult,
  executionError,
  queryHistory,
  onHistoryItemClick,
  connectionId = null,
  database,
  onRefresh,
}: QueryResultsPanelProps) {
  const detectedTable = useMemo(() => extractTableName(queryHistory), [queryHistory]);
  const [activeView, setActiveView] = useState<string>('results');

  return (
    <Tabs value={activeView} onValueChange={setActiveView}>
      <TabsList>
        <TabsTrigger value="results">
          Results {queryResult && `(${queryResult.total_rows})`}
        </TabsTrigger>
        <TabsTrigger value="messages">
          Messages {executionError && '(1)'}
        </TabsTrigger>
        <TabsTrigger value="history">
          History {queryHistory.length > 0 && `(${queryHistory.length})`}
        </TabsTrigger>
        <TabsTrigger value="visualize">
          <BarChart3 className="w-4 h-4 mr-1" />
          Visualize
        </TabsTrigger>
      </TabsList>

      <TabsContent value="results" className="p-0 flex-1 overflow-auto">
        {queryResult ? (
          <EditableResultTable
            result={queryResult}
            height={300}
            connectionId={connectionId}
            tableName={detectedTable}
            database={database}
            onRefresh={onRefresh}
          />
        ) : (
          <div className="flex items-center justify-center h-64 text-muted-foreground">
            <p>No query results. Execute a query to see results here.</p>
          </div>
        )}
      </TabsContent>

      <TabsContent value="messages" className="p-4 flex-1 overflow-auto">
        {executionError ? (
          <div className="flex items-start gap-2 p-3 bg-red-50 border border-red-200 rounded">
            <AlertCircle className="w-5 h-5 text-red-500 mt-0.5" />
            <div>
              <p className="font-semibold text-red-800">Query Error</p>
              <p className="text-sm text-red-700 mt-1">{executionError}</p>
            </div>
          </div>
        ) : queryResult ? (
          <div className="flex items-start gap-2 p-3 bg-green-50 border border-green-200 rounded">
            <CheckCircle className="w-5 h-5 text-green-500 mt-0.5" />
            <div>
              <p className="font-semibold text-green-800">Query Successful</p>
              <p className="text-sm text-green-700 mt-1">
                {queryResult.total_rows} rows returned in {queryResult.execution_time_ms}ms
              </p>
            </div>
          </div>
        ) : (
          <div className="text-muted-foreground">
            <p>No messages.</p>
          </div>
        )}
      </TabsContent>

      <TabsContent value="history" className="p-4 flex-1 overflow-auto">
        {queryHistory.length > 0 ? (
          <div className="space-y-2 max-h-64 overflow-y-auto">
            {queryHistory.map((item) => (
              <div
                key={item.id}
                className="p-3 border rounded hover:bg-muted cursor-pointer transition-colors"
                onClick={() => onHistoryItemClick(item)}
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <p className="text-sm font-mono truncate">{item.query_text}</p>
                    <div className="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
                      <span>{new Date(item.executed_at).toLocaleString()}</span>
                      <span>
                        {item.status === 'success' ? (
                          <span className="text-green-600">
                            ✓ {item.row_count} rows
                          </span>
                        ) : (
                          <span className="text-red-600">✗ Error</span>
                        )}
                      </span>
                      <span>{item.execution_time_ms}ms</span>
                    </div>
                  </div>
                  <History className="w-4 h-4 text-muted-foreground" />
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-muted-foreground">
            <p>No query history. Your executed queries will appear here.</p>
          </div>
        )}
      </TabsContent>

      <TabsContent value="visualize" className="p-0 flex-1 overflow-auto">
        {queryResult ? (
          <DataVisualization result={queryResult} />
        ) : (
          <div className="flex items-center justify-center h-64 text-muted-foreground">
            <p>No results to visualize.</p>
          </div>
        )}
      </TabsContent>
    </Tabs>
  );
}
