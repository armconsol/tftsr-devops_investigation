import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Badge } from '@/components/ui/index';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/index';
import { RefreshCw, ChevronDown, ChevronRight, RotateCcw } from 'lucide-react';
import { Certificate } from '@/lib/domain';

interface CertificateListProps {
  certificates: Certificate[];
  onRefresh: () => void;
  onRenew: (cert: Certificate) => void;
  isLoading?: boolean;
}

function certStatus(cert: Certificate): 'valid' | 'expiring' | 'expired' {
  if (!cert.notafter) return 'valid';
  const expiry = new Date(cert.notafter);
  const now = new Date();
  if (expiry < now) return 'expired';
  const thirtyDays = 30 * 24 * 60 * 60 * 1000;
  if (expiry.getTime() - now.getTime() < thirtyDays) return 'expiring';
  return 'valid';
}

function StatusBadge({ status }: { status: 'valid' | 'expiring' | 'expired' }) {
  if (status === 'valid') {
    return <Badge variant="success">Valid</Badge>;
  }
  if (status === 'expiring') {
    return (
      <Badge className="border-transparent bg-yellow-500 text-white">
        Expiring Soon
      </Badge>
    );
  }
  return <Badge variant="destructive">Expired</Badge>;
}

function truncateFingerprint(fp?: string): string {
  if (!fp) return '-';
  // Show first and last 8 hex chars separated by ellipsis
  const clean = fp.replace(/:/g, '');
  if (clean.length <= 16) return fp;
  return `${fp.slice(0, 8)}…${fp.slice(-8)}`;
}

function extractCN(subject: string): string {
  const match = subject.match(/CN=([^,/]+)/i);
  return match ? match[1] : subject;
}

