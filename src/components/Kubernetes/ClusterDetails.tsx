import React from "react";
import { Badge } from "@/components/ui";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";

interface ClusterDetailsProps {
  clusterId: string;
}

export function ClusterDetails({ clusterId }: ClusterDetailsProps) {
  return (
    <div className="h-full overflow-y-auto">
      <div className="mb-6">
        <h2 className="text-2xl font-semibold">Cluster Details</h2>
        <p className="text-muted-foreground">Cluster ID: {clusterId}</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-card rounded-lg border">
          <div className="border-b px-6 py-4">
            <h3 className="font-semibold">Basic Information</h3>
          </div>
          <div className="p-6">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <span className="text-sm text-muted-foreground">Name</span>
                <p className="font-medium">production-cluster</p>
              </div>
              <div>
                <span className="text-sm text-muted-foreground">Region</span>
                <p className="font-medium">us-east-1</p>
              </div>
              <div>
                <span className="text-sm text-muted-foreground">Kubernetes Version</span>
                <p className="font-mono">v1.28.4</p>
              </div>
              <div>
                <span className="text-sm text-muted-foreground">Platform</span>
                <p className="font-medium">EKS</p>
              </div>
              <div>
                <span className="text-sm text-muted-foreground">API Server</span>
                <p className="font-mono text-xs truncate">https://abc123.gr7.us-east-1.eks.amazonaws.com</p>
              </div>
              <div>
                <span className="text-sm text-muted-foreground">Status</span>
                <Badge variant="default">Running</Badge>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-card rounded-lg border">
          <div className="border-b px-6 py-4">
            <h3 className="font-semibold">Network Configuration</h3>
          </div>
          <div className="p-6">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <span className="text-sm text-muted-foreground">VPC ID</span>
                <p className="font-mono">vpc-0abc123def456</p>
              </div>
              <div>
                <span className="text-sm text-muted-foreground">Subnets</span>
                <div className="flex flex-wrap gap-1">
                  <Badge variant="secondary">subnet-1</Badge>
                  <Badge variant="secondary">subnet-2</Badge>
                  <Badge variant="secondary">subnet-3</Badge>
                </div>
              </div>
              <div>
                <span className="text-sm text-muted-foreground">Security Groups</span>
                <div className="flex flex-wrap gap-1">
                  <Badge variant="secondary">sg-001</Badge>
                  <Badge variant="secondary">sg-002</Badge>
                </div>
              </div>
              <div>
                <span className="text-sm text-muted-foreground">CIDR Block</span>
                <p className="font-mono">10.0.0.0/16</p>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-card rounded-lg border">
          <div className="border-b px-6 py-4">
            <h3 className="font-semibold">Node Configuration</h3>
          </div>
          <div className="p-6">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <span className="text-sm text-muted-foreground">Instance Type</span>
                <p className="font-medium">m5.xlarge</p>
              </div>
              <div>
                <span className="text-sm text-muted-foreground">Min Nodes</span>
                <p className="font-medium">3</p>
              </div>
              <div>
                <span className="text-sm text-muted-foreground">Max Nodes</span>
                <p className="font-medium">10</p>
              </div>
              <div>
                <span className="text-sm text-muted-foreground">Autoscaling</span>
                <Badge variant="default">Enabled</Badge>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-card rounded-lg border">
          <div className="border-b px-6 py-4">
            <h3 className="font-semibold">Security Configuration</h3>
          </div>
          <div className="p-6">
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Network Policy</span>
                <Badge variant="default">Enabled</Badge>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Pod Security Policy</span>
                <Badge variant="default">Enabled</Badge>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">RBAC</span>
                <Badge variant="default">Enabled</Badge>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Secret Encryption</span>
                <Badge variant="default">Enabled</Badge>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="bg-card rounded-lg border mt-6">
        <div className="border-b px-6 py-4">
          <h3 className="font-semibold">Node Pools</h3>
        </div>
        <div className="p-6">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Instance Type</TableHead>
                <TableHead>Nodes</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Auto-scaling</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow>
                <TableCell>general-purpose</TableCell>
                <TableCell className="font-mono">m5.xlarge</TableCell>
                <TableCell>3</TableCell>
                <TableCell>Running</TableCell>
                <TableCell>Enabled</TableCell>
              </TableRow>
              <TableRow>
                <TableCell>compute-optimized</TableCell>
                <TableCell className="font-mono">c5.2xlarge</TableCell>
                <TableCell>2</TableCell>
                <TableCell>Running</TableCell>
                <TableCell>Enabled</TableCell>
              </TableRow>
              <TableRow>
                <TableCell>memory-optimized</TableCell>
                <TableCell className="font-mono">r5.4xlarge</TableCell>
                <TableCell>2</TableCell>
                <TableCell>Running</TableCell>
                <TableCell>Enabled</TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>
      </div>
    </div>
  );
}
