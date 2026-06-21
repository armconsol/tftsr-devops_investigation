import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Badge } from '@/components/ui/index';
import { RefreshCw, Network, Plus, Edit, Trash2 } from 'lucide-react';
import { listNetworkInterfaces, listProxmoxClusters, NetworkInterface } from '@/lib/proxmoxClient';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { toast } from 'sonner';

export function ProxmoxNetworkPage() {
  const [interfaces, setInterfaces] = useState<NetworkInterface[]>([]);
  const [clusterId, setClusterId] = useState('');
  const [nodeId] = useState('localhost');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [editingInterface, setEditingInterface] = useState<NetworkInterface | null>(null);
  
  // Form state
  const [ifaceName, setIfaceName] = useState('');
  const [ifaceType, setIfaceType] = useState('eth');
  const [address, setAddress] = useState('');
  const [netmask, setNetmask] = useState('');
  const [gateway, setGateway] = useState('');
  const [active, setActive] = useState(true);

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
          void loadInterfaces(cls[0].id, nodeId);
        }
      })
      .catch(console.error);
  }, [loadInterfaces, nodeId]);

  const handleAddInterface = () => {
    setEditingInterface(null);
    setIfaceName('');
    setIfaceType('eth');
    setAddress('');
    setNetmask('');
    setGateway('');
    setActive(true);
    setShowAddDialog(true);
  };

  const handleEditInterface = (iface: NetworkInterface) => {
    setEditingInterface(iface);
    setIfaceName(iface.iface);
    setIfaceType(iface.type);
    setAddress(iface.address || '');
    setNetmask(iface.netmask || '');
    setGateway(iface.gateway || '');
    setActive(iface.active);
    setShowAddDialog(true);
  };

  const handleSubmit = async () => {
    if (!ifaceName || !ifaceType) {
      toast.error('Interface name and type are required');
      return;
    }

    try {
      if (editingInterface) {
        toast.info(`Updating interface ${ifaceName} - implementation pending`);
      } else {
        toast.info(`Creating interface ${ifaceName} - implementation pending`);
      }
      setShowAddDialog(false);
    } catch (error) {
      console.error('Failed to save interface:', error);
      toast.error(`Failed to save interface: ${error}`);
    }
  };

  const handleDeleteInterface = async (iface: NetworkInterface) => {
    if (!confirm(`Are you sure you want to delete interface ${iface.iface}?`)) {
      return;
    }

    try {
      toast.info(`Deleting interface ${iface.iface} - implementation pending`);
    } catch (error) {
      console.error('Failed to delete interface:', error);
      toast.error(`Failed to delete interface: ${error}`);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Network</h1>
          <p className="text-muted-foreground">Network interfaces and bridges</p>
        </div>
        <div className="flex items-center space-x-2">
          <Button variant="outline" size="sm" onClick={() => void loadInterfaces(clusterId, nodeId)} disabled={loading || !clusterId}>
            <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          <Button size="sm" onClick={handleAddInterface}>
            <Plus className="mr-2 h-4 w-4" />
            Add Interface
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
              {clusterId ? 'No network interfaces found.' : 'No cluster configured.'}
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
                  <div className="flex items-center gap-2">
                    <button
                      className="rounded p-1 hover:bg-accent"
                      onClick={() => handleEditInterface(iface)}
                      title="Edit"
                    >
                      <Edit className="h-4 w-4" />
                    </button>
                    <button
                      className="rounded p-1 hover:bg-red-100 hover:text-red-600"
                      onClick={() => handleDeleteInterface(iface)}
                      title="Delete"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editingInterface ? 'Edit Network Interface' : 'Add Network Interface'}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="iface">Interface Name</Label>
              <Input
                id="iface"
                value={ifaceName}
                onChange={(e) => setIfaceName(e.target.value)}
                placeholder="eth0"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="type">Interface Type</Label>
              <Input
                id="type"
                value={ifaceType}
                onChange={(e) => setIfaceType(e.target.value)}
                placeholder="eth, bond, bridge, vlan"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="address">IP Address</Label>
              <Input
                id="address"
                value={address}
                onChange={(e) => setAddress(e.target.value)}
                placeholder="192.168.1.100"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="netmask">Netmask</Label>
              <Input
                id="netmask"
                value={netmask}
                onChange={(e) => setNetmask(e.target.value)}
                placeholder="24"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="gateway">Gateway</Label>
              <Input
                id="gateway"
                value={gateway}
                onChange={(e) => setGateway(e.target.value)}
                placeholder="192.168.1.1"
              />
            </div>
            <div className="flex items-center space-x-2">
              <input
                type="checkbox"
                id="active"
                checked={active}
                onChange={(e) => setActive(e.target.checked)}
                className="rounded"
              />
              <Label htmlFor="active">Active</Label>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAddDialog(false)}>
              Cancel
            </Button>
            <Button onClick={handleSubmit}>
              {editingInterface ? 'Update' : 'Create'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
