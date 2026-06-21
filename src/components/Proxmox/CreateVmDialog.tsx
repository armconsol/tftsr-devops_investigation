import React, { useState, useEffect } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { listProxmoxNodes, listProxmoxDatastores, createProxmoxVm } from '@/lib/proxmoxClient';
import { toast } from 'sonner';

interface CreateVmDialogProps {
  isOpen: boolean;
  clusterId: string;
  onClose: () => void;
  onCreated: () => void;
}

const OS_TYPES = [
  { value: 'l26', label: 'Linux 2.6+' },
  { value: 'l24', label: 'Linux 2.4' },
  { value: 'win11', label: 'Windows 11' },
  { value: 'win10', label: 'Windows 10/2016/2019' },
  { value: 'win8', label: 'Windows 8/2012' },
  { value: 'win7', label: 'Windows 7/2008' },
  { value: 'other', label: 'Other' },
];

export function CreateVmDialog({ isOpen, clusterId, onClose, onCreated }: CreateVmDialogProps) {
  const [nodes, setNodes] = useState<string[]>([]);
  const [storages, setStorages] = useState<string[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [nodeId, setNodeId] = useState('');
  const [vmid, setVmid] = useState(100);
  const [name, setName] = useState('');
  const [memory, setMemory] = useState(2048);
  const [cores, setCores] = useState(2);
  const [sockets, setSockets] = useState(1);
  const [osType, setOsType] = useState('l26');
  const [storage, setStorage] = useState('');
  const [diskSize, setDiskSize] = useState(20);
  const [netBridge, setNetBridge] = useState('vmbr0');
  const [iso, setIso] = useState('');

  useEffect(() => {
    if (!isOpen || !clusterId) return;

    listProxmoxNodes(clusterId)
      .then((data) => {
        const nodeNames = (data as Array<{ node?: string; status?: string }>)
          .filter((n) => n.status === 'online' || n.node)
          .map((n) => n.node ?? '')
          .filter(Boolean);
        setNodes(nodeNames);
        setNodeId(nodeNames[0] ?? '');
      })
      .catch(() => toast.error('Failed to load cluster nodes'));

    listProxmoxDatastores(clusterId)
      .then((data) => {
        const storageIds = (data as Array<{ storage?: string }>)
          .map((s) => s.storage ?? '')
          .filter(Boolean);
        setStorages(storageIds);
        setStorage(storageIds[0] ?? 'local-lvm');
      })
      .catch(() => {
        setStorages(['local-lvm', 'local']);
        setStorage('local-lvm');
      });
  }, [isOpen, clusterId]);

  const handleSubmit = async () => {
    if (!nodeId) { toast.error('Please select a target node'); return; }
    if (!name.trim()) { toast.error('VM name is required'); return; }
    if (vmid < 100 || vmid > 999999999) { toast.error('VMID must be between 100 and 999999999'); return; }

    setIsSubmitting(true);
    try {
      await createProxmoxVm(clusterId, {
        nodeId,
        vmid,
        name: name.trim(),
        memory,
        cores,
        sockets,
        osType,
        storage,
        diskSize,
        netBridge,
        iso: iso.trim() || undefined,
      });
      toast.success(`VM "${name}" created successfully (VMID: ${vmid})`);
      onCreated();
      handleClose();
    } catch (err) {
      toast.error(`Failed to create VM: ${err}`);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleClose = () => {
    setName('');
    setVmid(100);
    setMemory(2048);
    setCores(2);
    setSockets(1);
    setOsType('l26');
    setDiskSize(20);
    setNetBridge('vmbr0');
    setIso('');
    onClose();
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleClose}>
      <DialogContent className="max-w-lg max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Create Virtual Machine</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <Label htmlFor="vm-node">Node</Label>
              <Select value={nodeId} onValueChange={setNodeId}>
                <SelectTrigger>
                  <SelectValue placeholder="Select node" />
                </SelectTrigger>
                <SelectContent>
                  {nodes.map((n) => (
                    <SelectItem key={n} value={n}>{n}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <Label htmlFor="vm-vmid">VM ID</Label>
              <Input
                id="vm-vmid"
                type="number"
                min={100}
                max={999999999}
                value={vmid}
                onChange={(e) => setVmid(Number(e.target.value))}
              />
            </div>
          </div>

          <div className="space-y-1">
            <Label htmlFor="vm-name">Name</Label>
            <Input
              id="vm-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="my-vm"
            />
          </div>

          <div className="space-y-1">
            <Label htmlFor="vm-ostype">OS Type</Label>
            <Select value={osType} onValueChange={setOsType}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {OS_TYPES.map((o) => (
                  <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="grid grid-cols-3 gap-4">
            <div className="space-y-1">
              <Label htmlFor="vm-memory">Memory (MB)</Label>
              <Input
                id="vm-memory"
                type="number"
                min={256}
                step={256}
                value={memory}
                onChange={(e) => setMemory(Number(e.target.value))}
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="vm-cores">Cores</Label>
              <Input
                id="vm-cores"
                type="number"
                min={1}
                max={128}
                value={cores}
                onChange={(e) => setCores(Number(e.target.value))}
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="vm-sockets">Sockets</Label>
              <Input
                id="vm-sockets"
                type="number"
                min={1}
                max={4}
                value={sockets}
                onChange={(e) => setSockets(Number(e.target.value))}
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <Label htmlFor="vm-storage">Storage</Label>
              {storages.length > 0 ? (
                <Select value={storage} onValueChange={setStorage}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select storage" />
                  </SelectTrigger>
                  <SelectContent>
                    {storages.map((s) => (
                      <SelectItem key={s} value={s}>{s}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              ) : (
                <Input
                  id="vm-storage"
                  value={storage}
                  onChange={(e) => setStorage(e.target.value)}
                  placeholder="local-lvm"
                />
              )}
            </div>
            <div className="space-y-1">
              <Label htmlFor="vm-disksize">Disk Size (GB)</Label>
              <Input
                id="vm-disksize"
                type="number"
                min={1}
                value={diskSize}
                onChange={(e) => setDiskSize(Number(e.target.value))}
              />
            </div>
          </div>

          <div className="space-y-1">
            <Label htmlFor="vm-bridge">Network Bridge</Label>
            <Input
              id="vm-bridge"
              value={netBridge}
              onChange={(e) => setNetBridge(e.target.value)}
              placeholder="vmbr0"
            />
          </div>

          <div className="space-y-1">
            <Label htmlFor="vm-iso">ISO Image (optional)</Label>
            <Input
              id="vm-iso"
              value={iso}
              onChange={(e) => setIso(e.target.value)}
              placeholder="local:iso/ubuntu-24.04.iso"
            />
            <p className="text-xs text-muted-foreground">Format: storage:iso/filename.iso</p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={handleClose} disabled={isSubmitting}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={isSubmitting || !nodeId || !name.trim()}>
            {isSubmitting ? 'Creating...' : 'Create VM'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
