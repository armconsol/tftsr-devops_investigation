import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Plus, Trash2, Eye } from 'lucide-react';
import { listProxmoxClusters } from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
import {
  listSavedViews,
  createSavedView,
  deleteSavedView,
  SavedView,
} from '@/lib/savedViews';

export function ProxmoxViewsPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [views, setViews] = useState<SavedView[]>([]);
  const [clusterId, setClusterId] = useState('');
  const [showCreate, setShowCreate] = useState(false);
  const [newViewName, setNewViewName] = useState('');
  const [newViewDescription, setNewViewDescription] = useState('');
  const [error, setError] = useState<string | null>(null);

  const loadViews = useCallback((cId: string) => {
    setError(null);
    setViews(listSavedViews(cId));
  }, []);

  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        setClusters(cls);
        if (cls.length > 0) {
          setClusterId(cls[0].id);
          loadViews(cls[0].id);
        }
      })
      .catch(console.error);
  }, [loadViews]);

  const handleClusterChange = (cId: string) => {
    setClusterId(cId);
    loadViews(cId);
  };

  const handleCreate = () => {
    const trimmed = newViewName.trim();
    if (!trimmed || !clusterId) return;
    setError(null);
    try {
      createSavedView(clusterId, {
        name: trimmed,
        description: newViewDescription.trim() || undefined,
      });
      setNewViewName('');
      setNewViewDescription('');
      setShowCreate(false);
      loadViews(clusterId);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDelete = (viewId: string) => {
    if (!clusterId) return;
    setError(null);
    deleteSavedView(clusterId, viewId);
    loadViews(clusterId);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Views</h1>
          <p className="text-muted-foreground">
            Saved resource views, stored locally per datacenter
          </p>
        </div>
        <div className="flex items-center space-x-2">
          {clusters.length > 1 && (
            <select
              className="rounded-md border px-3 py-1.5 text-sm bg-background"
              value={clusterId}
              onChange={(e) => handleClusterChange(e.target.value)}
            >
              {clusters.map((c) => (
                <option key={c.id} value={c.id}>{c.name}</option>
              ))}
            </select>
          )}
          <Button
            onClick={() => setShowCreate(true)}
            disabled={!clusterId || showCreate}
          >
            <Plus className="mr-2 h-4 w-4" />
            New View
          </Button>
        </div>
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
          <CardContent className="space-y-2">
            <input
              className="w-full rounded border bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring"
              placeholder="View name"
              value={newViewName}
              onChange={(e) => setNewViewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleCreate();
                if (e.key === 'Escape') setShowCreate(false);
              }}
              autoFocus
            />
            <input
              className="w-full rounded border bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring"
              placeholder="Description (optional)"
              value={newViewDescription}
              onChange={(e) => setNewViewDescription(e.target.value)}
            />
            <div className="flex gap-2">
              <Button onClick={handleCreate} disabled={!newViewName.trim()}>
                Create
              </Button>
              <Button
                variant="outline"
                onClick={() => {
                  setShowCreate(false);
                  setNewViewName('');
                  setNewViewDescription('');
                }}
              >
                Cancel
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {views.length === 0 && !showCreate ? (
        <Card>
          <CardContent className="pt-4 text-sm text-muted-foreground">
            {clusterId ? 'No saved views yet. Create one to get started.' : 'No datacenter configured.'}
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
                  onClick={() => handleDelete(v.view_id)}
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
