import React, { useState, useEffect, useCallback } from 'react';
import { RefreshCw, Trash2 } from 'lucide-react';
import { Button, Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/index';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Badge } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { AclList, UserList, RealmList } from '@/components/Proxmox';
import { useProxmoxClusters } from '@/hooks/useProxmoxClusters';
import {
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
  listTfaEntries,
  addTfaEntry,
  deleteTfaEntry,
  listUserTokens,
  createUserToken,
  deleteUserToken,
} from '@/lib/proxmoxClient';
import type {
  AclEntry,
  ProxmoxUser,
  AuthRealm,
  TfaEntry,
  UserToken,
  UserTokenCreateResult,
} from '@/lib/proxmoxClient';
import { toast } from 'sonner';

export function ProxmoxACLPage() {
  const { clusters, selectedClusterId, setSelectedClusterId } = useProxmoxClusters();
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

  // ─── TFA state ────────────────────────────────────────────────────────────────
  const [tfaEntries, setTfaEntries] = useState<TfaEntry[]>([]);
  const [isLoadingTfa, setIsLoadingTfa] = useState(false);
  const [tfaDialogOpen, setTfaDialogOpen] = useState(false);
  const [tfaUserId, setTfaUserId] = useState('');
  const [tfaDescription, setTfaDescription] = useState('');

  // ─── API Token state ──────────────────────────────────────────────────────────
  const [tokenUserId, setTokenUserId] = useState('');
  const [tokens, setTokens] = useState<UserToken[]>([]);
  const [isLoadingTokens, setIsLoadingTokens] = useState(false);
  const [tokenDialogOpen, setTokenDialogOpen] = useState(false);
  const [tokenName, setTokenName] = useState('');
  const [tokenComment, setTokenComment] = useState('');
  const [tokenPrivsep, setTokenPrivsep] = useState(true);
  const [createdToken, setCreatedToken] = useState<UserTokenCreateResult | null>(null);
  const [tokenResultDialogOpen, setTokenResultDialogOpen] = useState(false);

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

  const loadTfaEntries = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setIsLoadingTfa(true);
    try {
      setTfaEntries(await listTfaEntries(clusterId));
    } catch (err) {
      console.error('Failed to load TFA entries:', err);
      toast.error('Failed to load TFA entries');
    } finally {
      setIsLoadingTfa(false);
    }
  }, []);

  const loadUserTokens = useCallback(async (clusterId: string, userid: string) => {
    if (!clusterId || !userid.trim()) return;
    setIsLoadingTokens(true);
    try {
      setTokens(await listUserTokens(clusterId, userid.trim()));
    } catch (err) {
      console.error('Failed to load tokens:', err);
      toast.error('Failed to load tokens');
    } finally {
      setIsLoadingTokens(false);
    }
  }, []);

  useEffect(() => {
    if (selectedClusterId) {
      loadAcls(selectedClusterId);
      loadUsers(selectedClusterId);
      loadRealms(selectedClusterId);
    }
  }, [selectedClusterId, loadAcls, loadUsers, loadRealms]);

  // Load TFA when tab is entered
  useEffect(() => {
    if (activeTab === 'tfa' && selectedClusterId) {
      void loadTfaEntries(selectedClusterId);
    }
  }, [activeTab, selectedClusterId, loadTfaEntries]);

  const handleRefreshAll = () => {
    loadAcls(selectedClusterId);
    loadUsers(selectedClusterId);
    loadRealms(selectedClusterId);
    if (activeTab === 'tfa') void loadTfaEntries(selectedClusterId);
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

  // ─── TFA handlers ─────────────────────────────────────────────────────────────

  const handleOpenTfaDialog = () => {
    setTfaUserId('');
    setTfaDescription('');
    setTfaDialogOpen(true);
  };

  const handleSubmitTfa = async () => {
    if (!tfaUserId.trim()) { toast.error('User ID is required'); return; }
    try {
      await addTfaEntry(selectedClusterId, tfaUserId.trim(), 'totp', tfaDescription.trim() || undefined);
      toast.success('TOTP entry added');
      setTfaDialogOpen(false);
      await loadTfaEntries(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to add TFA entry: ${err}`);
    }
  };

  const handleDeleteTfa = async (entry: TfaEntry) => {
    if (!window.confirm(`Delete TFA entry "${entry.id}" for ${entry.userid}?`)) return;
    try {
      await deleteTfaEntry(selectedClusterId, entry.userid, entry.id);
      toast.success('TFA entry deleted');
      await loadTfaEntries(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to delete TFA entry: ${err}`);
    }
  };

  // ─── API Token handlers ───────────────────────────────────────────────────────

  const handleLoadTokens = () => {
    void loadUserTokens(selectedClusterId, tokenUserId);
  };

  const handleOpenTokenDialog = () => {
    setTokenName('');
    setTokenComment('');
    setTokenPrivsep(true);
    setTokenDialogOpen(true);
  };

  const handleSubmitToken = async () => {
    if (!tokenUserId.trim()) { toast.error('Load tokens for a user first'); return; }
    if (!tokenName.trim()) { toast.error('Token name is required'); return; }
    try {
      const result = await createUserToken(
        selectedClusterId,
        tokenUserId.trim(),
        tokenName.trim(),
        tokenComment.trim() || undefined,
        tokenPrivsep,
      );
      setTokenDialogOpen(false);
      setCreatedToken(result);
      setTokenResultDialogOpen(true);
      await loadUserTokens(selectedClusterId, tokenUserId);
    } catch (err) {
      toast.error(`Failed to create token: ${err}`);
    }
  };

  const handleDeleteToken = async (token: UserToken) => {
    if (!window.confirm(`Delete token "${token.tokenid}"?`)) return;
    const parts = token.tokenid.split('!');
    const tname = parts[parts.length - 1];
    try {
      await deleteUserToken(selectedClusterId, tokenUserId.trim(), tname);
      toast.success(`Token "${token.tokenid}" deleted`);
      await loadUserTokens(selectedClusterId, tokenUserId);
    } catch (err) {
      toast.error(`Failed to delete token: ${err}`);
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
          <TabsTrigger value="tfa">TFA</TabsTrigger>
          <TabsTrigger value="tokens">API Tokens</TabsTrigger>
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

        {/* ── TFA ────────────────────────────────────────────────────────────── */}
        <TabsContent value="tfa">
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <Button variant="outline" size="sm" onClick={() => void loadTfaEntries(selectedClusterId)} disabled={isLoadingTfa}>
                <RefreshCw className={`mr-2 h-4 w-4 ${isLoadingTfa ? 'animate-spin' : ''}`} />
                Refresh
              </Button>
              <Button size="sm" onClick={handleOpenTfaDialog} disabled={!selectedClusterId}>
                Add TOTP
              </Button>
            </div>
            {isLoadingTfa ? (
              <div className="text-sm text-muted-foreground">Loading...</div>
            ) : tfaEntries.length === 0 ? (
              <div className="text-sm text-muted-foreground py-8 text-center">No TFA entries configured</div>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>ID</TableHead>
                    <TableHead>User ID</TableHead>
                    <TableHead>Type</TableHead>
                    <TableHead>Description</TableHead>
                    <TableHead>Enabled</TableHead>
                    <TableHead>Created</TableHead>
                    <TableHead className="w-12" />
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {tfaEntries.map((entry) => (
                    <TableRow key={entry.id}>
                      <TableCell className="font-mono text-xs">{entry.id}</TableCell>
                      <TableCell>{entry.userid}</TableCell>
                      <TableCell>
                        <Badge variant="outline">{entry.tfaType}</Badge>
                      </TableCell>
                      <TableCell>{entry.description ?? '—'}</TableCell>
                      <TableCell>
                        <Badge variant={entry.enable ? 'default' : 'secondary'}>
                          {entry.enable ? 'Yes' : 'No'}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-xs text-muted-foreground">
                        {entry.created ? new Date(entry.created * 1000).toLocaleDateString() : '—'}
                      </TableCell>
                      <TableCell>
                        <Button
                          variant="ghost"
                          size="sm"
                          className="text-destructive hover:text-destructive"
                          onClick={() => void handleDeleteTfa(entry)}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </div>
        </TabsContent>

        {/* ── API Tokens ─────────────────────────────────────────────────────── */}
        <TabsContent value="tokens">
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              <Input
                className="w-64 h-8 text-sm"
                placeholder="user@realm"
                value={tokenUserId}
                onChange={(e) => setTokenUserId(e.target.value)}
                onKeyDown={(e) => { if (e.key === 'Enter') handleLoadTokens(); }}
              />
              <Button variant="outline" size="sm" onClick={handleLoadTokens} disabled={!tokenUserId.trim()}>
                Load Tokens
              </Button>
              {tokenUserId.trim() && (
                <Button size="sm" onClick={handleOpenTokenDialog} disabled={!selectedClusterId}>
                  Create Token
                </Button>
              )}
            </div>

            {isLoadingTokens ? (
              <div className="text-sm text-muted-foreground">Loading...</div>
            ) : tokens.length === 0 ? (
              <div className="text-sm text-muted-foreground py-8 text-center">
                {tokenUserId.trim() ? 'No tokens found for this user' : 'Enter a user ID to view their tokens'}
              </div>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Token ID</TableHead>
                    <TableHead>Comment</TableHead>
                    <TableHead>Priv Sep</TableHead>
                    <TableHead>Expires</TableHead>
                    <TableHead className="w-12" />
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {tokens.map((token) => (
                    <TableRow key={token.tokenid}>
                      <TableCell className="font-mono text-xs">{token.tokenid}</TableCell>
                      <TableCell>{token.comment ?? '—'}</TableCell>
                      <TableCell>
                        <Badge variant={token.privsep === 1 ? 'default' : 'secondary'}>
                          {token.privsep === 1 ? 'Yes' : 'No'}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-xs text-muted-foreground">
                        {token.expire ? new Date(token.expire * 1000).toLocaleDateString() : 'Never'}
                      </TableCell>
                      <TableCell>
                        <Button
                          variant="ghost"
                          size="sm"
                          className="text-destructive hover:text-destructive"
                          onClick={() => void handleDeleteToken(token)}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </div>
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
            <Button onClick={() => void handleSubmitAcl()}>Add</Button>
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
            <Button onClick={() => void handleSubmitUser()}>{userDialogMode === 'create' ? 'Create' : 'Save'}</Button>
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
            <Button onClick={() => void handleSubmitRealm()}>{realmDialogMode === 'create' ? 'Create' : 'Save'}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* TFA Add Dialog */}
      <Dialog open={tfaDialogOpen} onOpenChange={setTfaDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>Add TOTP Entry</DialogTitle></DialogHeader>
          <div className="space-y-3 py-2">
            <div className="space-y-1">
              <Label>User ID</Label>
              <Input
                value={tfaUserId}
                onChange={(e) => setTfaUserId(e.target.value)}
                placeholder="e.g. user@pam"
              />
            </div>
            <div className="space-y-1">
              <Label>Description (optional)</Label>
              <Input
                value={tfaDescription}
                onChange={(e) => setTfaDescription(e.target.value)}
                placeholder="e.g. Mobile authenticator"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setTfaDialogOpen(false)}>Cancel</Button>
            <Button onClick={() => void handleSubmitTfa()}>Add</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* API Token Create Dialog */}
      <Dialog open={tokenDialogOpen} onOpenChange={setTokenDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>Create API Token</DialogTitle></DialogHeader>
          <div className="space-y-3 py-2">
            <div className="space-y-1">
              <Label>Token Name</Label>
              <Input
                value={tokenName}
                onChange={(e) => setTokenName(e.target.value)}
                placeholder="e.g. mytoken"
              />
            </div>
            <div className="space-y-1">
              <Label>Comment (optional)</Label>
              <Input
                value={tokenComment}
                onChange={(e) => setTokenComment(e.target.value)}
              />
            </div>
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="token-privsep"
                checked={tokenPrivsep}
                onChange={(e) => setTokenPrivsep(e.target.checked)}
              />
              <Label htmlFor="token-privsep">Privilege Separation</Label>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setTokenDialogOpen(false)}>Cancel</Button>
            <Button onClick={() => void handleSubmitToken()}>Create</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Token Created — show value once */}
      <Dialog open={tokenResultDialogOpen} onOpenChange={setTokenResultDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>Token Created</DialogTitle></DialogHeader>
          <div className="py-2 space-y-2">
            <p className="text-sm text-muted-foreground">
              Copy this token value now. It cannot be retrieved again.
            </p>
            <pre className="text-xs font-mono bg-muted p-3 rounded break-all whitespace-pre-wrap">
              {createdToken?.value ?? '(no value returned)'}
            </pre>
            {createdToken?.fullTokenid && (
              <p className="text-xs text-muted-foreground">Token ID: {createdToken.fullTokenid}</p>
            )}
          </div>
          <DialogFooter>
            <Button onClick={() => setTokenResultDialogOpen(false)}>Done</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
