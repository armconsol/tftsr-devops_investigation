// Local persisted "saved views" for Proxmox resources.
//
// Neither Proxmox VE nor the Proxmox Datacenter Manager exposes a server-side
// `config/views` API (PVE returns 501 for it), so saved views are persisted
// locally in the browser/webview via localStorage instead of a dead server
// call. Views are scoped per cluster (datacenter).

export interface SavedView {
  view_id: string;
  name: string;
  description?: string;
}

const STORAGE_KEY = 'tftsr-proxmox-views';

type ViewStore = Record<string, SavedView[]>;

function readStore(): ViewStore {
  try {
    const raw =
      typeof localStorage !== 'undefined'
        ? localStorage.getItem(STORAGE_KEY)
        : null;
    if (!raw) return {};
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
      return parsed as ViewStore;
    }
    return {};
  } catch {
    // Malformed/corrupt storage — start fresh rather than throwing.
    return {};
  }
}

function writeStore(store: ViewStore): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(store));
}

/** Derive a URL-safe id from a view name. */
export function slugifyViewName(name: string): string {
  return name
    .trim()
    .toLowerCase()
    .replace(/\s+/g, '-')
    .replace(/[^a-z0-9-]/g, '');
}

/** List the saved views for a cluster (empty array if none). */
export function listSavedViews(clusterId: string): SavedView[] {
  if (!clusterId) return [];
  const store = readStore();
  return store[clusterId] ?? [];
}

/**
 * Create (or overwrite by id) a saved view for a cluster.
 * Returns the persisted view. Throws if the name is empty.
 */
export function createSavedView(
  clusterId: string,
  input: { name: string; description?: string }
): SavedView {
  const name = input.name.trim();
  if (!clusterId) throw new Error('A cluster must be selected');
  if (!name) throw new Error('View name is required');

  const base = slugifyViewName(name) || 'view';
  const store = readStore();
  const existing = store[clusterId] ?? [];

  // Ensure a unique id within the cluster.
  let viewId = base;
  let suffix = 1;
  while (existing.some((v) => v.view_id === viewId)) {
    suffix += 1;
    viewId = `${base}-${suffix}`;
  }

  const view: SavedView = {
    view_id: viewId,
    name,
    description: input.description?.trim() || undefined,
  };
  store[clusterId] = [...existing, view];
  writeStore(store);
  return view;
}

/** Delete a saved view by id from a cluster. */
export function deleteSavedView(clusterId: string, viewId: string): void {
  if (!clusterId) return;
  const store = readStore();
  const existing = store[clusterId];
  if (!existing) return;
  store[clusterId] = existing.filter((v) => v.view_id !== viewId);
  writeStore(store);
}
