import React, { useState, useEffect, useCallback } from 'react';
import { RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/index';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { HAGroupsList, HAResourcesList } from '@/components/Proxmox';
import {
  listProxmoxClusters,
  listHaGroups,
  listHaResources,
  createHaGroup,
  updateHaGroup,
  updateHaResource,
  deleteHaGroup,
  enableHaResource,
  deleteHaResource,
  HaGroup,
  HaResource,
} from '@/lib/proxmoxClient';
import { ClusterInfo } from '@/lib/domain';
import { toast } from 'sonner';

export function ProxmoxHAPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [selectedClusterId, setSelectedClusterId] = useState<string>('');
  const [groups, setGroups] = useState<HaGroup[]>([]);
  const [resources, setResources] = useState<HaResource[]>([]);
  const [isLoadingGroups, setIsLoadingGroups] = useState(false);
  const [isLoadingResources, setIsLoadingResources] = useState(false);
  const [createGroupOpen, setCreateGroupOpen] = useState(false);
  const [newGroupId, setNewGroupId] = useState('');
  const [newGroupNodes, setNewGroupNodes] = useState('');

  // Edit-group dialog state
  const [editGroupOpen, setEditGroupOpen] = useState(false);
  const [editGroup, setEditGroup] = useState<HaGroup | null>(null);
  const [editGroupNodes, setEditGroupNodes] = useState('');
  const [editGroupComment, setEditGroupComment] = useState('');
  const [editGroupRestricted, setEditGroupRestricted] = useState(false);

  // Edit-resource dialog state
  const [editResourceOpen, setEditResourceOpen] = useState(false);
  const [editResource, setEditResource] = useState<HaResource | null>(null);
  const [editResourceGroup, setEditResourceGroup] = useState('');
  const [editResourceState, setEditResourceState] = useState('started');
  const [editResourceMaxRestart, setEditResourceMaxRestart] = useState('');
  const [editResourceMaxRelocate, setEditResourceMaxRelocate] = useState('');
  const [editResourceComment, setEditResourceComment] = useState('');

  // Load clusters on mount and auto-select the first one
  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        setClusters(cls);
        if (cls.length > 0 && !selectedClusterId) {
          setSelectedClusterId(cls[0].id);
        }
      })
      .catch((err) => {
        console.error('Failed to load clusters:', err);
        toast.error('Failed to load clusters');
      });
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const loadGroups = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setIsLoadingGroups(true);
    try {
      const data = await listHaGroups(clusterId);
      setGroups(data);
    } catch (err) {
      console.error('Failed to load HA groups:', err);
      toast.error(`Failed to load HA groups: ${err}`);
    } finally {
      setIsLoadingGroups(false);
    }
  }, []);

  const loadResources = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setIsLoadingResources(true);
    try {
      const data = await listHaResources(clusterId);
      setResources(data);
    } catch (err) {
      console.error('Failed to load HA resources:', err);
      toast.error(`Failed to load HA resources: ${err}`);
    } finally {
      setIsLoadingResources(false);
    }
  }, []);

  useEffect(() => {
    if (selectedClusterId) {
      loadGroups(selectedClusterId);
      loadResources(selectedClusterId);
    }
  }, [selectedClusterId, loadGroups, loadResources]);

  const handleRefreshAll = () => {
    loadGroups(selectedClusterId);
    loadResources(selectedClusterId);
  };

  const handleDeleteGroup = async (id: string) => {
    try {
      await deleteHaGroup(selectedClusterId, id);
      toast.success(`HA group "${id}" deleted`);
      await loadGroups(selectedClusterId);
    } catch (err) {
      console.error('Failed to delete HA group:', err);
      toast.error('Failed to delete HA group');
    }
  };

  const handleEditGroup = (group: HaGroup) => {
    setEditGroup(group);
    setEditGroupNodes(group.nodes ?? '');
    setEditGroupComment(group.comment ?? '');
    setEditGroupRestricted(Boolean(group.restricted));
    setEditGroupOpen(true);
  };

  const handleEditGroupSubmit = async () => {
    if (!editGroup) return;
    try {
      await updateHaGroup(selectedClusterId, editGroup.id, {
        nodes: editGroupNodes
          .split(',')
          .map((n) => n.trim())
          .filter(Boolean),
        comment: editGroupComment.trim() || undefined,
        restricted: editGroupRestricted,
      });
      toast.success(`HA group "${editGroup.id}" updated`);
      setEditGroupOpen(false);
      await loadGroups(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to update HA group: ${err}`);
    }
  };

  const handleCreateGroup = () => {
    setNewGroupId('');
    setNewGroupNodes('');
    setCreateGroupOpen(true);
  };

  const handleCreateGroupSubmit = async () => {
    if (!newGroupId.trim()) { toast.error('Group ID is required'); return; }
    try {
      await createHaGroup(selectedClusterId, {
        id: newGroupId.trim(),
        nodes: newGroupNodes
          .split(',')
          .map((n) => n.trim())
          .filter(Boolean),
      });
      toast.success(`HA group "${newGroupId}" created`);
      setCreateGroupOpen(false);
      await loadGroups(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to create HA group: ${err}`);
    }
  };

  const handleEditResource = (resource: HaResource) => {
    setEditResource(resource);
    setEditResourceGroup(resource.group ?? '');
    setEditResourceState(resource.state || 'started');
    setEditResourceMaxRestart(
      resource.maxRestart != null ? String(resource.maxRestart) : ''
    );
    setEditResourceMaxRelocate(
      resource.maxRelocate != null ? String(resource.maxRelocate) : ''
    );
    setEditResourceComment(resource.comment ?? '');
    setEditResourceOpen(true);
  };

  const handleEditResourceSubmit = async () => {
    if (!editResource) return;
    const parseNum = (s: string): number | undefined => {
      const t = s.trim();
      if (!t) return undefined;
      const n = Number(t);
      return Number.isFinite(n) ? n : undefined;
    };
    try {
      await updateHaResource(selectedClusterId, editResource.sid, {
        group: editResourceGroup.trim() || undefined,
        state: editResourceState,
        maxRestart: parseNum(editResourceMaxRestart),
        maxRelocate: parseNum(editResourceMaxRelocate),
        comment: editResourceComment.trim() || undefined,
      });
      toast.success(`HA resource "${editResource.sid}" updated`);
      setEditResourceOpen(false);
      await loadResources(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to update HA resource: ${err}`);
    }
  };

  const handleEnableResource = async (resource: HaResource) => {
    try {
      await enableHaResource(selectedClusterId, resource.sid);
      toast.success(`HA resource "${resource.sid}" enabled`);
      await loadResources(selectedClusterId);
    } catch (err) {
      console.error('Failed to enable HA resource:', err);
      toast.error('Failed to enable HA resource');
    }
  };

  const handleRemoveResource = async (resource: HaResource) => {
    try {
      await deleteHaResource(selectedClusterId, resource.sid);
      toast.success(`HA resource "${resource.sid}" removed`);
      await loadResources(selectedClusterId);
    } catch (err) {
      console.error('Failed to remove HA resource:', err);
      toast.error(`Failed to remove HA resource: ${err}`);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">High Availability</h1>
          <p className="text-muted-foreground">Manage HA groups and resources</p>
        </div>
        <div className="flex items-center space-x-2">
          {clusters.length > 1 && (
            <select
              className="rounded-md border px-3 py-1.5 text-sm bg-background"
              value={selectedClusterId}
              onChange={(e) => setSelectedClusterId(e.target.value)}
            >
              {clusters.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name}
                </option>
              ))}
            </select>
          )}
          <Button variant="outline" size="sm" onClick={handleRefreshAll}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4">
        <HAGroupsList
          groups={groups}
          isLoading={isLoadingGroups}
          onRefresh={() => loadGroups(selectedClusterId)}
          onCreate={handleCreateGroup}
          onEdit={handleEditGroup}
          onDelete={handleDeleteGroup}
        />

        <HAResourcesList
          resources={resources}
          isLoading={isLoadingResources}
          onRefresh={() => loadResources(selectedClusterId)}
          onEnable={handleEnableResource}
          onEdit={handleEditResource}
          onRemove={handleRemoveResource}
        />
      </div>

      <Dialog open={editGroupOpen} onOpenChange={setEditGroupOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Edit HA Group{editGroup ? ` — ${editGroup.id}` : ''}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-2">
            <div className="space-y-1">
              <Label>Nodes (comma-separated)</Label>
              <Input
                value={editGroupNodes}
                onChange={(e) => setEditGroupNodes(e.target.value)}
                placeholder="e.g. vmhost1,vmhost2"
              />
            </div>
            <div className="space-y-1">
              <Label>Comment</Label>
              <Input
                value={editGroupComment}
                onChange={(e) => setEditGroupComment(e.target.value)}
                placeholder="Optional comment"
              />
            </div>
            <label className="flex items-center space-x-2 text-sm">
              <input
                type="checkbox"
                checked={editGroupRestricted}
                onChange={(e) => setEditGroupRestricted(e.target.checked)}
              />
              <span>Restricted</span>
            </label>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditGroupOpen(false)}>Cancel</Button>
            <Button onClick={handleEditGroupSubmit}>Save</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={editResourceOpen} onOpenChange={setEditResourceOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Edit HA Resource{editResource ? ` — ${editResource.sid}` : ''}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-2">
            <div className="space-y-1">
              <Label>Group</Label>
              <Input
                value={editResourceGroup}
                onChange={(e) => setEditResourceGroup(e.target.value)}
                placeholder="HA group (optional)"
              />
            </div>
            <div className="space-y-1">
              <Label>Requested State</Label>
              <select
                className="w-full rounded-md border px-3 py-2 text-sm bg-background"
                value={editResourceState}
                onChange={(e) => setEditResourceState(e.target.value)}
              >
                <option value="started">started</option>
                <option value="stopped">stopped</option>
                <option value="disabled">disabled</option>
                <option value="ignored">ignored</option>
              </select>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label>Max Restart</Label>
                <Input
                  type="number"
                  min="0"
                  value={editResourceMaxRestart}
                  onChange={(e) => setEditResourceMaxRestart(e.target.value)}
                  placeholder="e.g. 1"
                />
              </div>
              <div className="space-y-1">
                <Label>Max Relocate</Label>
                <Input
                  type="number"
                  min="0"
                  value={editResourceMaxRelocate}
                  onChange={(e) => setEditResourceMaxRelocate(e.target.value)}
                  placeholder="e.g. 1"
                />
              </div>
            </div>
            <div className="space-y-1">
              <Label>Comment</Label>
              <Input
                value={editResourceComment}
                onChange={(e) => setEditResourceComment(e.target.value)}
                placeholder="Optional comment"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditResourceOpen(false)}>Cancel</Button>
            <Button onClick={handleEditResourceSubmit}>Save</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={createGroupOpen} onOpenChange={setCreateGroupOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create HA Group</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-2">
            <div className="space-y-1">
              <Label>Group ID</Label>
              <Input
                value={newGroupId}
                onChange={(e) => setNewGroupId(e.target.value)}
                placeholder="e.g. ha-group-1"
              />
            </div>
            <div className="space-y-1">
              <Label>Nodes (comma-separated)</Label>
              <Input
                value={newGroupNodes}
                onChange={(e) => setNewGroupNodes(e.target.value)}
                placeholder="e.g. vmhost1,vmhost2"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateGroupOpen(false)}>Cancel</Button>
            <Button onClick={handleCreateGroupSubmit}>Create</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
