// Database management store

import { create } from 'zustand';

export interface DatabaseConnection {
  id: string;
  name: string;
  db_type: string;
  host: string;
  port: number;
  database_name?: string;
  username: string;
  ssl_enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface QueryResult {
  columns: Array<{
    name: string;
    data_type: string;
    nullable: boolean;
    primary_key: boolean;
  }>;
  rows: any[][];
  total_rows: number;
  execution_time_ms: number;
}

interface DatabaseState {
  // Connections
  connections: DatabaseConnection[];
  activeConnectionId: string | null;

  // Query state
  queryText: string;
  queryResults: QueryResult | null;
  isExecuting: boolean;
  executionError: string | null;

  // UI state
  selectedDatabase: string | null;
  selectedTable: string | null;

  // Actions
  setConnections: (connections: DatabaseConnection[]) => void;
  setActiveConnection: (id: string | null) => void;
  setQueryText: (text: string) => void;
  setQueryResults: (results: QueryResult | null) => void;
  setIsExecuting: (executing: boolean) => void;
  setExecutionError: (error: string | null) => void;
  setSelectedDatabase: (database: string | null) => void;
  setSelectedTable: (table: string | null) => void;
  clearQueryResults: () => void;
}

export const useDatabaseStore = create<DatabaseState>((set) => ({
  // Initial state
  connections: [],
  activeConnectionId: null,
  queryText: '',
  queryResults: null,
  isExecuting: false,
  executionError: null,
  selectedDatabase: null,
  selectedTable: null,

  // Actions
  setConnections: (connections) => set({ connections }),
  setActiveConnection: (id) => set({ activeConnectionId: id }),
  setQueryText: (text) => set({ queryText: text }),
  setQueryResults: (results) => set({ queryResults: results }),
  setIsExecuting: (executing) => set({ isExecuting: executing }),
  setExecutionError: (error) => set({ executionError: error }),
  setSelectedDatabase: (database) => set({ selectedDatabase: database }),
  setSelectedTable: (table) => set({ selectedTable: table }),
  clearQueryResults: () => set({ queryResults: null, executionError: null }),
}));
