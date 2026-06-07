import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { NetworkPolicyInfo } from "@/lib/tauriCommands";

interface NetworkPolicyListProps {
  networkpolicies: NetworkPolicyInfo[];
  clusterId: string;
  namespace: string;
}

export function NetworkPolicyList({ networkpolicies }: NetworkPolicyListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Pod Selector</TableHead>
            <TableHead>Policy Types</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {networkpolicies.length === 0 ? (
            <TableRow>
              <TableCell colSpan={5} className="text-center text-muted-foreground">
                No network policies found
              </TableCell>
            </TableRow>
          ) : (
            networkpolicies.map((np) => (
              <TableRow key={`${np.name}-${np.namespace}`}>
                <TableCell className="font-medium">{np.name}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{np.namespace}</TableCell>
                <TableCell className="text-sm font-mono truncate max-w-48">{np.pod_selector}</TableCell>
                <TableCell className="text-sm">{np.policy_types.join(", ") || "—"}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{np.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
