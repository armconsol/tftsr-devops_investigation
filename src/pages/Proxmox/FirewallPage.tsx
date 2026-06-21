import React, { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { FirewallRuleList } from '@/components/Proxmox';
import { listProxmoxClusters, listFirewallRules, addFirewallRule } from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
import { toast } from 'sonner';

export function ProxmoxFirewallPage() {
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [selectedClusterId, setSelectedClusterId] = useState<string>('');
  const [nodeInputValue, setNodeInputValue] = useState('localhost');
  const [nodeId, setNodeId] = useState('localhost');
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [rules, setRules] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // New rule dialog
  const [showNewRuleDialog, setShowNewRuleDialog] = useState(false);
  const [ruleAction, setRuleAction] = useState('ACCEPT');
  const [ruleProtocol, setRuleProtocol] = useState('tcp');
  const [ruleSource, setRuleSource] = useState('');
  const [ruleDest, setRuleDest] = useState('');
  const [ruleDport, setRuleDport] = useState('');
  const [ruleComment, setRuleComment] = useState('');

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

  useEffect(() => {
    if (selectedClusterId) loadRules(selectedClusterId, nodeId);
  }, [selectedClusterId, nodeId, loadRules]);

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

      <div className="flex items-center gap-3 flex-wrap">
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
    </div>
  );
}