export function CertificateList({
  certificates,
  onRefresh,
  onRenew,
  isLoading = false,
}: CertificateListProps) {
  const [expandedRows, setExpandedRows] = useState<Set<string>>(new Set());
  const [detailCert, setDetailCert] = useState<Certificate | null>(null);

  const validCount = certificates.filter((c) => certStatus(c) === 'valid').length;
  const expiringCount = certificates.filter((c) => certStatus(c) === 'expiring').length;
  const expiredCount = certificates.filter((c) => certStatus(c) === 'expired').length;

  function toggleRow(filename: string) {
    setExpandedRows((prev) => {
      const next = new Set(prev);
      if (next.has(filename)) {
        next.delete(filename);
      } else {
        next.add(filename);
      }
      return next;
    });
  }

  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle>Certificates</CardTitle>
          <div className="flex items-center space-x-3">
            <div className="flex items-center space-x-1 text-sm">
              <span className="h-2 w-2 rounded-full bg-green-500 inline-block" />
              <span>{validCount} Valid</span>
            </div>
            <div className="flex items-center space-x-1 text-sm">
              <span className="h-2 w-2 rounded-full bg-yellow-500 inline-block" />
              <span>{expiringCount} Expiring</span>
            </div>
            <div className="flex items-center space-x-1 text-sm">
              <span className="h-2 w-2 rounded-full bg-red-500 inline-block" />
              <span>{expiredCount} Expired</span>
            </div>
            <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
              <RefreshCw className={`mr-2 h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
              Refresh
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {certificates.length === 0 ? (
            <div className="flex items-center justify-center py-12 text-muted-foreground text-sm">
              No certificates found
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-6" />
                  <TableHead>Subject (CN)</TableHead>
                  <TableHead>SANs</TableHead>
                  <TableHead>Issuer</TableHead>
                  <TableHead>Valid From</TableHead>
                  <TableHead>Valid Until</TableHead>
                  <TableHead>Fingerprint</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {certificates.map((cert) => {
                  const status = certStatus(cert);
                  const isExpanded = expandedRows.has(cert.filename);
                  const rowClass =
                    status === 'expired'
                      ? 'bg-red-50/50 dark:bg-red-950/20'
                      : status === 'expiring'
                      ? 'bg-yellow-50/50 dark:bg-yellow-950/20'
                      : '';

                  return (
                    <React.Fragment key={cert.filename}>
                      <TableRow className={rowClass}>
                        <TableCell className="w-6 pr-0">
                          <button
                            onClick={() => toggleRow(cert.filename)}
                            className="rounded p-0.5 hover:bg-accent"
                            aria-label={isExpanded ? 'Collapse' : 'Expand'}
                          >
                            {isExpanded ? (
                              <ChevronDown className="h-4 w-4" />
                            ) : (
                              <ChevronRight className="h-4 w-4" />
                            )}
                          </button>
                        </TableCell>
                        <TableCell className="font-medium">
                          {extractCN(cert.subject)}
                        </TableCell>
                        <TableCell className="text-sm text-muted-foreground">
                          {cert.san && cert.san.length > 0
                            ? cert.san.slice(0, 2).join(', ') +
                              (cert.san.length > 2 ? ` +${cert.san.length - 2}` : '')
                            : '-'}
                        </TableCell>
                        <TableCell className="text-sm text-muted-foreground">
                          {cert.issuer ? extractCN(cert.issuer) : '-'}
                        </TableCell>
                        <TableCell className="text-sm">
                          {cert.notbefore ?? '-'}
                        </TableCell>
                        <TableCell className="text-sm">
                          {cert.notafter ?? '-'}
                        </TableCell>
                        <TableCell className="font-mono text-xs text-muted-foreground">
                          {truncateFingerprint(cert.fingerprint)}
                        </TableCell>
                        <TableCell>
                          <StatusBadge status={status} />
                        </TableCell>
                        <TableCell className="text-right">
                          <div className="flex items-center justify-end space-x-1">
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => setDetailCert(cert)}
                              title="View Details"
                            >
                              View
                            </Button>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => onRenew(cert)}
                              title="Renew certificate"
                            >
                              <RotateCcw className="mr-1 h-3 w-3" />
                              Renew
                            </Button>
                          </div>
                        </TableCell>
                      </TableRow>

                      {isExpanded && (
                        <TableRow className={rowClass}>
                          <TableCell colSpan={9} className="bg-muted/30 px-8 py-3">
                            <div className="grid grid-cols-2 gap-x-8 gap-y-1 text-sm">
                              <div>
                                <span className="font-medium text-muted-foreground">Filename: </span>
                                <span className="font-mono">{cert.filename}</span>
                              </div>
                              <div>
                                <span className="font-medium text-muted-foreground">Full Subject: </span>
                                <span>{cert.subject}</span>
                              </div>
                              {cert.issuer && (
                                <div>
                                  <span className="font-medium text-muted-foreground">Full Issuer: </span>
                                  <span>{cert.issuer}</span>
                                </div>
                              )}
                              {cert.fingerprint && (
                                <div>
                                  <span className="font-medium text-muted-foreground">Fingerprint: </span>
                                  <span className="font-mono text-xs">{cert.fingerprint}</span>
                                </div>
                              )}
                              {cert.san && cert.san.length > 0 && (
                                <div className="col-span-2">
                                  <span className="font-medium text-muted-foreground">All SANs: </span>
                                  <span>{cert.san.join(', ')}</span>
                                </div>
                              )}
                            </div>
                          </TableCell>
                        </TableRow>
                      )}
                    </React.Fragment>
                  );
                })}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Detail dialog */}
      <Dialog open={detailCert !== null} onOpenChange={(open) => { if (!open) setDetailCert(null); }}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Certificate Details</DialogTitle>
          </DialogHeader>
          {detailCert && (
            <div className="space-y-3 text-sm">
              <div className="grid grid-cols-[140px_1fr] gap-y-2">
                <span className="font-medium text-muted-foreground">Subject</span>
                <span>{detailCert.subject}</span>
                <span className="font-medium text-muted-foreground">Issuer</span>
                <span>{detailCert.issuer ?? '-'}</span>
                <span className="font-medium text-muted-foreground">Valid From</span>
                <span>{detailCert.notbefore ?? '-'}</span>
                <span className="font-medium text-muted-foreground">Valid Until</span>
                <span>{detailCert.notafter ?? '-'}</span>
                <span className="font-medium text-muted-foreground">Fingerprint</span>
                <span className="font-mono text-xs break-all">{detailCert.fingerprint ?? '-'}</span>
                <span className="font-medium text-muted-foreground">Filename</span>
                <span className="font-mono text-xs">{detailCert.filename}</span>
                {detailCert.san && detailCert.san.length > 0 && (
                  <>
                    <span className="font-medium text-muted-foreground">SANs</span>
                    <span>{detailCert.san.join(', ')}</span>
                  </>
                )}
                {detailCert.pem && (
                  <>
                    <span className="font-medium text-muted-foreground self-start pt-1">PEM</span>
                    <pre className="overflow-auto rounded bg-muted p-2 text-xs max-h-48">
                      {detailCert.pem}
                    </pre>
                  </>
                )}
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
}
