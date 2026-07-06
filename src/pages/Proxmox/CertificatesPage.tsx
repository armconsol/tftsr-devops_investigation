import React, { useState, useEffect, useRef } from 'react';
import { Card, CardContent } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/index';
import { RefreshCw, Upload, ShieldCheck } from 'lucide-react';
import { toast } from 'sonner';
import { CertificateList } from '@/components/Proxmox';
import {
  listCertificates,
  uploadCertificate,
  listAcmeAccounts,
  registerAcmeAccount,
  requestAcmeCertificate,
} from '@/lib/proxmoxClient';
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
  const [uploadPending, setUploadPending] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // ACME dialog state
  const [acmeOpen, setAcmeOpen] = useState(false);
  const [acmeDomain, setAcmeDomain] = useState('');
  const [acmeEmail, setAcmeEmail] = useState('');
  const [acmePending, setAcmePending] = useState(false);

  useEffect(() => {
    if (!selectedClusterId) return;
    void fetchCerts();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedClusterId, nodeId]);

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

  async function resolveAcmeAccountId(email: string): Promise<string> {
    const accounts = await listAcmeAccounts(selectedClusterId);
    if (accounts.length > 0) {
      return String(accounts[0].account_id);
    }
    if (!email.trim()) {
      throw new Error('No ACME account exists yet — enter an email to register one');
    }
    const account = await registerAcmeAccount(selectedClusterId, email.trim(), true);
    return String(account.account_id);
  }

  async function handleRenew(cert: Certificate) {
    const domain = cert.san?.[0] ?? cert.subject;
    if (!domain) {
      toast.error('Certificate has no domain to renew');
      return;
    }
    try {
      const accountId = await resolveAcmeAccountId('');
      await requestAcmeCertificate(selectedClusterId, domain, accountId);
      toast.success(`Renewal requested for ${domain}`);
    } catch (err) {
      toast.error(`Failed to renew certificate: ${err}`);
    } finally {
      await fetchCerts();
    }
  }

  async function handleUpload() {
    setUploadPending(true);
    try {
      await uploadCertificate(selectedClusterId, uploadCertPem.trim(), uploadKeyPem.trim());
      toast.success('Certificate uploaded');
      setUploadOpen(false);
      setUploadCertPem('');
      setUploadKeyPem('');
      await fetchCerts();
    } catch (err) {
      toast.error(`Failed to upload certificate: ${err}`);
    } finally {
      setUploadPending(false);
    }
  }

  async function handleOrderCertificate() {
    setAcmePending(true);
    try {
      const accountId = await resolveAcmeAccountId(acmeEmail);
      await requestAcmeCertificate(selectedClusterId, acmeDomain.trim(), accountId);
      toast.success(`Certificate requested for ${acmeDomain}`);
      setAcmeOpen(false);
      setAcmeDomain('');
      setAcmeEmail('');
      await fetchCerts();
    } catch (err) {
      toast.error(`Failed to order certificate: ${err}`);
    } finally {
      setAcmePending(false);
    }
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
              disabled={!uploadCertPem.trim() || !uploadKeyPem.trim() || uploadPending}
              onClick={() => void handleUpload()}
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
            <div className="space-y-2">
              <Label>ACME Account Email (only needed if no account exists yet)</Label>
              <Input
                placeholder="admin@example.com"
                value={acmeEmail}
                onChange={(e) => setAcmeEmail(e.target.value)}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAcmeOpen(false)}>
              Cancel
            </Button>
            <Button
              disabled={!acmeDomain.trim() || acmePending}
              onClick={() => void handleOrderCertificate()}
            >
              Order Certificate
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
