import React, { useState, useEffect, useCallback } from 'react';
import { RefreshCw } from 'lucide-react';
import { Button, Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/index';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { AclList, UserList, RealmList } from '@/components/Proxmox';
import {
  listProxmoxClusters,
  listAcls,
  listUsers,
  listRealms,
  createProxmoxAcl,
  deleteProxmoxAcl,
  createProxmoxUser,
  updateProxmoxUser,
  deleteProxmoxUser,
  createProxmoxRealm,
  updateProxmoxRealm,
  deleteProxmoxRealm,
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

  // ACL create dialog
  const [aclDialogOpen, setAclDialogOpen] = useState(false);
  const [aclPath, setAclPath] = useState('/');
  const [aclRole, setAclRole] = useState('');
  const [aclUsers, setAclUsers] = useState('');

  // User create/edit dialog
  const [userDialogOpen, setUserDialogOpen] = useState(false);
  const [userDialogMode, setUserDialogMode] = useState<'create' | 'edit'>('create');
  const [editingUser, setEditingUser] = useState<ProxmoxUser | null>(null);
  const [userForm, setUserForm] = useState({ userid: '', password: '', comment: '', email: '', enabled: true });

  // Realm create/edit dialog
  const [realmDialogOpen, setRealmDialogOpen] = useState(false);
  const [realmDialogMode, setRealmDialogMode] = useState<'create' | 'edit'>('create');
  const [editingRealm, setEditingRealm] = useState<AuthRealm | null>(null);
  const [realmForm, setRealmForm] = useState({ realm: '', realmType: 'pam', comment: '' });

  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        setClusters(cls);
        if (cls.length > 0) setSelectedClusterId(cls[0].id);
      })
      .catch((err) => {
        console.error('Failed to load clusters:', err);
        toast.error('Failed to load clusters');
      });
  }, []);

  const loadAcls = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setIsLoadingAcls(true);
    try {
      setAcls(await listAcls(clusterId));
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
      setUsers(await listUsers(clusterId));
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
      setRealms(await listRealms(clusterId));
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

  // ─── ACL handlers ────────────────────────────────────────────────────────────

  const handleAddAcl = () => {
    setAclPath('/');
    setAclRole('');
    setAclUsers('');
    setAclDialogOpen(true);
  };

  const handleSubmitAcl = async () => {
    if (!aclRole.trim()) { toast.error('Role is required'); return; }
    try {
      await createProxmoxAcl(selectedClusterId, aclPath, aclRole.trim(), aclUsers.trim() || undefined);
      toast.success('ACL entry created');
      setAclDialogOpen(false);
      await loadAcls(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to create ACL: ${err}`);
    }
  };

  const handleDeleteAcl = async (acl: AclEntry) => {
    try {
      await deleteProxmoxAcl(selectedClusterId, acl.path, acl.roleid, acl.ugid);
      toast.success('ACL entry deleted');
      await loadAcls(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to delete ACL: ${err}`);
    }
  };

  // ─── User handlers ────────────────────────────────────────────────────────────

  const handleCreateUser = () => {
    setUserDialogMode('create');
    setEditingUser(null);
    setUserForm({ userid: '', password: '', comment: '', email: '', enabled: true });
    setUserDialogOpen(true);
  };

  const handleEditUser = (user: ProxmoxUser) => {
    setUserDialogMode('edit');
    setEditingUser(user);
    setUserForm({ userid: user.userid, password: '', comment: user.comment ?? '', email: user.email ?? '', enabled: user.enabled });
    setUserDialogOpen(true);
  };

  const handleSubmitUser = async () => {
    try {
      if (userDialogMode === 'create') {
        if (!userForm.userid.trim() || !userForm.password) { toast.error('User ID and password are required'); return; }
        await createProxmoxUser(selectedClusterId, userForm.userid.trim(), userForm.password, userForm.comment || undefined, userForm.email || undefined, userForm.enabled);
        toast.success(`User "${userForm.userid}" created`);
      } else if (editingUser) {
        await updateProxmoxUser(selectedClusterId, editingUser.userid, userForm.comment || undefined, userForm.email || undefined, userForm.enabled);
        toast.success(`User "${editingUser.userid}" updated`);
      }
      setUserDialogOpen(false);
      await loadUsers(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to ${userDialogMode === 'create' ? 'create' : 'update'} user: ${err}`);
    }
  };

  const handleDeleteUser = async (user: ProxmoxUser) => {
    try {
      await deleteProxmoxUser(selectedClusterId, user.userid);
      toast.success(`User "${user.userid}" deleted`);
      await loadUsers(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to delete user: ${err}`);
    }
  };

  const handleEnableUser = async (user: ProxmoxUser) => {
    try {
      await updateProxmoxUser(selectedClusterId, user.userid, undefined, undefined, true);
      toast.success(`User "${user.userid}" enabled`);
      await loadUsers(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to enable user: ${err}`);
    }
  };

  const handleDisableUser = async (user: ProxmoxUser) => {
    try {
      await updateProxmoxUser(selectedClusterId, user.userid, undefined, undefined, false);
      toast.success(`User "${user.userid}" disabled`);
      await loadUsers(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to disable user: ${err}`);
    }
  };

  // ─── Realm handlers ───────────────────────────────────────────────────────────

  const handleCreateRealm = () => {
    setRealmDialogMode('create');
    setEditingRealm(null);
    setRealmForm({ realm: '', realmType: 'pam', comment: '' });
    setRealmDialogOpen(true);
  };

  const handleEditRealm = (realm: AuthRealm) => {
    setRealmDialogMode('edit');
    setEditingRealm(realm);
    setRealmForm({ realm: realm.realm, realmType: realm.type, comment: realm.comment ?? '' });
    setRealmDialogOpen(true);
  };

  const handleSubmitRealm = async () => {
    try {
      if (realmDialogMode === 'create') {
        if (!realmForm.realm.trim()) { toast.error('Realm ID is required'); return; }
        await createProxmoxRealm(selectedClusterId, realmForm.realm.trim(), realmForm.realmType, realmForm.comment || undefined);
        toast.success(`Realm "${realmForm.realm}" created`);
      } else if (editingRealm) {
        await updateProxmoxRealm(selectedClusterId, editingRealm.realm, realmForm.comment || undefined);
        toast.success(`Realm "${editingRealm.realm}" updated`);
      }
      setRealmDialogOpen(false);
      await loadRealms(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to ${realmDialogMode === 'create' ? 'create' : 'update'} realm: ${err}`);
    }
  };

  const handleDeleteRealm = async (realm: AuthRealm) => {
    try {
      await deleteProxmoxRealm(selectedClusterId, realm.realm);
      toast.success(`Realm "${realm.realm}" deleted`);
      await loadRealms(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to delete realm: ${err}`);
    }
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
                <option key={c.id} value={c.id}>{c.name}</option>
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
            onAdd={handleAddAcl}
            onEdit={() => toast.info('Edit ACL: delete the existing entry and create a new one')}
            onDelete={handleDeleteAcl}
          />
        </TabsContent>

        <TabsContent value="users">
          <UserList
            users={users}
            isLoading={isLoadingUsers}
            onRefresh={() => loadUsers(selectedClusterId)}
            onCreate={handleCreateUser}
            onEdit={handleEditUser}
            onDelete={handleDeleteUser}
            onEnable={handleEnableUser}
            onDisable={handleDisableUser}
          />
        </TabsContent>

        <TabsContent value="realms">
          <RealmList
            realms={realms}
            isLoading={isLoadingRealms}
            onRefresh={() => loadRealms(selectedClusterId)}
            onCreate={handleCreateRealm}
            onEdit={handleEditRealm}
            onDelete={handleDeleteRealm}
          />
        </TabsContent>
      </Tabs>

      {/* ACL Create Dialog */}
      <Dialog open={aclDialogOpen} onOpenChange={setAclDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>Add ACL Entry</DialogTitle></DialogHeader>
          <div className="space-y-3 py-2">
            <div className="space-y-1">
              <Label>Path</Label>
              <Input value={aclPath} onChange={(e) => setAclPath(e.target.value)} placeholder="e.g. /vms/100" />
            </div>
            <div className="space-y-1">
              <Label>Role</Label>
              <Input value={aclRole} onChange={(e) => setAclRole(e.target.value)} placeholder="e.g. PVEVMAdmin" />
            </div>
            <div className="space-y-1">
              <Label>User/Group (optional)</Label>
              <Input value={aclUsers} onChange={(e) => setAclUsers(e.target.value)} placeholder="e.g. user@pam or @group" />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAclDialogOpen(false)}>Cancel</Button>
            <Button onClick={handleSubmitAcl}>Add</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* User Create/Edit Dialog */}
      <Dialog open={userDialogOpen} onOpenChange={setUserDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{userDialogMode === 'create' ? 'Create User' : 'Edit User'}</DialogTitle>
          </DialogHeader>
          <div className="space-y-3 py-2">
            {userDialogMode === 'create' && (
              <>
                <div className="space-y-1">
                  <Label>User ID (user@realm)</Label>
                  <Input value={userForm.userid} onChange={(e) => setUserForm((f) => ({ ...f, userid: e.target.value }))} placeholder="e.g. alice@pam" />
                </div>
                <div className="space-y-1">
                  <Label>Password</Label>
                  <Input type="password" value={userForm.password} onChange={(e) => setUserForm((f) => ({ ...f, password: e.target.value }))} />
                </div>
              </>
            )}
            <div className="space-y-1">
              <Label>Comment</Label>
              <Input value={userForm.comment} onChange={(e) => setUserForm((f) => ({ ...f, comment: e.target.value }))} />
            </div>
            <div className="space-y-1">
              <Label>Email</Label>
              <Input type="email" value={userForm.email} onChange={(e) => setUserForm((f) => ({ ...f, email: e.target.value }))} />
            </div>
            <div className="flex items-center gap-2">
              <input type="checkbox" id="user-enabled" checked={userForm.enabled} onChange={(e) => setUserForm((f) => ({ ...f, enabled: e.target.checked }))} />
              <Label htmlFor="user-enabled">Enabled</Label>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setUserDialogOpen(false)}>Cancel</Button>
            <Button onClick={handleSubmitUser}>{userDialogMode === 'create' ? 'Create' : 'Save'}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Realm Create/Edit Dialog */}
      <Dialog open={realmDialogOpen} onOpenChange={setRealmDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{realmDialogMode === 'create' ? 'Create Auth Realm' : 'Edit Auth Realm'}</DialogTitle>
          </DialogHeader>
          <div className="space-y-3 py-2">
            {realmDialogMode === 'create' && (
              <>
                <div className="space-y-1">
                  <Label>Realm ID</Label>
                  <Input value={realmForm.realm} onChange={(e) => setRealmForm((f) => ({ ...f, realm: e.target.value }))} placeholder="e.g. ldap-corp" />
                </div>
                <div className="space-y-1">
                  <Label>Type</Label>
                  <select
                    className="w-full rounded-md border px-3 py-2 text-sm bg-background"
                    value={realmForm.realmType}
                    onChange={(e) => setRealmForm((f) => ({ ...f, realmType: e.target.value }))}
                  >
                    <option value="pam">PAM</option>
                    <option value="pve">PVE</option>
                    <option value="ldap">LDAP</option>
                    <option value="ad">Active Directory</option>
                    <option value="openid">OpenID Connect</option>
                  </select>
                </div>
              </>
            )}
            <div className="space-y-1">
              <Label>Comment</Label>
              <Input value={realmForm.comment} onChange={(e) => setRealmForm((f) => ({ ...f, comment: e.target.value }))} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setRealmDialogOpen(false)}>Cancel</Button>
            <Button onClick={handleSubmitRealm}>{realmDialogMode === 'create' ? 'Create' : 'Save'}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
