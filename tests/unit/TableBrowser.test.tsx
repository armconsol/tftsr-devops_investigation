import { describe, expect, it, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TableBrowser } from '../../src/components/Database/TableBrowser';

const mocks = vi.hoisted(() => ({
  getTableMetadataCmd: vi.fn(),
  browseTableDataCmd: vi.fn(),
  insertTableRowCmd: vi.fn(),
  updateTableRowCmd: vi.fn(),
  deleteTableRowCmd: vi.fn(),
}));

vi.mock('../../src/lib/tauriCommands', () => ({
  getTableMetadataCmd: mocks.getTableMetadataCmd,
  browseTableDataCmd: mocks.browseTableDataCmd,
  insertTableRowCmd: mocks.insertTableRowCmd,
  updateTableRowCmd: mocks.updateTableRowCmd,
  deleteTableRowCmd: mocks.deleteTableRowCmd,
}));

describe('TableBrowser', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    mocks.getTableMetadataCmd.mockResolvedValue({
      table_name: 'users',
      row_count: 2,
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
        {
          id: { type: 'Integer', value: 2 },
          name: { type: 'String', value: 'bob' },
        },
      ],
      total_count: 2,
      page_number: 0,
      page_size: 25,
      total_pages: 1,
    });
  });

  it('loads and displays table data', async () => {
    render(<TableBrowser connectionId="conn-1" database="main" table="users" />);

    await waitFor(() =>
      expect(mocks.getTableMetadataCmd).toHaveBeenCalledWith('conn-1', 'main', 'users')
    );
    await waitFor(() => expect(mocks.browseTableDataCmd).toHaveBeenCalled());

    expect(await screen.findByText('alice')).toBeInTheDocument();
    expect(screen.getByText('bob')).toBeInTheDocument();
    expect(screen.getByText(/PK:\s*id/)).toBeInTheDocument();
  });

  it('sends sort params when a column header is clicked', async () => {
    const user = userEvent.setup();
    render(<TableBrowser connectionId="conn-1" database="main" table="users" />);

    await waitFor(() => expect(mocks.browseTableDataCmd).toHaveBeenCalledTimes(1));

    await user.click(screen.getByTitle('Sort by name'));

    await waitFor(() => {
      expect(mocks.browseTableDataCmd).toHaveBeenLastCalledWith(
        expect.objectContaining({
          sort: { column: 'name', direction: 'ASC' },
        })
      );
    });
  });
});
