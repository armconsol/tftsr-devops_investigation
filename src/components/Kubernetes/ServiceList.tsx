import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Pencil, Trash2 } from "lucide-react";
import type { ServiceInfo } from "@/lib/tauriCommands";
import { deleteResourceCmd, getResourceYamlCmd } from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface ServiceListProps {
  services: ServiceInfo[];
  clusterId: string;
  namespace: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "edit"; svc: ServiceInfo; yaml: string }
  | { type: "delete"; svc: ServiceInfo }
  | null;

export function ServiceList({ services, clusterId, namespace, onRefresh }: ServiceListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

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

  const openEdit = async (svc: ServiceInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "services", namespace, svc.name);
      setActiveModal({ type: "edit", svc, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDelete = async () => {
    if (activeModal?.type !== "delete") return;
    setIsDeleting(true);
    try {
      await deleteResourceCmd(clusterId, "services", namespace, activeModal.svc.name);
      setActiveModal(null);
      onRefresh?.();
    } finally {
      setIsDeleting(false);
    }
  };

  return (
    <>
      {actionError && (
        <p className="mb-2 text-sm text-destructive">{actionError}</p>
      )}
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
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {services.length === 0 ? (
              <TableRow>
                <TableCell colSpan={7} className="text-center text-muted-foreground">
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
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(service),
                        },
                        {
                          label: "Delete",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "delete", svc: service }),
                        },
                      ]}
                    />
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      {activeModal?.type === "edit" && (
        <EditResourceModal
          isOpen
          clusterId={clusterId}
          namespace={namespace}
          resourceType="services"
          resourceName={activeModal.svc.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}

      {activeModal?.type === "delete" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="Service"
          resourceName={activeModal.svc.name}
          isLoading={isDeleting}
          onConfirm={handleDelete}
        />
      )}
    </>
  );
}
