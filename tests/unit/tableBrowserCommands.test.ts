import { describe, expect, it, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import * as tauriCommands from '@/lib/tauriCommands';

vi.mock('@tauri-apps/api/core');

type MockedInvoke = typeof invoke & {
  mockResolvedValue: (value: unknown) => void;
};

describe('table browser commands', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('sends camelCase payloads for table browsing commands', async () => {
    (invoke as MockedInvoke).mockResolvedValue({
      rows: [],
      total_count: 0,
      page_number: 0,
      page_size: 50,
      total_pages: 0,
    });

    await tauriCommands.browseTableDataCmd({
      connectionId: 'conn-1',
      database: 'main',
      table: 'users',
    });

    expect(invoke).toHaveBeenCalledWith('browse_table_data', {
      connectionId: 'conn-1',
      database: 'main',
      table: 'users',
      pagination: undefined,
      sort: undefined,
      filters: undefined,
    });
  });

  it('sends camelCase payloads for table row counts', async () => {
    (invoke as MockedInvoke).mockResolvedValue(2);

    await tauriCommands.getTableRowCountCmd('conn-1', 'main', 'users');

    expect(invoke).toHaveBeenCalledWith('get_table_row_count', {
      connectionId: 'conn-1',
      database: 'main',
      table: 'users',
    });
  });

  it('sends camelCase payloads for table metadata', async () => {
    (invoke as MockedInvoke).mockResolvedValue({
      table_name: 'users',
      row_count: 2,
      primary_key: 'id',
      estimated_size_bytes: null,
      columns: [],
    });

    await tauriCommands.getTableMetadataCmd('conn-1', 'main', 'users');

    expect(invoke).toHaveBeenCalledWith('get_table_metadata', {
      connectionId: 'conn-1',
      database: 'main',
      table: 'users',
    });
  });

  it('sends camelCase payloads for row mutations', async () => {
    (invoke as MockedInvoke).mockResolvedValue({
      values: {},
    });

    await tauriCommands.insertTableRowCmd('conn-1', 'main', 'users', { values: {} });
    expect(invoke).toHaveBeenCalledWith('insert_table_row', {
      connectionId: 'conn-1',
      database: 'main',
      table: 'users',
      rowData: { values: {} },
    });

    vi.mocked(invoke).mockResolvedValueOnce({
      values: {},
    });
    await tauriCommands.updateTableRowCmd(
      'conn-1',
      'main',
      'users',
      'id',
      { type: 'Integer', value: 1 },
      { values: { name: { type: 'String', value: 'alice' } } }
    );
    expect(invoke).toHaveBeenCalledWith('update_table_row', {
      connectionId: 'conn-1',
      database: 'main',
      table: 'users',
      primaryKeyCol: 'id',
      primaryKeyValue: { type: 'Integer', value: 1 },
      rowData: { values: { name: { type: 'String', value: 'alice' } } },
    });

    vi.mocked(invoke).mockResolvedValueOnce(undefined);
    await tauriCommands.deleteTableRowCmd(
      'conn-1',
      'main',
      'users',
      'id',
      { type: 'Integer', value: 1 }
    );
    expect(invoke).toHaveBeenCalledWith('delete_table_row', {
      connectionId: 'conn-1',
      database: 'main',
      table: 'users',
      primaryKeyCol: 'id',
      primaryKeyValue: { type: 'Integer', value: 1 },
    });
  });
});
