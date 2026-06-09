import React, { useCallback, useEffect, useState } from "react";
import { MoreHorizontal, RefreshCw } from "lucide-react";
import {
  Button,
  Badge,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui";
import { helmListReleasesCmd, helmRollbackCmd, helmUninstallCmd } from "@/lib/tauriCommands";
import type { HelmRelease } from "@/lib/tauriCommands";

interface HelmReleaseListProps {
  clusterId: string;
  namespace: string;
}

type ConfirmAction =
  | { type: "rollback"; release: HelmRelease }
  | { type: "uninstall"; release: HelmRelease };

function statusVariant(
  status: string
): "success" | "destructive" | "secondary" | "default" {
  switch (status.toLowerCase()) {
    case "deployed":
      return "success";
    case "failed":
      return "destructive";
    case "pending-install":
    case "pending-upgrade":
    case "pending-rollback":
      return "default";
    case "superseded":
      return "secondary";
    default:
      return "secondary";
  }
}

function statusLabel(status: string): string {
  return status
    .split("-")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

export function HelmReleaseList({ clusterId, namespace }: HelmReleaseListProps) {
  const [releases, setReleases] = useState<HelmRelease[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [openMenuId, setOpenMenuId] = useState<string | null>(null);
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null);
  const [actionInProgress, setActionInProgress] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const loadReleases = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await helmListReleasesCmd(clusterId, namespace);
      setReleases(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [clusterId, namespace]);

  useEffect(() => {
    void loadReleases();
  }, [loadReleases]);

  const handleConfirm = async () => {
    if (!confirmAction) return;
    setActionInProgress(true);
    setActionError(null);
    try {
      const { release } = confirmAction;
      if (confirmAction.type === "rollback") {
        await helmRollbackCmd(clusterId, release.namespace, release.name);
      } else {
        await helmUninstallCmd(clusterId, release.namespace, release.name);
        setReleases((prev) => prev.filter((r) => r.name !== release.name));
      }
      setConfirmAction(null);
      if (confirmAction.type === "rollback") {
        await loadReleases();
      }
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err));
    } finally {
      setActionInProgress(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-32 text-muted-foreground">
        <RefreshCw className="h-5 w-5 animate-spin mr-2" />
        Loading releases…
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <span className="text-sm text-muted-foreground">
          {releases.length} release{releases.length !== 1 ? "s" : ""}
        </span>
        <Button size="sm" variant="outline" onClick={() => void loadReleases()}>
          <RefreshCw className="h-3.5 w-3.5 mr-1" />
          Refresh
        </Button>
      </div>

      {error && (
        <div className="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </div>
      )}

      <div className="border rounded-md overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Namespace</TableHead>
              <TableHead>Chart</TableHead>
              <TableHead>Chart Version</TableHead>
              <TableHead>App Version</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Updated</TableHead>
              <TableHead className="w-12" />
            </TableRow>
          </TableHeader>
          <TableBody>
            {releases.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center text-muted-foreground">
                  No releases found
                </TableCell>
              </TableRow>
            ) : (
              releases.map((release) => {
                const menuKey = `${release.namespace}/${release.name}`;
                return (
                  <TableRow key={menuKey}>
                    <TableCell className="font-medium">{release.name}</TableCell>
                    <TableCell className="text-muted-foreground">{release.namespace}</TableCell>
                    <TableCell className="font-mono text-xs">{release.chart}</TableCell>
                    <TableCell className="font-mono text-xs">{release.chart_version}</TableCell>
                    <TableCell className="font-mono text-xs">{release.app_version || "—"}</TableCell>
                    <TableCell>
                      <Badge variant={statusVariant(release.status)}>
                        {statusLabel(release.status)}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-muted-foreground text-xs">{release.updated}</TableCell>
                    <TableCell>
                      <div className="relative">
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={() =>
                            setOpenMenuId(openMenuId === menuKey ? null : menuKey)
                          }
                          aria-label="Actions"
                        >
                          <MoreHorizontal className="h-4 w-4" />
                        </Button>
                        {openMenuId === menuKey && (
                          <div
                            className="absolute right-0 top-full mt-1 z-50 w-36 rounded-md border bg-card shadow-md"
                            onMouseLeave={() => setOpenMenuId(null)}
                          >
                            <button
                              className="w-full text-left px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground transition-colors"
                              onClick={() => {
                                setOpenMenuId(null);
                                setConfirmAction({ type: "rollback", release });
                              }}
                            >
                              Rollback
                            </button>
                            <button
                              className="w-full text-left px-3 py-2 text-sm text-destructive hover:bg-destructive/10 transition-colors"
                              onClick={() => {
                                setOpenMenuId(null);
                                setConfirmAction({ type: "uninstall", release });
                              }}
                            >
                              Uninstall
                            </button>
                          </div>
                        )}
                      </div>
                    </TableCell>
                  </TableRow>
                );
              })
            )}
          </TableBody>
        </Table>
      </div>

      {/* Confirm dialog */}
      <Dialog open={confirmAction != null} onOpenChange={(o) => { if (!o) setConfirmAction(null); }}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle>
              {confirmAction?.type === "rollback" ? "Rollback Release" : "Uninstall Release"}
            </DialogTitle>
          </DialogHeader>
          <p className="text-sm text-muted-foreground">
            {confirmAction?.type === "rollback" ? (
              <>
                Roll back <span className="font-medium text-foreground">{confirmAction.release.name}</span> to the
                previous revision? This cannot be undone without a re-deploy.
              </>
            ) : (
              <>
                Permanently uninstall <span className="font-medium text-foreground">{confirmAction?.release.name}</span>?
                All Kubernetes resources created by this release will be removed.
              </>
            )}
          </p>
          {actionError && (
            <div className="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive">
              {actionError}
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmAction(null)} disabled={actionInProgress}>
              Cancel
            </Button>
            <Button
              variant={confirmAction?.type === "uninstall" ? "destructive" : "default"}
              onClick={() => void handleConfirm()}
              disabled={actionInProgress}
            >
              {actionInProgress
                ? "Working…"
                : confirmAction?.type === "rollback"
                ? "Rollback"
                : "Uninstall"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
