import React, { useState, useEffect } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Upload } from 'lucide-react';
import { open as openFileDialog } from '@tauri-apps/plugin-dialog';
import {
  listProxmoxClusters,
  listProxmoxNodes,
  listProxmoxStorages,
  listIsoImages,
  uploadIsoImage,
  createProxmoxVm,
} from '@/lib/proxmoxClient';
import type { ClusterInfo } from '@/lib/domain';
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
  const [clusters, setClusters] = useState<ClusterInfo[]>([]);
  const [selectedClusterId, setSelectedClusterId] = useState(clusterId);
  const [nodes, setNodes] = useState<string[]>([]);
  const [storages, setStorages] = useState<string[]>([]);
  const [isoStorages, setIsoStorages] = useState<string[]>([]);
  const [isoImages, setIsoImages] = useState<{ volid: string; name?: string }[]>([]);
  const [isoStorage, setIsoStorage] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isUploading, setIsUploading] = useState(false);

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
    if (!isOpen) return;
    listProxmoxClusters()
      .then((cls) => {
        setClusters(cls);
        const target = cls.find((c) => c.id === clusterId) ? clusterId : cls[0]?.id ?? clusterId;
        setSelectedClusterId(target);
      })
      .catch(console.error);
  }, [isOpen, clusterId]);

  useEffect(() => {
    if (!isOpen || !selectedClusterId) return;

    listProxmoxNodes(selectedClusterId)
      .then((data) => {
        const nodeNames = (data as Array<{ node?: string; status?: string }>)
          .filter((n) => n.status === 'online' || n.node)
          .map((n) => n.node ?? '')
          .filter(Boolean);
        setNodes(nodeNames);
        setNodeId(nodeNames[0] ?? '');
      })
      .catch(() => toast.error('Failed to load cluster nodes'));
  }, [isOpen, selectedClusterId]);

  useEffect(() => {
    if (!isOpen || !selectedClusterId || !nodeId) return;

    listProxmoxStorages(selectedClusterId, nodeId)
      .then((data) => {
        const storageIds = data.map((s) => s.storage).filter(Boolean);
        setStorages(storageIds);
        setStorage(storageIds[0] ?? 'local-lvm');

        const isoCapable = data
          .filter((s) => !s.content || s.content.includes('iso'))
          .map((s) => s.storage)
          .filter(Boolean);
        setIsoStorages(isoCapable);
        setIsoStorage(isoCapable[0] ?? '');
      })
      .catch(() => {
        setStorages(['local-lvm', 'local']);
        setStorage('local-lvm');
      });
  }, [isOpen, selectedClusterId, nodeId]);

  useEffect(() => {
    if (!isOpen || !selectedClusterId || !nodeId || !isoStorage) {
      setIsoImages([]);
      return;
    }

    listIsoImages(selectedClusterId, nodeId, isoStorage)
      .then((imgs) => {
        setIsoImages(imgs);
      })
      .catch(() => setIsoImages([]));
  }, [isOpen, selectedClusterId, nodeId, isoStorage]);

  const handleUploadIso = async () => {
    if (!selectedClusterId || !nodeId || !isoStorage) {
      toast.error('Select a cluster, node, and ISO storage before uploading');
      return;
    }
    const selected = await openFileDialog({
      title: 'Select ISO file',
      filters: [{ name: 'ISO Images', extensions: ['iso'] }],
      multiple: false,
    });
    if (!selected) return;

    const filePath = selected as string;
    setIsUploading(true);
    try {
      await uploadIsoImage(selectedClusterId, nodeId, isoStorage, filePath);
      toast.success('ISO upload started — refreshing image list');
      const imgs = await listIsoImages(selectedClusterId, nodeId, isoStorage);
      setIsoImages(imgs);
    } catch (e) {
      toast.error(`Upload failed: ${e}`);
    } finally {
      setIsUploading(false);
    }
  };

  const handleSubmit = async () => {
    if (!nodeId) { toast.error('Please select a target node'); return; }
    if (!name.trim()) { toast.error('VM name is required'); return; }
    if (vmid < 100 || vmid > 999999999) { toast.error('VMID must be between 100 and 999999999'); return; }

    setIsSubmitting(true);
    try {
      await createProxmoxVm(selectedClusterId, {
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
        iso: iso || undefined,
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

  const isoLabel = (volid: string, imgName?: string) => {
    const filename = imgName ?? volid.split('/').pop() ?? volid;
    return filename;
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleClose}>
      <DialogContent className="max-w-lg max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Create Virtual Machine</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2">
          {clusters.length > 1 && (
            <div className="space-y-1">
              <Label>Datacenter / Cluster</Label>
              <Select value={selectedClusterId} onValueChange={setSelectedClusterId}>
                <SelectTrigger>
                  <SelectValue placeholder="Select cluster" />
                </SelectTrigger>
                <SelectContent>
                  {clusters.map((c) => (
                    <SelectItem key={c.id} value={c.id}>{c.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <Label>Node</Label>
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
            <Label>OS Type</Label>
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
                max={512}
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
              <Label>Storage</Label>
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

          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label>ISO Image (optional)</Label>
              {isoStorage && (
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => void handleUploadIso()}
                  disabled={isUploading || !nodeId}
                  className="h-7 text-xs"
                >
                  <Upload className="mr-1.5 h-3 w-3" />
                  {isUploading ? 'Uploading...' : 'Upload ISO'}
                </Button>
              )}
            </div>
            {isoStorages.length > 0 && (
              <div className="space-y-1">
                <Label className="text-xs text-muted-foreground">Storage</Label>
                <Select value={isoStorage} onValueChange={setIsoStorage}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select ISO storage" />
                  </SelectTrigger>
                  <SelectContent>
                    {isoStorages.map((s) => (
                      <SelectItem key={s} value={s}>{s}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}
            {isoImages.length > 0 ? (
              <Select value={iso} onValueChange={setIso}>
                <SelectTrigger>
                  <SelectValue placeholder="Select ISO (optional)" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">— None —</SelectItem>
                  {isoImages.map((img) => (
                    <SelectItem key={img.volid} value={img.volid}>
                      {isoLabel(img.volid, img.name)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : (
              <Input
                value={iso}
                onChange={(e) => setIso(e.target.value)}
                placeholder="local:iso/ubuntu-24.04.iso"
              />
            )}
            <p className="text-xs text-muted-foreground">
              {isoImages.length > 0
                ? `${isoImages.length} ISO(s) available`
                : 'Format: storage:iso/filename'}
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={handleClose} disabled={isSubmitting}>
            Cancel
          </Button>
          <Button
            onClick={() => void handleSubmit()}
            disabled={isSubmitting || !nodeId || !name.trim()}
          >
            {isSubmitting ? 'Creating...' : 'Create VM'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
