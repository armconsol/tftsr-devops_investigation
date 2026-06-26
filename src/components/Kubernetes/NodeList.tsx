import React, { useState } from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Badge } from "@/components/ui";
import { ShieldOff, ShieldCheck, Trash2, Pencil } from "lucide-react";
import type { NodeInfo } from "@/lib/tauriCommands";
import {
  cordonNodeCmd,
  uncordonNodeCmd,
  drainNodeCmd,
  getResourceYamlCmd,
} from "@/lib/tauriCommands";
import { ResourceActionMenu } from "./ResourceActionMenu";
import { ConfirmDeleteDialog } from "./ConfirmDeleteDialog";
import { EditResourceModal } from "./EditResourceModal";

interface NodeListProps {
  nodes: NodeInfo[];
  clusterId: string;
  onRefresh?: () => void;
}

type ActiveModal =
  | { type: "drain"; node: NodeInfo }
  | { type: "edit"; node: NodeInfo; yaml: string }
  | null;

export function NodeList({ nodes, clusterId, onRefresh }: NodeListProps) {
  const [activeModal, setActiveModal] = useState<ActiveModal>(null);
  const [isActing, setIsActing] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const getNodeStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case "ready":
        return "bg-green-500";
      case "notready":
        return "bg-red-500";
      case "schedulingdisabled":
        return "bg-yellow-500";
      default:
        return "bg-gray-500";
    }
  };

  const isSchedulingDisabled = (node: NodeInfo) =>
    node.status.toLowerCase().includes("schedulingdisabled") ||
    node.roles.toLowerCase().includes("schedulingdisabled");

  const handleCordon = async (node: NodeInfo) => {
    setActionError(null);
    try {
      await cordonNodeCmd(clusterId, node.name);
      onRefresh?.();
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleUncordon = async (node: NodeInfo) => {
    setActionError(null);
    try {
      await uncordonNodeCmd(clusterId, node.name);
      onRefresh?.();
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDrain = async () => {
    if (activeModal?.type !== "drain") return;
    setIsActing(true);
    try {
      await drainNodeCmd(clusterId, activeModal.node.name);
      setActiveModal(null);
      onRefresh?.();
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsActing(false);
    }
  };

  const openEdit = async (node: NodeInfo) => {
    setActionError(null);
    try {
      const yaml = await getResourceYamlCmd(clusterId, "nodes", "", node.name);
      setActiveModal({ type: "edit", node, yaml });
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
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
              <TableHead>Status</TableHead>
              <TableHead>Roles</TableHead>
              <TableHead>Version</TableHead>
              <TableHead>Internal IP</TableHead>
              <TableHead>OS Image</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {nodes.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center text-muted-foreground">
                  No nodes found
                </TableCell>
              </TableRow>
            ) : (
              nodes.map((node) => (
                <TableRow key={node.name}>
                  <TableCell className="font-medium">{node.name}</TableCell>
                  <TableCell>
                    <Badge className={`${getNodeStatusColor(node.status)} text-white`}>
                      {node.status}
                    </Badge>
                  </TableCell>
                  <TableCell>{node.roles}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{node.version}</TableCell>
                  <TableCell className="text-sm font-mono">{node.internal_ip}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{node.os_image}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{node.age}</TableCell>
                  <TableCell className="text-right">
                    <ResourceActionMenu
                      actions={[
                        {
                          label: "Cordon",
                          icon: ShieldOff,
                          hidden: isSchedulingDisabled(node),
                          onClick: () => handleCordon(node),
                        },
                        {
                          label: "Uncordon",
                          icon: ShieldCheck,
                          hidden: !isSchedulingDisabled(node),
                          onClick: () => handleUncordon(node),
                        },
                        {
                          label: "Drain",
                          icon: Trash2,
                          variant: "destructive",
                          onClick: () => setActiveModal({ type: "drain", node }),
                        },
                        {
                          label: "Edit",
                          icon: Pencil,
                          onClick: () => openEdit(node),
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

      {activeModal?.type === "drain" && (
        <ConfirmDeleteDialog
          open
          onOpenChange={(o) => { if (!o) setActiveModal(null); }}
          resourceType="Node"
          resourceName={activeModal.node.name}
          isLoading={isActing}
          onConfirm={handleDrain}
          variant="force-delete"
        />
      )}

      {activeModal?.type === "edit" && (
        <EditResourceModal
          isOpen
          clusterId={clusterId}
          namespace=""
          resourceType="nodes"
          resourceName={activeModal.node.name}
          initialYaml={activeModal.yaml}
          onClose={() => { setActiveModal(null); onRefresh?.(); }}
        />
      )}
    </>
  );
}
