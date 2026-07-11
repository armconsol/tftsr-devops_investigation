import React, { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/index';
import { FirewallRuleList } from '@/components/Proxmox';
import {
  listFirewallRules,
  addFirewallRule,
  listClusterFirewallRules,
  getClusterFirewallStatus,
  listGuestFirewallRules,
  addGuestFirewallRule,
  deleteGuestFirewallRule,
} from '@/lib/proxmoxClient';
import type { ClusterFirewallStatus } from '@/lib/proxmoxClient';
import { useProxmoxClusters } from '@/hooks/useProxmoxClusters';
import { toast } from 'sonner';

export function ProxmoxFirewallPage() {
  const { clusters, selectedClusterId, setSelectedClusterId } = useProxmoxClusters();
  const [nodeInputValue, setNodeInputValue] = useState('localhost');
  const [nodeId, setNodeId] = useState('localhost');
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [rules, setRules] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const [activeTab, setActiveTab] = useState('node-rules');

  // Cluster rules tab
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [clusterRules, setClusterRules] = useState<any[]>([]);
  const [clusterStatus, setClusterStatus] = useState<ClusterFirewallStatus | null>(null);
  const [clusterRulesLoading, setClusterRulesLoading] = useState(false);

  // Guest rules tab
  const [guestNodeInput, setGuestNodeInput] = useState('localhost');
  const [guestNode, setGuestNode] = useState('localhost');
  const [vmIdInput, setVmIdInput] = useState('');
  const [vmId, setVmId] = useState<number | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [guestRules, setGuestRules] = useState<any[]>([]);
  const [guestLoading, setGuestLoading] = useState(false);

  // Guest rule dialog
  const [showGuestRuleDialog, setShowGuestRuleDialog] = useState(false);
  const [guestRuleAction, setGuestRuleAction] = useState('ACCEPT');
  const [guestRuleProtocol, setGuestRuleProtocol] = useState('tcp');
  const [guestRuleSource, setGuestRuleSource] = useState('');
  const [guestRuleDest, setGuestRuleDest] = useState('');
  const [guestRuleDport, setGuestRuleDport] = useState('');

  // Node rule dialog
  const [showNewRuleDialog, setShowNewRuleDialog] = useState(false);
  const [ruleAction, setRuleAction] = useState('ACCEPT');
  const [ruleProtocol, setRuleProtocol] = useState('tcp');
  const [ruleSource, setRuleSource] = useState('');
  const [ruleDest, setRuleDest] = useState('');
  const [ruleDport, setRuleDport] = useState('');
  const [ruleComment, setRuleComment] = useState('');

  const loadRules = useCallback(async (clusterId: string, nId: string) => {
    if (!clusterId) return;
    setIsLoading(true);
    try {
      const data = await listFirewallRules(clusterId, nId);
      setRules(data);
    } catch (err) {
      console.error('Failed to load firewall rules:', err);
      toast.error('Failed to load firewall rules');
    } finally {
      setIsLoading(false);
    }
  }, []);

  const loadClusterRules = useCallback(async (cId: string) => {
    if (!cId) return;
    setClusterRulesLoading(true);
    try {
      const [fetchedRules, status] = await Promise.all([
        listClusterFirewallRules(cId),
        getClusterFirewallStatus(cId),
      ]);
      setClusterRules(fetchedRules);
      setClusterStatus(status);
    } catch (err) {
      toast.error(`Failed to load cluster firewall: ${err}`);
    } finally {
      setClusterRulesLoading(false);
    }
  }, []);

  const loadGuestRules = useCallback(async (cId: string, node: string, vid: number) => {
    setGuestLoading(true);
    try {
      const data = await listGuestFirewallRules(cId, node, vid);
      setGuestRules(data);
    } catch (err) {
      toast.error(`Failed to load guest firewall rules: ${err}`);
    } finally {
      setGuestLoading(false);
    }
  }, []);

  useEffect(() => {
    if (selectedClusterId) loadRules(selectedClusterId, nodeId);
  }, [selectedClusterId, nodeId, loadRules]);

  useEffect(() => {
    if (selectedClusterId) loadClusterRules(selectedClusterId);
  }, [selectedClusterId, loadClusterRules]);

  const applyNodeId = () => {
    setNodeId(nodeInputValue.trim() || 'localhost');
  };

  const handleNewRule = () => {
    setRuleAction('ACCEPT');
    setRuleProtocol('tcp');
    setRuleSource('');
    setRuleDest('');
    setRuleDport('');
    setRuleComment('');
    setShowNewRuleDialog(true);
  };

  const handleSubmitNewRule = async () => {
    if (!ruleAction || !ruleProtocol) {
      toast.error('Action and protocol are required');
      return;
    }

    try {
      await addFirewallRule(selectedClusterId, nodeId, {
        action: ruleAction,
        proto: ruleProtocol,
        source: ruleSource || undefined,
        dest: ruleDest || undefined,
        dport: ruleDport || undefined,
        comment: ruleComment || undefined,
        enable: 1,
      });
      toast.success('Firewall rule created');
      setShowNewRuleDialog(false);
      await loadRules(selectedClusterId, nodeId);
    } catch (error) {
      console.error('Failed to create firewall rule:', error);
      toast.error(`Failed to create firewall rule: ${error}`);
    }
  };

  const handleSubmitGuestRule = async () => {
    if (!vmId) { toast.error('No VM selected'); return; }
    try {
      await addGuestFirewallRule(
        selectedClusterId, guestNode, vmId,
        guestRuleAction,
        guestRuleProtocol || undefined,
        guestRuleSource || undefined,
        guestRuleDest || undefined,
        guestRuleDport || undefined,
        true,
      );
      toast.success('Guest firewall rule added');
      setShowGuestRuleDialog(false);
      await loadGuestRules(selectedClusterId, guestNode, vmId);
    } catch (err) {
      toast.error(`Failed to add guest rule: ${err}`);
    }
  };

  const handleDeleteGuestRule = async (pos: number) => {
    if (!vmId) return;
    try {
      await deleteGuestFirewallRule(selectedClusterId, guestNode, vmId, pos);
      toast.success('Rule deleted');
      await loadGuestRules(selectedClusterId, guestNode, vmId);
    } catch (err) {
      toast.error(`Failed to delete rule: ${err}`);
    }
  };

  if (clusters.length === 0 && !isLoading) {
    return (
      <div className="space-y-4">
        <div>
          <h1 className="text-2xl font-bold">Firewall</h1>
          <p className="text-muted-foreground">Configure firewall rules</p>
        </div>
        <div className="text-center py-12 text-muted-foreground">
          <p>No Proxmox clusters configured.</p>
          <p className="text-sm mt-1">Add a remote connection first.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Firewall</h1>
          <p className="text-muted-foreground">Configure firewall rules</p>
        </div>
      </div>

      {clusters.length > 0 && (
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">Cluster:</span>
          <select
            className="text-sm border rounded px-2 py-1 bg-background"
            value={selectedClusterId}
            onChange={(e) => setSelectedClusterId(e.target.value)}
          >
            {clusters.map((c) => (
              <option key={c.id} value={c.id}>{c.name}</option>
            ))}
          </select>
        </div>
      )}

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="cluster-rules">Cluster Rules</TabsTrigger>
          <TabsTrigger value="node-rules">Node Rules</TabsTrigger>
          <TabsTrigger value="guest-rules">Guest Rules</TabsTrigger>
        </TabsList>

        <TabsContent value="cluster-rules">
          <div className="space-y-4 pt-4">
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => loadClusterRules(selectedClusterId)}
                disabled={clusterRulesLoading}
              >
                <RefreshCw className="mr-2 h-4 w-4" />
                Refresh
              </Button>
            </div>

            {clusterStatus && (
              <div className="text-sm text-muted-foreground mb-3 flex gap-4">
                <span>Status: <span className="font-medium">{clusterStatus.enable === 1 ? 'Enabled' : 'Disabled'}</span></span>
                <span>Policy In: <span className="font-medium">{clusterStatus.policyIn ?? '—'}</span></span>
                <span>Policy Out: <span className="font-medium">{clusterStatus.policyOut ?? '—'}</span></span>
              </div>
            )}

            {clusterRulesLoading ? (
              <p className="text-sm text-muted-foreground">Loading cluster rules...</p>
            ) : clusterRules.length === 0 ? (
              <p className="text-sm text-muted-foreground">No cluster firewall rules found.</p>
            ) : (
              <div className="rounded-md border">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b">
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Pos</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Action</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Protocol</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Source</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Dest</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Port</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Comment</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Enabled</th>
                    </tr>
                  </thead>
                  <tbody>
                    {clusterRules.map((rule, i) => (
                      <tr key={i} className="border-b last:border-0 hover:bg-muted/50">
                        <td className="py-2 px-3">{rule.pos ?? i}</td>
                        <td className="py-2 px-3">{rule.action ?? '—'}</td>
                        <td className="py-2 px-3">{rule.proto ?? '—'}</td>
                        <td className="py-2 px-3">{rule.source ?? '—'}</td>
                        <td className="py-2 px-3">{rule.dest ?? '—'}</td>
                        <td className="py-2 px-3">{rule.dport ?? '—'}</td>
                        <td className="py-2 px-3">{rule.comment ?? '—'}</td>
                        <td className="py-2 px-3">
                          <span className={rule.enable === 1 ? "text-xs bg-green-100 text-green-800 px-2 py-0.5 rounded" : "text-xs bg-gray-100 text-gray-600 px-2 py-0.5 rounded"}>
                            {rule.enable === 1 ? 'Yes' : 'No'}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </TabsContent>

        <TabsContent value="node-rules">
          <div className="space-y-4 pt-4">
            <div className="flex items-center gap-3 flex-wrap">
              <div className="flex items-center gap-2">
                <span className="text-sm text-muted-foreground">Node:</span>
                <Input
                  className="w-36 h-8 text-sm"
                  value={nodeInputValue}
                  onChange={(e) => setNodeInputValue(e.target.value)}
                  onKeyDown={(e) => { if (e.key === 'Enter') applyNodeId(); }}
                  placeholder="localhost"
                />
                <Button variant="outline" size="sm" onClick={applyNodeId}>Apply</Button>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={() => loadRules(selectedClusterId, nodeId)}
              >
                <RefreshCw className="mr-2 h-4 w-4" />
                Refresh
              </Button>
            </div>

            <FirewallRuleList
              rules={rules}
              onRefresh={() => loadRules(selectedClusterId, nodeId)}
              onNewRule={handleNewRule}
            />
          </div>
        </TabsContent>

        <TabsContent value="guest-rules">
          <div className="space-y-4 pt-4">
            <div className="flex items-center gap-3 flex-wrap">
              <div className="flex items-center gap-2">
                <span className="text-sm text-muted-foreground">VM ID:</span>
                <Input
                  className="w-24 h-8 text-sm"
                  value={vmIdInput}
                  onChange={(e) => setVmIdInput(e.target.value)}
                  placeholder="e.g. 100"
                  type="number"
                />
              </div>
              <div className="flex items-center gap-2">
                <span className="text-sm text-muted-foreground">Node:</span>
                <Input
                  className="w-36 h-8 text-sm"
                  value={guestNodeInput}
                  onChange={(e) => setGuestNodeInput(e.target.value)}
                  placeholder="localhost"
                />
              </div>
              <Button
                size="sm"
                onClick={() => {
                  const id = parseInt(vmIdInput, 10);
                  if (isNaN(id)) { toast.error('Enter a valid VM ID'); return; }
                  setVmId(id);
                  setGuestNode(guestNodeInput.trim() || 'localhost');
                  loadGuestRules(selectedClusterId, guestNodeInput.trim() || 'localhost', id);
                }}
              >
                Load
              </Button>
            </div>

            {vmId !== null && (
              <div className="flex items-center justify-between">
                <p className="text-sm text-muted-foreground">
                  Rules for VM <span className="font-medium">{vmId}</span> on node <span className="font-medium">{guestNode}</span>
                </p>
                <Button size="sm" onClick={() => {
                  setGuestRuleAction('ACCEPT');
                  setGuestRuleProtocol('tcp');
                  setGuestRuleSource('');
                  setGuestRuleDest('');
                  setGuestRuleDport('');
                  setShowGuestRuleDialog(true);
                }}>
                  Add Rule
                </Button>
              </div>
            )}

            {guestLoading ? (
              <p className="text-sm text-muted-foreground">Loading guest rules...</p>
            ) : vmId === null ? (
              <p className="text-sm text-muted-foreground">Enter a VM ID and node, then click Load.</p>
            ) : guestRules.length === 0 ? (
              <p className="text-sm text-muted-foreground">No firewall rules found for this guest.</p>
            ) : (
              <div className="rounded-md border">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b">
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Pos</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Action</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Protocol</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Source</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Dest</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Port</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground">Enabled</th>
                      <th className="py-2 px-3 text-left font-medium text-muted-foreground"></th>
                    </tr>
                  </thead>
                  <tbody>
                    {guestRules.map((rule, i) => (
                      <tr key={i} className="border-b last:border-0 hover:bg-muted/50">
                        <td className="py-2 px-3">{rule.pos ?? i}</td>
                        <td className="py-2 px-3">{rule.action ?? '—'}</td>
                        <td className="py-2 px-3">{rule.proto ?? '—'}</td>
                        <td className="py-2 px-3">{rule.source ?? '—'}</td>
                        <td className="py-2 px-3">{rule.dest ?? '—'}</td>
                        <td className="py-2 px-3">{rule.dport ?? '—'}</td>
                        <td className="py-2 px-3">
                          <span className={rule.enable === 1 ? "text-xs bg-green-100 text-green-800 px-2 py-0.5 rounded" : "text-xs bg-gray-100 text-gray-600 px-2 py-0.5 rounded"}>
                            {rule.enable === 1 ? 'Yes' : 'No'}
                          </span>
                        </td>
                        <td className="py-2 px-3">
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-6 text-xs text-destructive hover:text-destructive"
                            onClick={() => handleDeleteGuestRule(rule.pos ?? i)}
                          >
                            Delete
                          </Button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </TabsContent>
      </Tabs>

      {/* Node rule dialog */}
      <Dialog open={showNewRuleDialog} onOpenChange={setShowNewRuleDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>New Firewall Rule</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="ruleAction">Action</Label>
              <Select value={ruleAction} onValueChange={setRuleAction}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="ACCEPT">ACCEPT</SelectItem>
                  <SelectItem value="DROP">DROP</SelectItem>
                  <SelectItem value="REJECT">REJECT</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="ruleProtocol">Protocol</Label>
              <Select value={ruleProtocol} onValueChange={setRuleProtocol}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="tcp">TCP</SelectItem>
                  <SelectItem value="udp">UDP</SelectItem>
                  <SelectItem value="icmp">ICMP</SelectItem>
                  <SelectItem value="any">Any</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="ruleSource">Source (optional)</Label>
              <Input
                id="ruleSource"
                value={ruleSource}
                onChange={(e) => setRuleSource(e.target.value)}
                placeholder="e.g. 192.168.1.0/24"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="ruleDest">Destination (optional)</Label>
              <Input
                id="ruleDest"
                value={ruleDest}
                onChange={(e) => setRuleDest(e.target.value)}
                placeholder="e.g. 10.0.0.1"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="ruleDport">Destination Port (optional)</Label>
              <Input
                id="ruleDport"
                value={ruleDport}
                onChange={(e) => setRuleDport(e.target.value)}
                placeholder="e.g. 80, 443, 8000:9000"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="ruleComment">Comment (optional)</Label>
              <Input
                id="ruleComment"
                value={ruleComment}
                onChange={(e) => setRuleComment(e.target.value)}
                placeholder="Rule description"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowNewRuleDialog(false)}>
              Cancel
            </Button>
            <Button onClick={handleSubmitNewRule}>
              Create Rule
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Guest rule dialog */}
      <Dialog open={showGuestRuleDialog} onOpenChange={setShowGuestRuleDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add Guest Firewall Rule</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>Action</Label>
              <Select value={guestRuleAction} onValueChange={setGuestRuleAction}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="ACCEPT">ACCEPT</SelectItem>
                  <SelectItem value="DROP">DROP</SelectItem>
                  <SelectItem value="REJECT">REJECT</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>Protocol</Label>
              <Select value={guestRuleProtocol} onValueChange={setGuestRuleProtocol}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="tcp">TCP</SelectItem>
                  <SelectItem value="udp">UDP</SelectItem>
                  <SelectItem value="icmp">ICMP</SelectItem>
                  <SelectItem value="any">Any</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>Source (optional)</Label>
              <Input
                value={guestRuleSource}
                onChange={(e) => setGuestRuleSource(e.target.value)}
                placeholder="e.g. 192.168.1.0/24"
              />
            </div>
            <div className="space-y-2">
              <Label>Destination (optional)</Label>
              <Input
                value={guestRuleDest}
                onChange={(e) => setGuestRuleDest(e.target.value)}
                placeholder="e.g. 10.0.0.1"
              />
            </div>
            <div className="space-y-2">
              <Label>Destination Port (optional)</Label>
              <Input
                value={guestRuleDport}
                onChange={(e) => setGuestRuleDport(e.target.value)}
                placeholder="e.g. 80, 443, 8000:9000"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowGuestRuleDialog(false)}>
              Cancel
            </Button>
            <Button onClick={handleSubmitGuestRule}>
              Add Rule
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
