import React, { useState, useEffect, useCallback } from 'react';
import { Button, Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/index';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Card, CardContent } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { RefreshCw, Plus, Trash2 } from 'lucide-react';
import {
  listProxmoxClusters,
  listSdnZones,
  listSdnVnets,
  listSdnControllers,
  createSdnZone,
  deleteSdnZone,
  createSdnVnet,
  deleteSdnVnet,
} from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
import { toast } from 'sonner';

export function ProxmoxSDNPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [selectedClusterId, setSelectedClusterId] = useState<string>('');
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [zones, setZones] = useState<any[]>([]);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [vnets, setVnets] = useState<any[]>([]);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [controllers, setControllers] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [activeTab, setActiveTab] = useState('zones');

  const [zoneDialogOpen, setZoneDialogOpen] = useState(false);
  const [zoneId, setZoneId] = useState('');
  const [zoneAsn, setZoneAsn] = useState('65000');
  const [zoneVni, setZoneVni] = useState('10000');

  const [vnetDialogOpen, setVnetDialogOpen] = useState(false);
  const [vnetId, setVnetId] = useState('');
  const [vnetZone, setVnetZone] = useState('');
  const [vnetL2vni, setVnetL2vni] = useState('10000');

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

  const loadAll = useCallback(async (clusterId: string) => {
    if (!clusterId) return;
    setIsLoading(true);
    try {
      const [z, v, c] = await Promise.all([
        listSdnZones(clusterId).catch(() => []),
        listSdnVnets(clusterId).catch(() => []),
        listSdnControllers(clusterId).catch(() => []),
      ]);
      setZones(z);
      setVnets(v);
      setControllers(c);
    } catch (err) {
      console.error('Failed to load SDN data:', err);
      toast.error('Failed to load SDN data');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (selectedClusterId) loadAll(selectedClusterId);
  }, [selectedClusterId, loadAll]);

  const handleCreateZone = async () => {
    if (!zoneId.trim()) { toast.error('Zone ID is required'); return; }
    try {
      await createSdnZone(selectedClusterId, zoneId.trim(), parseInt(zoneAsn) || 65000, parseInt(zoneVni) || 10000);
      toast.success(`SDN zone "${zoneId}" created`);
      setZoneDialogOpen(false);
      await loadAll(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to create SDN zone: ${err}`);
    }
  };

  const handleDeleteZone = async (zone: string) => {
    try {
      await deleteSdnZone(selectedClusterId, zone);
      toast.success(`SDN zone "${zone}" deleted`);
      await loadAll(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to delete SDN zone: ${err}`);
    }
  };

  const handleCreateVnet = async () => {
    if (!vnetId.trim() || !vnetZone.trim()) { toast.error('VNet ID and zone are required'); return; }
    try {
      await createSdnVnet(selectedClusterId, vnetId.trim(), vnetZone.trim(), parseInt(vnetL2vni) || 10000);
      toast.success(`VNet "${vnetId}" created`);
      setVnetDialogOpen(false);
      await loadAll(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to create VNet: ${err}`);
    }
  };

  const handleDeleteVnet = async (vnet: string) => {
    try {
      await deleteSdnVnet(selectedClusterId, vnet);
      toast.success(`VNet "${vnet}" deleted`);
      await loadAll(selectedClusterId);
    } catch (err) {
      toast.error(`Failed to delete VNet: ${err}`);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">SDN</h1>
          <p className="text-muted-foreground">Software Defined Networking</p>
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
          <Button variant="outline" size="sm" onClick={() => loadAll(selectedClusterId)} disabled={isLoading}>
            <RefreshCw className={`mr-2 h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="zones">Zones ({zones.length})</TabsTrigger>
          <TabsTrigger value="vnets">Virtual Networks ({vnets.length})</TabsTrigger>
          <TabsTrigger value="controllers">Controllers ({controllers.length})</TabsTrigger>
        </TabsList>

        <TabsContent value="zones">
          <Card>
            <CardContent className="pt-4">
              <div className="flex justify-end mb-3">
                <Button size="sm" onClick={() => { setZoneId(''); setZoneAsn('65000'); setZoneVni('10000'); setZoneDialogOpen(true); }}>
                  <Plus className="mr-2 h-4 w-4" />New Zone
                </Button>
              </div>
              {zones.length === 0 ? (
                <p className="text-sm text-muted-foreground text-center py-8">No SDN zones configured.</p>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Zone</TableHead>
                      <TableHead>Type</TableHead>
                      <TableHead>State</TableHead>
                      <TableHead className="w-16"></TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {zones.map((z) => (
                      <TableRow key={z.zone ?? z.id}>
                        <TableCell className="font-mono text-sm">{z.zone ?? z.id}</TableCell>
                        <TableCell>{z.type ?? 'evpn'}</TableCell>
                        <TableCell>{z.state ?? '—'}</TableCell>
                        <TableCell>
                          <Button variant="ghost" size="sm" onClick={() => handleDeleteZone(z.zone ?? z.id)}>
                            <Trash2 className="h-4 w-4 text-destructive" />
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="vnets">
          <Card>
            <CardContent className="pt-4">
              <div className="flex justify-end mb-3">
                <Button size="sm" onClick={() => { setVnetId(''); setVnetZone(zones[0]?.zone ?? ''); setVnetL2vni('10000'); setVnetDialogOpen(true); }}>
                  <Plus className="mr-2 h-4 w-4" />New VNet
                </Button>
              </div>
              {vnets.length === 0 ? (
                <p className="text-sm text-muted-foreground text-center py-8">No virtual networks configured.</p>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>VNet</TableHead>
                      <TableHead>Zone</TableHead>
                      <TableHead>L2VNI</TableHead>
                      <TableHead>Status</TableHead>
                      <TableHead className="w-16"></TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {vnets.map((v) => (
                      <TableRow key={v.vnet ?? v.id}>
                        <TableCell className="font-mono text-sm">{v.vnet ?? v.id}</TableCell>
                        <TableCell>{v.zone ?? '—'}</TableCell>
                        <TableCell>{v.l2vni ?? '—'}</TableCell>
                        <TableCell>{v.status ?? '—'}</TableCell>
                        <TableCell>
                          <Button variant="ghost" size="sm" onClick={() => handleDeleteVnet(v.vnet ?? v.id)}>
                            <Trash2 className="h-4 w-4 text-destructive" />
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="controllers">
          <Card>
            <CardContent className="pt-4">
              {controllers.length === 0 ? (
                <p className="text-sm text-muted-foreground text-center py-8">No SDN controllers configured.</p>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Controller</TableHead>
                      <TableHead>Type</TableHead>
                      <TableHead>State</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {controllers.map((c) => (
                      <TableRow key={c.controller ?? c.id}>
                        <TableCell className="font-mono text-sm">{c.controller ?? c.id}</TableCell>
                        <TableCell>{c.type ?? '—'}</TableCell>
                        <TableCell>{c.state ?? '—'}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Zone Create Dialog */}
      <Dialog open={zoneDialogOpen} onOpenChange={setZoneDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>Create EVPN Zone</DialogTitle></DialogHeader>
          <div className="space-y-3 py-2">
            <div className="space-y-1">
              <Label>Zone ID</Label>
              <Input value={zoneId} onChange={(e) => setZoneId(e.target.value)} placeholder="e.g. evpn-zone1" />
            </div>
            <div className="space-y-1">
              <Label>ASN</Label>
              <Input type="number" value={zoneAsn} onChange={(e) => setZoneAsn(e.target.value)} placeholder="65000" />
            </div>
            <div className="space-y-1">
              <Label>VNI</Label>
              <Input type="number" value={zoneVni} onChange={(e) => setZoneVni(e.target.value)} placeholder="10000" />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setZoneDialogOpen(false)}>Cancel</Button>
            <Button onClick={handleCreateZone}>Create</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* VNet Create Dialog */}
      <Dialog open={vnetDialogOpen} onOpenChange={setVnetDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>Create Virtual Network</DialogTitle></DialogHeader>
          <div className="space-y-3 py-2">
            <div className="space-y-1">
              <Label>VNet ID</Label>
              <Input value={vnetId} onChange={(e) => setVnetId(e.target.value)} placeholder="e.g. vnet100" />
            </div>
            <div className="space-y-1">
              <Label>Zone</Label>
              {zones.length > 0 ? (
                <select
                  className="w-full rounded-md border px-3 py-2 text-sm bg-background"
                  value={vnetZone}
                  onChange={(e) => setVnetZone(e.target.value)}
                >
                  {zones.map((z) => (
                    <option key={z.zone ?? z.id} value={z.zone ?? z.id}>{z.zone ?? z.id}</option>
                  ))}
                </select>
              ) : (
                <Input value={vnetZone} onChange={(e) => setVnetZone(e.target.value)} placeholder="zone-id" />
              )}
            </div>
            <div className="space-y-1">
              <Label>L2VNI</Label>
              <Input type="number" value={vnetL2vni} onChange={(e) => setVnetL2vni(e.target.value)} placeholder="10000" />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setVnetDialogOpen(false)}>Cancel</Button>
            <Button onClick={handleCreateVnet}>Create</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
