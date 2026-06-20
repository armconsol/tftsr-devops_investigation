import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { PoolList, OSDList, CephHealthWidget, MonitorList } from '@/components/Proxmox';
import { listProxmoxClusters, listCephPools, listCephOsd, getCephHealth } from '@/lib/proxmoxClient';
import { toast } from 'sonner';

export function ProxmoxCephPage() {
  const [clusterId, setClusterId] = useState<string>('');
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [health, setHealth] = useState<any>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [pools, setPools] = useState<any[]>([]);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [osds, setOsds] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isCephEnabled, setIsCephEnabled] = useState<boolean | null>(null);

  const loadData = useCallback(async (cId: string) => {
    if (!cId) return;
    setLoading(true);
    setError(null);

    // Check Ceph availability by fetching health first
    let cephAvailable = false;
    try {
      const h = await getCephHealth(cId);
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
        listCephPools(cId),
        listCephOsd(cId),
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
        if (cls.length > 0) {
          setClusterId(cls[0].id);
          loadData(cls[0].id);
        } else {
          setIsCephEnabled(false);
        }
      })
      .catch((err) => {
        console.error('Failed to load clusters:', err);
        setError('Failed to load clusters');
        setIsCephEnabled(false);
      });
  }, [loadData]);

  const handleRefresh = () => {
    if (clusterId) loadData(clusterId);
  };

  if (isCephEnabled === false) {
    return (
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Ceph Storage</h1>
            <p className="text-muted-foreground">Manage Ceph clusters and storage</p>
          </div>
        </div>
        <Card>
          <CardContent className="py-12 text-center text-muted-foreground">
            {error ? (
              <p>{error}</p>
            ) : (
              <>
                <p className="text-base font-medium">Ceph is not configured on this cluster</p>
                <p className="text-sm mt-1">
                  Ceph storage requires a dedicated Ceph cluster deployment on the Proxmox nodes.
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
        <div className="flex space-x-2">
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
          <CardTitle>Monitors</CardTitle>
        </CardHeader>
        <CardContent>
          <MonitorList
            monitors={[]}
            onRefresh={handleRefresh}
          />
        </CardContent>
      </Card>
    </div>
  );
}
