import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Badge } from "@/components/ui";
import type { ServiceInfo } from "@/lib/tauriCommands";

interface ServiceListProps {
  services: ServiceInfo[];
  clusterId: string;
  namespace: string;
}

export function ServiceList({ services, clusterId: _clusterId, namespace: _namespace }: ServiceListProps) {
  const getServiceTypeColor = (type: string) => {
    switch (type.toLowerCase()) {
      case "clusterip":
        return "bg-blue-500";
      case "nodeport":
        return "bg-purple-500";
      case "loadbalancer":
        return "bg-green-500";
      case "externalname":
        return "bg-gray-500";
      default:
        return "bg-gray-500";
    }
  };

  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Cluster IP</TableHead>
            <TableHead>External IP</TableHead>
            <TableHead>Ports</TableHead>
            <TableHead>Age</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {services.length === 0 ? (
            <TableRow>
              <TableCell colSpan={6} className="text-center text-muted-foreground">
                No services found
              </TableCell>
            </TableRow>
          ) : (
            services.map((service) => (
              <TableRow key={`${service.name}-${service.namespace}`}>
                <TableCell className="font-medium">{service.name}</TableCell>
                <TableCell>
                  <Badge className={`${getServiceTypeColor(service.type)} text-white`}>
                    {service.type}
                  </Badge>
                </TableCell>
                <TableCell className="font-mono text-sm">{service.cluster_ip}</TableCell>
                <TableCell className="font-mono text-sm">
                  {service.external_ip || "N/A"}
                </TableCell>
                <TableCell>
                  <div className="space-y-1">
                    {service.ports.map((port) => (
                      <div key={`${port.port}-${port.protocol}`} className="text-sm">
                        {port.name ? `${port.name}: ` : ""}
                        {port.port}/{port.protocol}
                        {port.target_port && ` → ${port.target_port}`}
                      </div>
                    ))}
                  </div>
                </TableCell>
                <TableCell className="text-muted-foreground">{service.age}</TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
