import React from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Checkbox } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { AlertCircle } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/index';

interface VM {
  id: string;
  name: string;
  vmid: number;
  node: string;
  cluster: string;
}

interface MigrationFormProps {
  vm: VM;
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (targetNode: string, targetCluster: string, online: boolean, maxDowntime: number) => void;
  availableNodes: VM[];
}

export function MigrationForm({
  vm,
  isOpen,
  onClose,
  onSubmit,
  availableNodes,
}: MigrationFormProps) {
  const [targetNode, setTargetNode] = React.useState('');
  const [targetCluster, setTargetCluster] = React.useState('');
  const [online, setOnline] = React.useState(true);
  const [maxDowntime, setMaxDowntime] = React.useState(30);

  const handleSubmit = () => {
    onSubmit(targetNode, targetCluster, online, maxDowntime);
    onClose();
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Migrate {vm.name} (VM {vm.vmid})</DialogTitle>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Live migration requires the same hardware configuration on both nodes
            </AlertDescription>
          </Alert>
          <div className="space-y-2">
            <Label htmlFor="targetNode">Target Node</Label>
            <Select value={targetNode} onValueChange={setTargetNode}>
              <SelectTrigger>
                <SelectValue placeholder="Select target node" />
              </SelectTrigger>
              <SelectContent>
                {availableNodes
                  .filter((n) => n.id !== vm.id)
                  .map((node) => (
                    <SelectItem key={node.id} value={node.id}>
                      {node.name} ({node.node})
                    </SelectItem>
                  ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="targetCluster">Target Cluster</Label>
            <Select value={targetCluster} onValueChange={setTargetCluster}>
              <SelectTrigger>
                <SelectValue placeholder="Select target cluster" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="same">Same Cluster</SelectItem>
                <SelectItem value="different">Different Cluster</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="online"
                checked={online}
                onCheckedChange={(checked) => setOnline(checked as boolean)}
              />
              <Label htmlFor="online">Live Migration</Label>
            </div>
            <p className="text-xs text-muted-foreground">
              Keep VM running during migration
            </p>
          </div>
          <div className="space-y-2">
            <Label htmlFor="maxDowntime">Max Downtime (ms)</Label>
            <Input
              id="maxDowntime"
              type="number"
              value={maxDowntime}
              onChange={(e) => setMaxDowntime(Number(e.target.value))}
              min={10}
              max={1000}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={!targetNode}>
            Start Migration
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
