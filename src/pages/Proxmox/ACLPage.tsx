import React, { useState, useEffect, useCallback } from 'react';
import { RefreshCw } from 'lucide-react';
import { Button, Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/index';
import { AclList, UserList, RealmList } from '@/components/Proxmox';
import {
  listProxmoxClusters,
  listAcls,
  listUsers,
  listRealms,
  AclEntry,
  ProxmoxUser,
  AuthRealm,
} from '@/lib/proxmoxClient';
import { ClusterInfo } from '@/lib/domain';
import { toast } from 'sonner';

export function ProxmoxACLPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [selectedClusterId, setSelectedClusterId] = useState<string>('');
  const [activeTab, setActiveTab] = useState<string>('acl');

  const [acls, setAcls] = useState<AclEntry[]>([]);
  const [users, setUsers] = useState<ProxmoxUser[]>([]);
  const [realms, setRealms] = useState<AuthRealm[]>([]);

  const [isLoadingAcls, setIsLoadingAcls] = useState(false);
  const [isLoadingUsers, setIsLoadingUsers] = useState(false);
  const [isLoadingRealms, setIsLoadingRealms] = useState(false);

  // Load clusters on mount, auto-select the first
  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        setClusters(cls);
        if (cls.length > 0) {
          setSelectedClusterId(cls[0].id);
        }
      })
      .catch((err) => {
        console.error('Failed to load clusters:', err);
        toast.error('Failed to load clusters');
      });
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const loadAcls = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setIsLoadingAcls(true);
    try {
      const data = await listAcls(clusterId);
      setAcls(data);
    } catch (err) {
      console.error('Failed to load ACLs:', err);
      toast.error('Failed to load ACLs');
    } finally {
      setIsLoadingAcls(false);
    }
  }, []);

  const loadUsers = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setIsLoadingUsers(true);
    try {
      const data = await listUsers(clusterId);
      setUsers(data);
    } catch (err) {
      console.error('Failed to load users:', err);
      toast.error('Failed to load users');
    } finally {
      setIsLoadingUsers(false);
    }
  }, []);

  const loadRealms = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setIsLoadingRealms(true);
    try {
      const data = await listRealms(clusterId);
      setRealms(data);
    } catch (err) {
      console.error('Failed to load realms:', err);
      toast.error('Failed to load auth realms');
    } finally {
      setIsLoadingRealms(false);
    }
  }, []);

  useEffect(() => {
    if (selectedClusterId) {
      loadAcls(selectedClusterId);
      loadUsers(selectedClusterId);
      loadRealms(selectedClusterId);
    }
  }, [selectedClusterId, loadAcls, loadUsers, loadRealms]);

  const handleRefreshAll = () => {
    loadAcls(selectedClusterId);
    loadUsers(selectedClusterId);
    loadRealms(selectedClusterId);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Access Control &amp; Users</h1>
          <p className="text-muted-foreground">Manage permissions, users, and authentication realms</p>
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

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="acl">ACLs</TabsTrigger>
          <TabsTrigger value="users">Users</TabsTrigger>
          <TabsTrigger value="realms">Auth Realms</TabsTrigger>
        </TabsList>

        <TabsContent value="acl">
          <AclList
            acls={acls}
            isLoading={isLoadingAcls}
            onRefresh={() => loadAcls(selectedClusterId)}
            onAdd={() => toast.info('Add ACL — not yet implemented')}
            onEdit={() => toast.info('Edit ACL — not yet implemented')}
            onDelete={() => toast.info('Delete ACL — not yet implemented')}
          />
        </TabsContent>

        <TabsContent value="users">
          <UserList
            users={users}
            isLoading={isLoadingUsers}
            onRefresh={() => loadUsers(selectedClusterId)}
            onCreate={() => toast.info('Create user — not yet implemented')}
            onEdit={() => toast.info('Edit user — not yet implemented')}
            onDelete={() => toast.info('Delete user — not yet implemented')}
            onEnable={() => toast.info('Enable user — not yet implemented')}
            onDisable={() => toast.info('Disable user — not yet implemented')}
          />
        </TabsContent>

        <TabsContent value="realms">
          <RealmList
            realms={realms}
            isLoading={isLoadingRealms}
            onRefresh={() => loadRealms(selectedClusterId)}
            onCreate={() => toast.info('Create realm — not yet implemented')}
            onEdit={() => toast.info('Edit realm — not yet implemented')}
            onDelete={() => toast.info('Delete realm — not yet implemented')}
          />
        </TabsContent>
      </Tabs>
    </div>
  );
}
