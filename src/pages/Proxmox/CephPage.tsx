import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Badge } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { PoolList, OSDList, CephHealthWidget, NodeSelect } from '@/components/Proxmox';
import { useProxmoxNodes } from '@/hooks/useProxmoxNodes';
import {
  listProxmoxClusters,
  listCephPools,
  listCephOsd,
  getCephHealth,
  listCephMonitors,
  listCephManagers,
  listCephfs,
  getCephFlags,
} from '@/lib/proxmoxClient';
import type { CephMonitor, CephMgr, CephFs, CephHealth, CephPool, CephOsd } from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
import { toast } from 'sonner';

export function ProxmoxCephPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [clusterId, setClusterId] = useState<string>('');
  const [health, setHealth] = useState<CephHealth | null>(null);
  const [pools, setPools] = useState<CephPool[]>([]);
  const [osds, setOsds] = useState<CephOsd[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isCephEnabled, setIsCephEnabled] = useState<boolean | null>(null);

  const [activeTab, setActiveTab] = useState('monitors');
  const [monitors, setMonitors] = useState<CephMonitor[]>([]);
  const [managers, setManagers] = useState<CephMgr[]>([]);
  const [cephFsList, setCephFsList] = useState<CephFs[]>([]);
  const [cephFlags, setCephFlags] = useState<Record<string, unknown> | null>(null);
  const [tabLoading, setTabLoading] = useState(false);
  const { nodeNames, selectedNode: tabNode, setSelectedNode, loading: nodesLoading } =
    useProxmoxNodes(clusterId);

  const loadData = useCallback(async (cId: string, node: string) => {
    if (!cId || !node) return;
    setLoading(true);
    setError(null);

    let cephAvailable = false;
    try {
      const h = await getCephHealth(cId, node);
      setHealth(h);
      cephAvailable = true;
    } catch {
      setIsCephEnabled(false);
      setLoading(false);
      return;
    }

    if (cephAvailable) {
      setIsCephEnabled(true);
      const [poolsResult, osdsResult] = await Promise.allSettled([
        listCephPools(cId, node),
        listCephOsd(cId, node),
      ]);

      if (poolsResult.status === 'fulfilled') {
        setPools(poolsResult.value);
      } else {
        toast.error('Failed to load Ceph pools');
      }

      if (osdsResult.status === 'fulfilled') {
        setOsds(osdsResult.value);
      } else {
        toast.error('Failed to load Ceph OSDs');
      }
    }

    setLoading(false);
  }, []);

  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        setClusters(cls);
        if (cls.length > 0) {
          setClusterId(cls[0].id);
        } else {
          setIsCephEnabled(false);
        }
      })
      .catch((err) => {
        console.error('Failed to load clusters:', err);
        setError('Failed to load clusters');
        setIsCephEnabled(false);
      });
  }, []);

  useEffect(() => {
    if (!clusterId || !tabNode) return;
    loadData(clusterId, tabNode);
  }, [clusterId, tabNode, loadData]);

  const handleClusterChange = (id: string) => {
    setClusterId(id);
    setHealth(null);
    setPools([]);
    setOsds([]);
    setMonitors([]);
    setManagers([]);
    setCephFsList([]);
    setCephFlags(null);
    setIsCephEnabled(null);
  };

  useEffect(() => {
    if (!clusterId || !tabNode || !isCephEnabled) return;
    listCephMonitors(clusterId, tabNode)
      .then(setMonitors)
      .catch(() => toast.error('Failed to load Ceph monitors'));
  }, [clusterId, tabNode, isCephEnabled]);

  const loadManagers = useCallback(async () => {
    if (!clusterId) return;
    setTabLoading(true);
    try {
      const data = await listCephManagers(clusterId, tabNode);
      setManagers(data);
    } catch {
      toast.error('Failed to load Ceph managers');
    } finally {
      setTabLoading(false);
    }
  }, [clusterId, tabNode]);

  const loadCephFs = useCallback(async () => {
    if (!clusterId) return;
    setTabLoading(true);
    try {
      const data = await listCephfs(clusterId, tabNode);
      setCephFsList(data);
    } catch {
      toast.error('Failed to load CephFS');
    } finally {
      setTabLoading(false);
    }
  }, [clusterId, tabNode]);

  const loadFlags = useCallback(async () => {
    if (!clusterId) return;
    setTabLoading(true);
    try {
      const data = await getCephFlags(clusterId, tabNode);
      setCephFlags(data);
    } catch {
      toast.error('Failed to load Ceph flags');
    } finally {
      setTabLoading(false);
    }
  }, [clusterId, tabNode]);

  const handleRefresh = () => {
    if (clusterId && tabNode) loadData(clusterId, tabNode);
  };

  const clusterSelector = clusters.length > 1 && (
    <Select value={clusterId} onValueChange={handleClusterChange}>
      <SelectTrigger className="h-8 w-48 text-sm">
        <SelectValue placeholder="Select datacenter" />
      </SelectTrigger>
      <SelectContent>
        {clusters.map((c) => (
          <SelectItem key={c.id} value={c.id}>{c.name}</SelectItem>
        ))}
      </SelectContent>
    </Select>
  );

  if (isCephEnabled === false) {
    return (
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Ceph Storage</h1>
            <p className="text-muted-foreground">Manage Ceph clusters and storage</p>
          </div>
          <div className="flex space-x-2">{clusterSelector}</div>
        </div>
        <Card>
          <CardContent className="py-12 text-center text-muted-foreground">
            {error ? (
              <p>{error}</p>
            ) : (
              <>
                <p className="text-base font-medium">Ceph is not configured on this datacenter</p>
                <p className="text-sm mt-1">
                  Ceph storage requires a dedicated Ceph cluster deployment on the Proxmox nodes.
                  {clusters.length > 1 && ' Select a different datacenter above if Ceph runs elsewhere.'}
                </p>
              </>
            )}
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Ceph Storage</h1>
          <p className="text-muted-foreground">Manage Ceph clusters and storage</p>
        </div>
        <div className="flex items-center space-x-2">
          {clusterSelector}
          <Button variant="outline" size="sm" onClick={handleRefresh} disabled={loading}>
            <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Ceph Health</CardTitle>
          </CardHeader>
          <CardContent>
            {health ? (
              <CephHealthWidget health={health} />
            ) : (
              <p className="text-sm text-muted-foreground">Loading health data...</p>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Pools</CardTitle>
          </CardHeader>
          <CardContent>
            <PoolList
              pools={pools}
              onRefresh={handleRefresh}
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>OSDs</CardTitle>
          </CardHeader>
          <CardContent>
            <OSDList
              osds={osds}
              onRefresh={handleRefresh}
            />
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Ceph Details</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-2">
            <Label className="text-sm">Node:</Label>
            <NodeSelect
              nodeNames={nodeNames}
              value={tabNode}
              onChange={setSelectedNode}
              loading={nodesLoading}
              disabled={!clusterId}
            />
          </div>

          <Tabs value={activeTab} onValueChange={setActiveTab}>
            <TabsList>
              <TabsTrigger value="monitors">Monitors</TabsTrigger>
              <TabsTrigger value="managers">Managers</TabsTrigger>
              <TabsTrigger value="cephfs">CephFS</TabsTrigger>
              <TabsTrigger value="flags">Flags</TabsTrigger>
            </TabsList>

            <TabsContent value="monitors">
              {monitors.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">No data found.</div>
              ) : (
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b">
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Name</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Address</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Version</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Quorum</th>
                    </tr>
                  </thead>
                  <tbody>
                    {monitors.map((mon) => (
                      <tr key={mon.name} className="border-b last:border-0 hover:bg-muted/50">
                        <td className="py-2 px-3">{mon.name}</td>
                        <td className="py-2 px-3">{mon.address}</td>
                        <td className="py-2 px-3">{mon.version ?? '—'}</td>
                        <td className="py-2 px-3">
                          {mon.quorum === true ? (
                            <Badge className="bg-green-100 text-green-800 text-xs px-2 py-0.5 rounded">
                              In Quorum
                            </Badge>
                          ) : (
                            <Badge className="bg-red-100 text-red-800 text-xs px-2 py-0.5 rounded">
                              Not in Quorum
                            </Badge>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </TabsContent>

            <TabsContent value="managers">
              <div className="flex justify-end mb-3">
                <Button variant="outline" size="sm" onClick={loadManagers} disabled={tabLoading}>
                  <RefreshCw className={`mr-2 h-4 w-4 ${tabLoading ? 'animate-spin' : ''}`} />
                  Load
                </Button>
              </div>
              {managers.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">No data found.</div>
              ) : (
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b">
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Name</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Address</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">State</th>
                    </tr>
                  </thead>
                  <tbody>
                    {managers.map((mgr) => (
                      <tr key={mgr.name} className="border-b last:border-0 hover:bg-muted/50">
                        <td className="py-2 px-3">{mgr.name}</td>
                        <td className="py-2 px-3">{mgr.addr ?? '—'}</td>
                        <td className="py-2 px-3">
                          {mgr.state === 'active' ? (
                            <Badge className="bg-green-100 text-green-800 text-xs px-2 py-0.5 rounded">
                              active
                            </Badge>
                          ) : (
                            <Badge className="bg-gray-100 text-gray-700 text-xs px-2 py-0.5 rounded">
                              {mgr.state ?? 'standby'}
                            </Badge>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </TabsContent>

            <TabsContent value="cephfs">
              <div className="flex justify-end mb-3">
                <Button variant="outline" size="sm" onClick={loadCephFs} disabled={tabLoading}>
                  <RefreshCw className={`mr-2 h-4 w-4 ${tabLoading ? 'animate-spin' : ''}`} />
                  Load
                </Button>
              </div>
              {cephFsList.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">No data found.</div>
              ) : (
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b">
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Name</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Metadata Pool</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Data Pool IDs</th>
                    </tr>
                  </thead>
                  <tbody>
                    {cephFsList.map((fs) => (
                      <tr key={fs.name} className="border-b last:border-0 hover:bg-muted/50">
                        <td className="py-2 px-3">{fs.name}</td>
                        <td className="py-2 px-3">{fs.metadataPool ?? '—'}</td>
                        <td className="py-2 px-3">{(fs.dataPoolIds ?? []).join(', ') || '—'}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </TabsContent>

            <TabsContent value="flags">
              <div className="flex justify-end mb-3">
                <Button variant="outline" size="sm" onClick={loadFlags} disabled={tabLoading}>
                  <RefreshCw className={`mr-2 h-4 w-4 ${tabLoading ? 'animate-spin' : ''}`} />
                  Load
                </Button>
              </div>
              {cephFlags === null ? (
                <div className="text-center py-8 text-muted-foreground">No data found.</div>
              ) : (
                <pre className="text-xs bg-muted p-3 rounded overflow-auto">
                  {JSON.stringify(cephFlags, null, 2)}
                </pre>
              )}
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  );
}
