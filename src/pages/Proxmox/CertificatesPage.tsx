import React, { useState, useEffect, useRef } from 'react';
import { Card, CardContent } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { RefreshCw, Upload, ShieldCheck } from 'lucide-react';
import { CertificateList } from '@/components/Proxmox';
import { listCertificates } from '@/lib/proxmoxClient';
import { Certificate } from '@/lib/domain';
import { useProxmoxClusters } from '@/hooks/useProxmoxClusters';

export function ProxmoxCertificatesPage() {
  const { clusters, selectedClusterId, setSelectedClusterId } = useProxmoxClusters();
  const [nodeId, setNodeId] = useState<string>('pve');
  const [certificates, setCertificates] = useState<Certificate[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Upload dialog state
  const [uploadOpen, setUploadOpen] = useState(false);
  const [uploadCertPem, setUploadCertPem] = useState('');
  const [uploadKeyPem, setUploadKeyPem] = useState('');
  const fileInputRef = useRef<HTMLInputElement>(null);

  // ACME dialog state
  const [acmeOpen, setAcmeOpen] = useState(false);
  const [acmeDomain, setAcmeDomain] = useState('');

  useEffect(() => {
    if (!selectedClusterId) return;
    void fetchCerts();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedClusterId]);

  async function fetchCerts() {
    setLoading(true);
    setError(null);
    try {
      const raw = await listCertificates(selectedClusterId, nodeId);
      const mapped: Certificate[] = (raw as Record<string, unknown>[]).map((c) => ({
        filename: String(c['filename'] ?? c['subject'] ?? 'unknown'),
        subject: String(c['subject'] ?? ''),
        san: Array.isArray(c['san']) ? (c['san'] as string[]) : undefined,
        issuer: c['issuer'] != null ? String(c['issuer']) : undefined,
        notbefore: c['notbefore'] != null ? String(c['notbefore']) : undefined,
        notafter: c['notafter'] != null ? String(c['notafter']) : undefined,
        fingerprint: c['fingerprint'] != null ? String(c['fingerprint']) : undefined,
        pem: c['pem'] != null ? String(c['pem']) : undefined,
      }));
      setCertificates(mapped);
    } catch (err) {
      setError(String(err));
      setCertificates([]);
    } finally {
      setLoading(false);
    }
  }

  function handleRenew(_cert: Certificate) {

    void fetchCerts();
  }

  function handleFileChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      setUploadCertPem(String(ev.target?.result ?? ''));
    };
    reader.readAsText(file);
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Certificates</h1>
          <p className="text-muted-foreground">Manage TLS certificates across clusters</p>
        </div>
        <div className="flex items-center space-x-2">
          {clusters.length > 1 && (
            <Select value={selectedClusterId} onValueChange={setSelectedClusterId}>
              <SelectTrigger className="w-48">
                <SelectValue placeholder="Select cluster" />
              </SelectTrigger>
              <SelectContent>
                {clusters.map((c) => (
                  <SelectItem key={c.id} value={c.id}>
                    {c.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
          <Button variant="outline" size="sm" onClick={fetchCerts} disabled={loading || !selectedClusterId}>
            <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          <Button variant="outline" size="sm" onClick={() => setAcmeOpen(true)}>
            <ShieldCheck className="mr-2 h-4 w-4" />
            Order via ACME
          </Button>
          <Button size="sm" onClick={() => setUploadOpen(true)}>
            <Upload className="mr-2 h-4 w-4" />
            Upload Certificate
          </Button>
        </div>
      </div>

      {error && (
        <div className="rounded-md border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
          {error}
        </div>
      )}

      {!selectedClusterId && clusters.length === 0 && !loading && (
        <Card>
          <CardContent className="flex items-center justify-center py-12 text-muted-foreground text-sm">
            No clusters configured. Add a cluster in Remotes first.
          </CardContent>
        </Card>
      )}

      {selectedClusterId && (
        <CertificateList
          certificates={certificates}
          onRefresh={fetchCerts}
          onRenew={handleRenew}
          isLoading={loading}
        />
      )}

      {/* Upload Certificate Dialog */}
      <Dialog open={uploadOpen} onOpenChange={setUploadOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Upload Custom Certificate</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label>Certificate File (.pem / .crt)</Label>
              <Input
                ref={fileInputRef}
                type="file"
                accept=".pem,.crt,.cer"
                onChange={handleFileChange}
              />
            </div>
            <div className="space-y-2">
              <Label>Certificate PEM</Label>
              <textarea
                className="flex min-h-[100px] w-full rounded-md border border-input bg-background px-3 py-2 text-xs font-mono ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 resize-y"
                placeholder="-----BEGIN CERTIFICATE-----"
                value={uploadCertPem}
                onChange={(e) => setUploadCertPem(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label>Private Key PEM</Label>
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-xs font-mono ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 resize-y"
                placeholder="-----BEGIN PRIVATE KEY-----"
                value={uploadKeyPem}
                onChange={(e) => setUploadKeyPem(e.target.value)}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setUploadOpen(false)}>
              Cancel
            </Button>
            <Button
              disabled={!uploadCertPem.trim()}
              onClick={() => {

                setUploadOpen(false);
                setUploadCertPem('');
                setUploadKeyPem('');
                void fetchCerts();
              }}
            >
              Upload
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ACME Dialog */}
      <Dialog open={acmeOpen} onOpenChange={setAcmeOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Order Certificate via ACME</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Request a certificate from an ACME provider for the selected cluster node.
            </p>
            <div className="space-y-2">
              <Label>Domain / Node</Label>
              <Input
                placeholder="e.g. pve.example.com"
                value={acmeDomain}
                onChange={(e) => setAcmeDomain(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label>Node ID</Label>
              <Input
                placeholder="pve"
                value={nodeId}
                onChange={(e) => setNodeId(e.target.value)}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAcmeOpen(false)}>
              Cancel
            </Button>
            <Button
              disabled={!acmeDomain.trim()}
              onClick={() => {

                setAcmeOpen(false);
                setAcmeDomain('');
              }}
            >
              Order Certificate
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
