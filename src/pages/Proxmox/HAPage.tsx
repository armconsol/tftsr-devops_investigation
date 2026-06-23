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
      toast.error('Failed to load HA groups');
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
      toast.error('Failed to load HA resources');
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
    // Placeholder: edit dialog integration to be wired when dialog component is available
    toast.info(`Edit group: ${group.id}`);
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
        nodes: newGroupNodes,
      });
      toast.success(`HA group "${newGroupId}" created`);
      setCreateGroupOpen(false);
      await loadGroups(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to create HA group: ${err}`);
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
          onRemove={handleRemoveResource}
        />
      </div>

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
