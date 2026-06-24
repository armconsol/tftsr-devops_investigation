import React, { useState, useEffect, useCallback } from 'react';
import { RefreshCw, Package, Database } from 'lucide-react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/index';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Badge } from '@/components/ui/index';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/index';
import {
  listProxmoxClusters,
  listAptUpdates,
  listAptRepositories,
  updateAptRepos,
} from '@/lib/proxmoxClient';
import type { AptPackage, AptRepository } from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
import { toast } from 'sonner';

export function ProxmoxUpdatesPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [clusterId, setClusterId] = useState('');
  const [nodeInputValue, setNodeInputValue] = useState('localhost');
  const [nodeId, setNodeId] = useState('localhost');
  const [activeTab, setActiveTab] = useState('updates');

  const [updates, setUpdates] = useState<AptPackage[]>([]);
  const [repositories, setRepositories] = useState<AptRepository[]>([]);
  const [isLoadingUpdates, setIsLoadingUpdates] = useState(false);
  const [isLoadingRepos, setIsLoadingRepos] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);

  useEffect(() => {
    listProxmoxClusters()
      .then((cls) => {
        setClusters(cls);
        if (cls.length > 0) setClusterId(cls[0].id);
      })
      .catch((err: unknown) => {
        console.error('Failed to load clusters:', err);
        toast.error('Failed to load clusters');
      });
  }, []);

  const loadUpdates = useCallback(async (cId: string, nId: string) => {
    if (!cId) return;
    setIsLoadingUpdates(true);
    try {
      const data = await listAptUpdates(cId, nId);
      setUpdates(data as AptPackage[]);
    } catch (err: unknown) {
      console.error('Failed to load APT updates:', err);
      toast.error('Failed to load APT updates');
    } finally {
      setIsLoadingUpdates(false);
    }
  }, []);

  const loadRepositories = useCallback(async (cId: string, nId: string) => {
    if (!cId) return;
    setIsLoadingRepos(true);
    try {
      const data = await listAptRepositories(cId, nId);
      setRepositories(data as AptRepository[]);
    } catch (err: unknown) {
      console.error('Failed to load APT repositories:', err);
      toast.error('Failed to load APT repositories');
    } finally {
      setIsLoadingRepos(false);
    }
  }, []);

  useEffect(() => {
    if (!clusterId) return;
    if (activeTab === 'updates') {
      void loadUpdates(clusterId, nodeId);
    } else if (activeTab === 'repos') {
      void loadRepositories(clusterId, nodeId);
    }
  }, [activeTab, clusterId, nodeId, loadUpdates, loadRepositories]);

  const applyNodeId = () => {
    setNodeId(nodeInputValue.trim() || 'localhost');
  };

  const handleRefreshCache = async () => {
    if (!clusterId) return;
    setIsRefreshing(true);
    try {
      await updateAptRepos(clusterId, nodeId);
      toast.success('APT cache refreshed');
      await loadUpdates(clusterId, nodeId);
    } catch (err: unknown) {
      toast.error(`Failed to refresh APT cache: ${String(err)}`);
    } finally {
      setIsRefreshing(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Package Updates</h1>
          <p className="text-muted-foreground">APT package updates and repository management</p>
        </div>
      </div>

      <div className="flex items-center gap-3 flex-wrap">
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Cluster:</span>
          <select
            className="text-sm border rounded px-2 py-1 bg-background"
            value={clusterId}
            onChange={(e) => setClusterId(e.target.value)}
          >
            {clusters.length === 0 && <option value="">No clusters</option>}
            {clusters.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Node:</span>
          <Input
            className="w-36 h-8 text-sm"
            value={nodeInputValue}
            onChange={(e) => setNodeInputValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') applyNodeId();
            }}
            placeholder="localhost"
          />
          <Button variant="outline" size="sm" onClick={applyNodeId}>
            Apply
          </Button>
        </div>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="updates">
            <Package className="mr-2 h-4 w-4" />
            Available Updates
          </TabsTrigger>
          <TabsTrigger value="repos">
            <Database className="mr-2 h-4 w-4" />
            Repositories
          </TabsTrigger>
        </TabsList>

        <TabsContent value="updates">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>
                Available Updates
                {updates.length > 0 && (
                  <Badge className="ml-2" variant="secondary">
                    {updates.length}
                  </Badge>
                )}
              </CardTitle>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => void loadUpdates(clusterId, nodeId)}
                  disabled={isLoadingUpdates}
                >
                  <RefreshCw className={`mr-2 h-4 w-4 ${isLoadingUpdates ? 'animate-spin' : ''}`} />
                  Refresh
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => void handleRefreshCache()}
                  disabled={isRefreshing || !clusterId}
                >
                  <RefreshCw className={`mr-2 h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
                  Refresh APT Cache
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              {isLoadingUpdates ? (
                <div className="text-muted-foreground text-sm">Loading updates...</div>
              ) : updates.length === 0 ? (
                <div className="text-muted-foreground text-sm py-6 text-center">
                  System is up to date
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Package</TableHead>
                      <TableHead>Current Version</TableHead>
                      <TableHead>New Version</TableHead>
                      <TableHead>Origin</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {updates.map((pkg, i) => (
                      <TableRow key={`${pkg.package}-${i}`}>
                        <TableCell className="font-mono">{pkg.package}</TableCell>
                        <TableCell className="text-muted-foreground">{pkg.version}</TableCell>
                        <TableCell>
                          {pkg.newVersion ? (
                            <span className="text-green-600">{pkg.newVersion}</span>
                          ) : (
                            <span className="text-muted-foreground">—</span>
                          )}
                        </TableCell>
                        <TableCell className="text-muted-foreground text-xs">
                          {pkg.description ?? '—'}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="repos">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>APT Repositories</CardTitle>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void loadRepositories(clusterId, nodeId)}
                disabled={isLoadingRepos}
              >
                <RefreshCw className={`mr-2 h-4 w-4 ${isLoadingRepos ? 'animate-spin' : ''}`} />
                Refresh
              </Button>
            </CardHeader>
            <CardContent>
              {isLoadingRepos ? (
                <div className="text-muted-foreground text-sm">Loading repositories...</div>
              ) : repositories.length === 0 ? (
                <div className="text-muted-foreground text-sm py-6 text-center">
                  No repositories configured
                </div>
              ) : (
                <div className="space-y-2">
                  {repositories.map((repo, i) => (
                    <div key={i} className="p-3 border rounded text-sm">
                      <div className="flex items-center justify-between mb-1">
                        <div className="font-mono text-xs text-muted-foreground">
                          {repo.types.join(' ')} {repo.uris.join(' ')}
                        </div>
                        <Badge variant={repo.enabled ? 'default' : 'secondary'}>
                          {repo.enabled ? 'Enabled' : 'Disabled'}
                        </Badge>
                      </div>
                      <div className="flex items-center gap-4 mt-1 text-xs text-muted-foreground">
                        {repo.suites.length > 0 && (
                          <span>
                            <span className="font-medium">Suites:</span> {repo.suites.join(', ')}
                          </span>
                        )}
                        {repo.components.length > 0 && (
                          <span>
                            <span className="font-medium">Components:</span>{' '}
                            {repo.components.join(', ')}
                          </span>
                        )}
                        {repo.comment && (
                          <span className="italic">{repo.comment}</span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
