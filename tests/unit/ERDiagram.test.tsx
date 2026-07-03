import { describe, expect, it, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ERDiagram } from '../../src/pages/Database/ERDiagram';

const mocks = vi.hoisted(() => ({
  getDatabasesCmd: vi.fn(),
  generateErDiagramCmd: vi.fn(),
  toPng: vi.fn(),
}));

vi.mock('../../src/lib/tauriCommands', () => ({
  getDatabasesCmd: mocks.getDatabasesCmd,
  generateErDiagramCmd: mocks.generateErDiagramCmd,
}));

vi.mock('html-to-image', () => ({
  toPng: mocks.toPng,
}));

vi.mock('../../src/stores/databaseStore', () => ({
  useDatabaseStore: () => ({ activeConnectionId: 'conn-1' }),
}));

describe('ERDiagram', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    mocks.getDatabasesCmd.mockResolvedValue(['main']);
    mocks.generateErDiagramCmd.mockResolvedValue({
      tables: [
        {
          name: 'users',
          columns: [
            { name: 'id', data_type: 'INTEGER', primary_key: true },
            { name: 'name', data_type: 'TEXT', primary_key: false },
          ],
          position: { x: 0, y: 0 },
        },
      ],
      relationships: [],
    });
    mocks.toPng.mockResolvedValue('data:image/png;base64,er-diagram');
  });

  it('renders without requiring a React Flow provider', async () => {
    const user = userEvent.setup();
    render(<ERDiagram />);

    await waitFor(() => expect(mocks.getDatabasesCmd).toHaveBeenCalledWith('conn-1'));

    await user.click(screen.getByRole('button', { name: 'Generate' }));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Export' })).toBeEnabled());

    await user.click(screen.getByRole('button', { name: 'Export' }));
    await waitFor(() => expect(mocks.toPng).toHaveBeenCalled());

    const [target, options] = mocks.toPng.mock.calls[mocks.toPng.mock.calls.length - 1] ?? [];
    expect(target).toHaveClass('react-flow__viewport');
    expect(options).toMatchObject({ width: 1024, height: 768 });
  });
});
