import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Plus, Trash2, Eye } from 'lucide-react';
import {
  listClusterViews,
  createClusterView,
  deleteClusterView,
  listProxmoxClusters,
  ClusterView,
} from '@/lib/proxmoxClient';

export function ProxmoxViewsPage() {
  const [views, setViews] = useState<ClusterView[]>([]);
  const [clusterId, setClusterId] = useState('');
  const [showCreate, setShowCreate] = useState(false);
  const [newViewName, setNewViewName] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [deleting, setDeleting] = useState<string | null>(null);

  const loadViews = useCallback(async (cId: string) => {
    if (!cId) return;
    setError(null);
    try {
      const v = await listClusterViews(cId);
      setViews(v);
    } catch (e) {
      const errorMsg = String(e);
      // Handle 501 Not Implemented error gracefully
      if (errorMsg.includes('501') || errorMsg.includes('Not Implemented')) {
        setError('Cluster views feature is not implemented by this Proxmox server. This is a server-side limitation.');
      } else {
        setError(errorMsg);
      }
    }
  }, []);

  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        if (cls.length > 0) {
          setClusterId(cls[0].id);
          void loadViews(cls[0].id);
        }
      })
      .catch(console.error);
  }, [loadViews]);

  const handleCreate = async () => {
    const trimmed = newViewName.trim();
    if (!trimmed || !clusterId) return;
    setError(null);
    try {
      // Generate a simple ID from the name (lowercase, hyphenated)
      const viewId = trimmed.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');
      await createClusterView(clusterId, viewId, trimmed);
      setNewViewName('');
      setShowCreate(false);
      void loadViews(clusterId);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDelete = async (viewId: string) => {
    if (!clusterId) return;
    setDeleting(viewId);
    setError(null);
    try {
      await deleteClusterView(clusterId, viewId);
      void loadViews(clusterId);
    } catch (e) {
      setError(String(e));
    } finally {
      setDeleting(null);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Views</h1>
          <p className="text-muted-foreground">Custom resource views and dashboards</p>
        </div>
        <Button
          onClick={() => setShowCreate(true)}
          disabled={!clusterId || showCreate}
        >
          <Plus className="mr-2 h-4 w-4" />
          New View
        </Button>
      </div>

      {error && (
        <div className="rounded border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </div>
      )}

      {showCreate && (
        <Card>
          <CardHeader>
            <CardTitle>Create View</CardTitle>
          </CardHeader>
          <CardContent className="flex gap-2">
            <input
              className="flex-1 rounded border bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring"
              placeholder="View name"
              value={newViewName}
              onChange={(e) => setNewViewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') void handleCreate();
                if (e.key === 'Escape') setShowCreate(false);
              }}
              autoFocus
            />
            <Button onClick={() => void handleCreate()} disabled={!newViewName.trim()}>
              Create
            </Button>
            <Button variant="outline" onClick={() => { setShowCreate(false); setNewViewName(''); }}>
              Cancel
            </Button>
          </CardContent>
        </Card>
      )}

      {views.length === 0 && !showCreate ? (
        <Card>
          <CardContent className="pt-4 text-sm text-muted-foreground">
            {clusterId ? 'No custom views configured.' : 'No cluster configured.'}
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-2">
          {views.map((v) => (
            <Card key={v.view_id}>
              <CardContent className="flex items-center justify-between pt-4">
                <div className="flex items-center gap-2">
                  <Eye className="h-4 w-4 shrink-0 text-muted-foreground" />
                  <div>
                    <span className="font-medium">{v.name}</span>
                    {v.description && (
                      <p className="text-xs text-muted-foreground">{v.description}</p>
                    )}
                  </div>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => void handleDelete(v.view_id)}
                  disabled={deleting === v.view_id}
                  title="Delete view"
                >
                  <Trash2 className="h-4 w-4 text-destructive" />
                </Button>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
