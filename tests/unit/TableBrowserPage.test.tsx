import { describe, expect, it, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { TableBrowserPage } from '../../src/pages/Database/TableBrowserPage';

const mocks = vi.hoisted(() => ({
  listDatabaseConnectionsCmd: vi.fn(),
  getDatabasesCmd: vi.fn(),
  getTablesCmd: vi.fn(),
  getTableMetadataCmd: vi.fn(),
  browseTableDataCmd: vi.fn(),
}));

vi.mock('../../src/lib/tauriCommands', () => ({
  listDatabaseConnectionsCmd: mocks.listDatabaseConnectionsCmd,
  getDatabasesCmd: mocks.getDatabasesCmd,
  getTablesCmd: mocks.getTablesCmd,
  getTableMetadataCmd: mocks.getTableMetadataCmd,
  browseTableDataCmd: mocks.browseTableDataCmd,
}));

describe('TableBrowserPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    mocks.listDatabaseConnectionsCmd.mockResolvedValue([
      {
        id: 'conn-1',
        name: 'Local Postgres',
        db_type: 'postgres',
        host: 'localhost',
        port: 5432,
        username: 'postgres',
        ssl_enabled: false,
        created_at: '2026-07-03T00:00:00Z',
        updated_at: '2026-07-03T00:00:00Z',
      },
    ]);

    mocks.getDatabasesCmd.mockResolvedValue(['main']);
    mocks.getTablesCmd.mockResolvedValue(['users']);
    mocks.getTableMetadataCmd.mockResolvedValue({
      table_name: 'users',
      row_count: 1,
      primary_key: 'id',
      estimated_size_bytes: null,
      columns: [
        { name: 'id', data_type: 'INTEGER', nullable: false, primary_key: true },
        { name: 'name', data_type: 'TEXT', nullable: false, primary_key: false },
      ],
    });
    mocks.browseTableDataCmd.mockResolvedValue({
      rows: [
        {
          id: { type: 'Integer', value: 1 },
          name: { type: 'String', value: 'alice' },
        },
      ],
      total_count: 1,
      page_number: 0,
      page_size: 25,
      total_pages: 1,
    });
  });

  it('loads a table browser without requiring SQL', async () => {
    render(<TableBrowserPage />);

    await waitFor(() => expect(mocks.listDatabaseConnectionsCmd).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(mocks.getTablesCmd).toHaveBeenCalledWith('conn-1', 'main'));

    expect(await screen.findByRole('heading', { name: 'users' })).toBeInTheDocument();
    expect(await screen.findByText('alice')).toBeInTheDocument();
  });
});
