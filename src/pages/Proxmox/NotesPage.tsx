import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Textarea } from '@/components/ui/index';
import { Edit, Save, X } from 'lucide-react';
import { getClusterNotes, updateClusterNotes, listProxmoxClusters } from '@/lib/proxmoxClient';

export function ProxmoxNotesPage() {
  const [notes, setNotes] = useState('');
  const [editMode, setEditMode] = useState(false);
  const [draft, setDraft] = useState('');
  const [clusterId, setClusterId] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const init = async () => {
      try {
        const clusters = await listProxmoxClusters();
        if (clusters.length > 0) {
          setClusterId(clusters[0].id);
          const n = await getClusterNotes(clusters[0].id);
          setNotes(n);
        }
      } catch (e) {
        setError(String(e));
      }
    };
    void init();
  }, []);

  const handleEdit = () => {
    setDraft(notes);
    setEditMode(true);
  };

  const handleCancel = () => setEditMode(false);

  const handleSave = async () => {
    setSaving(true);
    try {
      await updateClusterNotes(clusterId, draft);
      setNotes(draft);
      setEditMode(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Notes</h1>
          <p className="text-muted-foreground">Cluster notes and documentation</p>
        </div>
        {!editMode ? (
          <Button variant="outline" onClick={handleEdit}>
            <Edit className="mr-2 h-4 w-4" />
            Edit
          </Button>
        ) : (
          <div className="flex space-x-2">
            <Button variant="outline" onClick={handleCancel}>
              <X className="mr-2 h-4 w-4" />
              Cancel
            </Button>
            <Button onClick={() => void handleSave()} disabled={saving}>
              <Save className="mr-2 h-4 w-4" />
              {saving ? 'Saving...' : 'Save'}
            </Button>
          </div>
        )}
      </div>

      {error && <div className="text-destructive text-sm">{error}</div>}

      <Card>
        <CardHeader>
          <CardTitle>Cluster Notes</CardTitle>
        </CardHeader>
        <CardContent>
          {!editMode ? (
            <pre className="whitespace-pre-wrap text-sm font-mono min-h-[200px]">
              {notes || (
                <span className="text-muted-foreground">
                  No notes yet. Click Edit to add notes.
                </span>
              )}
            </pre>
          ) : (
            <Textarea
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              className="min-h-[300px] font-mono text-sm"
              placeholder="Enter cluster notes here..."
            />
          )}
        </CardContent>
      </Card>
    </div>
  );
}
