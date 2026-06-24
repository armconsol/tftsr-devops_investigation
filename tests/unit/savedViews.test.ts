import { describe, it, expect, beforeEach } from 'vitest';
import {
  listSavedViews,
  createSavedView,
  deleteSavedView,
  slugifyViewName,
} from '@/lib/savedViews';

describe('savedViews', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('returns an empty list when nothing is saved', () => {
    expect(listSavedViews('cluster-a')).toEqual([]);
  });

  it('slugifies names into url-safe ids', () => {
    expect(slugifyViewName('My Prod View!')).toBe('my-prod-view');
  });

  it('creates and lists a view scoped to a cluster', () => {
    const view = createSavedView('cluster-a', { name: 'Prod VMs' });
    expect(view.view_id).toBe('prod-vms');
    expect(view.name).toBe('Prod VMs');
    expect(listSavedViews('cluster-a')).toHaveLength(1);
    // Other clusters are unaffected.
    expect(listSavedViews('cluster-b')).toEqual([]);
  });

  it('generates unique ids when names collide', () => {
    const a = createSavedView('cluster-a', { name: 'Same' });
    const b = createSavedView('cluster-a', { name: 'Same' });
    expect(a.view_id).toBe('same');
    expect(b.view_id).toBe('same-2');
    expect(listSavedViews('cluster-a')).toHaveLength(2);
  });

  it('throws when the name is empty', () => {
    expect(() => createSavedView('cluster-a', { name: '   ' })).toThrow();
  });

  it('deletes a view by id', () => {
    createSavedView('cluster-a', { name: 'Keep' });
    const drop = createSavedView('cluster-a', { name: 'Drop' });
    deleteSavedView('cluster-a', drop.view_id);
    const remaining = listSavedViews('cluster-a');
    expect(remaining).toHaveLength(1);
    expect(remaining[0].name).toBe('Keep');
  });

  it('survives corrupt storage without throwing', () => {
    localStorage.setItem('tftsr-proxmox-views', '{not json');
    expect(listSavedViews('cluster-a')).toEqual([]);
  });

  it('persists across reads (separate calls share storage)', () => {
    createSavedView('cluster-a', { name: 'Persisted', description: 'desc' });
    const reloaded = listSavedViews('cluster-a');
    expect(reloaded[0].description).toBe('desc');
  });
});
