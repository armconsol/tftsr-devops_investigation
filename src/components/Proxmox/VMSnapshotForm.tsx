import React from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Checkbox } from '@/components/ui/index';

interface SnapshotFormProps {
  vmName: string;
  vmID: number;
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (name: string, description: string, memory: boolean, quiesce: boolean) => void;
}

export function SnapshotForm({
  vmName,
  vmID,
  isOpen,
  onClose,
  onSubmit,
}: SnapshotFormProps) {
  const [name, setName] = React.useState('');
  const [description, setDescription] = React.useState('');
  const [memory, setMemory] = React.useState(false);
  const [quiesce, setQuiesce] = React.useState(false);

  const handleSubmit = () => {
    onSubmit(name, description, memory, quiesce);
    onClose();
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create Snapshot for {vmName} (VM {vmID})</DialogTitle>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="name">Snapshot Name</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., pre-update-backup"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="description">Description</Label>
            <Input
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Optional description"
            />
          </div>
          <div className="space-y-2">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="memory"
                checked={memory}
                onCheckedChange={(checked) => setMemory(checked as boolean)}
              />
              <Label htmlFor="memory">Include Memory</Label>
            </div>
            <p className="text-xs text-muted-foreground">
              Include the VM's memory state in the snapshot
            </p>
          </div>
          <div className="space-y-2">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="quiesce"
                checked={quiesce}
                onCheckedChange={(checked) => setQuiesce(checked as boolean)}
              />
              <Label htmlFor="quiesce">Quiesce (freeze filesystem)</Label>
            </div>
            <p className="text-xs text-muted-foreground">
              Freeze filesystem before snapshot for consistency
            </p>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={!name}>
            Create Snapshot
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
