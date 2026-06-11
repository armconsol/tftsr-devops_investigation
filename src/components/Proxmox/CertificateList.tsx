import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal, Trash2 } from 'lucide-react';

interface CertificateInfo {
  id: string;
  commonName: string;
  issuer: string;
  validFrom: string;
  validUntil: string;
  status: 'valid' | 'expiring' | 'expired';
}

interface CertificateListProps {
  certificates: CertificateInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onUpload?: () => void;
  onDelete?: (cert: CertificateInfo) => void;
  onRenew?: (cert: CertificateInfo) => void;
}

export function CertificateList({
  certificates,
  onRefresh,
  isLoading,
  onUpload,
  onDelete,
  onRenew,
}: CertificateListProps) {
  const validCount = certificates.filter((c) => c.status === 'valid').length;
  const expiringCount = certificates.filter((c) => c.status === 'expiring').length;
  const expiredCount = certificates.filter((c) => c.status === 'expired').length;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Certificates</CardTitle>
        <div className="flex space-x-2">
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-green-500">●</span>
            <span>{validCount} Valid</span>
          </div>
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-yellow-500">●</span>
            <span>{expiringCount} Expiring</span>
          </div>
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-red-500">●</span>
            <span>{expiredCount} Expired</span>
          </div>
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          <Button size="sm" onClick={onUpload}>
            <span className="mr-2 h-4 w-4">⬆️</span>
            Upload
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>ID</TableHead>
                <TableHead>Common Name</TableHead>
                <TableHead>Issuer</TableHead>
                <TableHead>Valid From</TableHead>
                <TableHead>Valid Until</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {certificates.map((cert) => (
                <TableRow key={cert.id}>
                  <TableCell className="font-medium">{cert.id}</TableCell>
                  <TableCell>{cert.commonName}</TableCell>
                  <TableCell>{cert.issuer}</TableCell>
                  <TableCell>{cert.validFrom}</TableCell>
                  <TableCell>{cert.validUntil}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      cert.status === 'valid' ? 'bg-green-100 text-green-800' :
                      cert.status === 'expiring' ? 'bg-yellow-100 text-yellow-800' :
                      'bg-red-100 text-red-800'
                    }`}>
                      {cert.status}
                    </span>
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onRenew?.(cert)}
                        title="Renew"
                      >
                        <span className="h-4 w-4 text-xs">🔄</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onDelete?.(cert)}
                        title="Delete"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        title="More"
                      >
                        <MoreHorizontal className="h-4 w-4" />
                      </button>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  );
}
