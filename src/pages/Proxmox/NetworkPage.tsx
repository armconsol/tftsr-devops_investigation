import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Badge } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Checkbox } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { RefreshCw, Network, Plus, Pencil, Trash2, RotateCcw } from 'lucide-react';
import {
  listNetworkInterfaces,
  createNetworkInterface,
  updateNetworkInterface,
  deleteNetworkInterface,
  listProxmoxClusters,
  reloadNetworkConfig,
  NetworkInterface,
  NetworkInterfaceConfig,
} from '@/lib/proxmoxClient';
import { toast } from 'sonner';

interface FormState {
  ifaceName: string;
  ifaceType: string;
  address: string;
  netmask: string;
  gateway: string;
  autostart: boolean;
  active: boolean;
}

const defaultForm: FormState = {
  ifaceName: '',
  ifaceType: 'eth',
  address: '',
  netmask: '',
  gateway: '',
  autostart: false,
  active: false,
};

export function ProxmoxNetworkPage() {
  const [interfaces, setInterfaces] = useState<NetworkInterface[]>([]);
  const [clusterId, setClusterId] = useState('');
  const [nodeId, setNodeId] = useState('');
  const [nodeInput, setNodeInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [showDialog, setShowDialog] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [editingInterface, setEditingInterface] = useState<NetworkInterface | null>(null);
  const [form, setForm] = useState<FormState>(defaultForm);
  const [submitting, setSubmitting] = useState(false);

  const loadInterfaces = useCallback(async (cId: string, nId: string) => {
    if (!cId) return;
    setLoading(true);
    setError(null);
    try {
      const ifaces = await listNetworkInterfaces(cId, nId);
      setInterfaces(ifaces);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        if (cls.length > 0) {
          setClusterId(cls[0].id);
        }
      })
      .catch(console.error);
  }, []);

  const handleNodeApply = () => {
    const trimmed = nodeInput.trim();
    if (!trimmed) return;
    setNodeId(trimmed);
    void loadInterfaces(clusterId, trimmed);
  };

  const handleAddInterface = () => {
    setIsEditing(false);
    setEditingInterface(null);
    setForm(defaultForm);
    setShowDialog(true);
  };

  const handleEditInterface = (iface: NetworkInterface) => {
    setIsEditing(true);
    setEditingInterface(iface);
    setForm({
      ifaceName: iface.iface,
      ifaceType: iface.type,
      address: iface.address ?? '',
      netmask: iface.netmask ?? '',
      gateway: iface.gateway ?? '',
      autostart: iface.autostart,
      active: iface.active,
    });
    setShowDialog(true);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!clusterId) return;

    const config: NetworkInterfaceConfig = {
      iface: form.ifaceName,
      type: form.ifaceType,
      address: form.address || undefined,
      netmask: form.netmask || undefined,
      gateway: form.gateway || undefined,
      active: form.active,
      autostart: form.autostart,
    };

    setSubmitting(true);
    try {
      if (isEditing && editingInterface) {
        await updateNetworkInterface(clusterId, nodeId, editingInterface.iface, config);
        toast.success(`Interface "${editingInterface.iface}" updated`);
      } else {
        await createNetworkInterface(clusterId, nodeId, config);
        toast.success(`Interface "${config.iface}" created`);
      }
      setShowDialog(false);
      await loadInterfaces(clusterId, nodeId);
    } catch (e) {
      toast.error(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  const handleReload = async () => {
    try {
      const upid = await reloadNetworkConfig(clusterId, nodeId);
      toast.success(`Network reload started: ${upid}`);
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleDeleteInterface = async (iface: NetworkInterface) => {
    if (!window.confirm(`Delete interface "${iface.iface}"? This cannot be undone.`)) return;
    try {
      await deleteNetworkInterface(clusterId, nodeId, iface.iface);
      toast.success(`Interface "${iface.iface}" deleted`);
      await loadInterfaces(clusterId, nodeId);
    } catch (e) {
      toast.error(String(e));
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Network</h1>
          <p className="text-muted-foreground">Network interfaces and bridges</p>
        </div>
        <div className="flex items-center gap-2">
          <Input
            className="h-8 w-36 text-sm"
            placeholder="Node name"
            value={nodeInput}
            onChange={(e) => setNodeInput(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') handleNodeApply(); }}
          />
          <Button variant="outline" size="sm" onClick={handleNodeApply} disabled={!nodeInput.trim() || !clusterId}>
            Load
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => void handleReload()}
            disabled={!clusterId || !nodeId}
          >
            <RotateCcw className="mr-2 h-4 w-4" />
            Apply Network Changes
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={handleAddInterface}
            disabled={!clusterId || !nodeId}
          >
            <Plus className="mr-2 h-4 w-4" />
            Add Interface
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => void loadInterfaces(clusterId, nodeId)}
            disabled={loading || !clusterId || !nodeId}
          >
            <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {error && (
        <div className="rounded border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </div>
      )}

      <Card>
        <CardHeader>
          <CardTitle>Network Interfaces</CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="text-sm text-muted-foreground">Loading...</div>
          ) : interfaces.length === 0 ? (
            <div className="text-sm text-muted-foreground">
              {!clusterId ? 'No cluster configured.' : !nodeId ? 'Enter a node name above and click Load.' : 'No network interfaces found.'}
            </div>
          ) : (
            <div className="space-y-2">
              {interfaces.map((iface, i) => (
                <div key={`${iface.iface}-${i}`} className="flex items-center gap-3 rounded border p-3">
                  <Network className="h-4 w-4 shrink-0 text-muted-foreground" />
                  <div className="flex-1 min-w-0">
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="font-mono font-medium">{iface.iface}</span>
                      <Badge variant="outline">{iface.type}</Badge>
                      <Badge variant={iface.active ? 'default' : 'secondary'}>
                        {iface.active ? 'Active' : 'Inactive'}
                      </Badge>
                      {iface.autostart && (
                        <Badge variant="outline" className="text-xs">Autostart</Badge>
                      )}
                    </div>
                    {(iface.address || iface.gateway) && (
                      <div className="mt-1 text-xs text-muted-foreground">
                        {iface.address && (
                          <span>
                            {iface.address}
                            {iface.netmask ? `/${iface.netmask}` : ''}
                          </span>
                        )}
                        {iface.gateway && (
                          <span className="ml-2">gw {iface.gateway}</span>
                        )}
                      </div>
                    )}
                    {iface.comments && (
                      <div className="mt-1 text-xs italic text-muted-foreground">
                        {iface.comments}
                      </div>
                    )}
                  </div>
                  <div className="flex items-center gap-1 shrink-0">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleEditInterface(iface)}
                      title="Edit"
                    >
                      <Pencil className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => void handleDeleteInterface(iface)}
                      title="Delete"
                      className="text-destructive hover:text-destructive"
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      <Dialog open={showDialog} onOpenChange={setShowDialog}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>{isEditing ? 'Edit Interface' : 'Add Interface'}</DialogTitle>
          </DialogHeader>
          <form onSubmit={(e) => void handleSubmit(e)} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="ifaceName">Interface Name</Label>
              <Input
                id="ifaceName"
                value={form.ifaceName}
                onChange={(e) => setForm((f) => ({ ...f, ifaceName: e.target.value }))}
                placeholder="e.g. vmbr0"
                disabled={isEditing || submitting}
                required
              />
            </div>

            <div className="space-y-2">
              <Label>Type</Label>
              <Select
                value={form.ifaceType}
                onValueChange={(v) => setForm((f) => ({ ...f, ifaceType: v }))}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="eth">eth</SelectItem>
                  <SelectItem value="bridge">bridge</SelectItem>
                  <SelectItem value="bond">bond</SelectItem>
                  <SelectItem value="vlan">vlan</SelectItem>
                  <SelectItem value="OVSBridge">OVS Bridge</SelectItem>
                  <SelectItem value="OVSBond">OVS Bond</SelectItem>
                  <SelectItem value="OVSPort">OVS Port</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="address">IP Address</Label>
              <Input
                id="address"
                value={form.address}
                onChange={(e) => setForm((f) => ({ ...f, address: e.target.value }))}
                placeholder="e.g. 192.168.1.100"
                disabled={submitting}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="netmask">Netmask</Label>
              <Input
                id="netmask"
                value={form.netmask}
                onChange={(e) => setForm((f) => ({ ...f, netmask: e.target.value }))}
                placeholder="e.g. 255.255.255.0"
                disabled={submitting}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="gateway">Gateway</Label>
              <Input
                id="gateway"
                value={form.gateway}
                onChange={(e) => setForm((f) => ({ ...f, gateway: e.target.value }))}
                placeholder="e.g. 192.168.1.1"
                disabled={submitting}
              />
            </div>

            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
                <Checkbox
                  id="autostart"
                  checked={form.autostart}
                  onCheckedChange={(v) => setForm((f) => ({ ...f, autostart: v as boolean }))}
                  disabled={submitting}
                />
                <Label htmlFor="autostart">Autostart</Label>
              </div>
              <div className="flex items-center gap-2">
                <Checkbox
                  id="active"
                  checked={form.active}
                  onCheckedChange={(v) => setForm((f) => ({ ...f, active: v as boolean }))}
                  disabled={submitting}
                />
                <Label htmlFor="active">Active</Label>
              </div>
            </div>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => setShowDialog(false)}
                disabled={submitting}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={submitting}>
                {submitting ? 'Saving...' : isEditing ? 'Update' : 'Create'}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  );
}
