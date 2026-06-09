import React, { useState, useEffect } from "react";
import { Loader2, Save, X } from "lucide-react";
import { Button } from "@/components/ui";
import { YamlEditor } from "@/components/Kubernetes/YamlEditor";
import { createResourceCmd, editResourceCmd } from "@/lib/tauriCommands";
import { BottomPanelTabType } from "@/stores/bottomPanelStore";

export interface YamlEditorTabData {
  /** Type drives the submit behaviour */
  mode:
    | BottomPanelTabType.EDIT_RESOURCE
    | BottomPanelTabType.CREATE_RESOURCE
    | BottomPanelTabType.INSTALL_CHART
    | BottomPanelTabType.UPGRADE_CHART;
  clusterId: string;
  namespace: string;
  resourceType?: string;
  resourceName?: string;
  initialYaml?: string;
  /** For helm flows: the chart name being installed/upgraded */
  chartName?: string;
}

interface YamlEditorTabProps {
  tabId: string;
  data: YamlEditorTabData;
  onClose?: (tabId: string) => void;
}

function actionLabel(mode: YamlEditorTabData["mode"]): string {
  switch (mode) {
    case BottomPanelTabType.EDIT_RESOURCE:
      return "Save";
    case BottomPanelTabType.CREATE_RESOURCE:
      return "Create";
    case BottomPanelTabType.INSTALL_CHART:
      return "Install";
    case BottomPanelTabType.UPGRADE_CHART:
      return "Upgrade";
  }
}

export function YamlEditorTab({ tabId, data, onClose }: YamlEditorTabProps) {
  const [yaml, setYaml] = useState(data.initialYaml ?? "");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    setYaml(data.initialYaml ?? "");
  }, [data.initialYaml]);

  const handleSubmit = async () => {
    setIsSubmitting(true);
    setError(null);
    setSuccess(null);
    try {
      switch (data.mode) {
        case BottomPanelTabType.CREATE_RESOURCE:
          await createResourceCmd(
            data.clusterId,
            data.namespace,
            data.resourceType ?? "",
            yaml
          );
          setSuccess("Resource created");
          break;
        case BottomPanelTabType.EDIT_RESOURCE:
          await editResourceCmd(
            data.clusterId,
            data.namespace,
            data.resourceType ?? "",
            data.resourceName ?? "",
            yaml
          );
          setSuccess("Resource updated");
          break;
        case BottomPanelTabType.INSTALL_CHART:
        case BottomPanelTabType.UPGRADE_CHART:
          // Helm flows are wired up to the existing helm modals; the YAML view
          // here just lets the user prepare values.yaml. Submit is no-op until
          // the corresponding tauri commands are added.
          setSuccess(
            data.mode === BottomPanelTabType.INSTALL_CHART
              ? "Helm install requires the install dialog to complete the flow."
              : "Helm upgrade requires the upgrade dialog to complete the flow."
          );
          break;
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  const label = actionLabel(data.mode);

  return (
    <div className="flex flex-col h-full p-3 gap-2 min-h-0" data-testid="yaml-editor-tab">
      <div className="flex items-center justify-between gap-2">
        <div className="text-xs text-muted-foreground">
          {data.resourceType && <span className="font-mono">{data.resourceType}</span>}
          {data.resourceName && (
            <>
              {" / "}
              <span className="font-mono font-medium">{data.resourceName}</span>
            </>
          )}
          {data.chartName && (
            <span className="font-mono font-medium">{data.chartName}</span>
          )}
          {data.namespace && (
            <span className="ml-2">ns: {data.namespace}</span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => onClose?.(tabId)}
            disabled={isSubmitting}
          >
            <X className="h-3.5 w-3.5 mr-1" />
            Cancel
          </Button>
          <Button size="sm" onClick={() => void handleSubmit()} disabled={isSubmitting}>
            {isSubmitting ? (
              <>
                <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />
                Working...
              </>
            ) : (
              <>
                <Save className="h-3.5 w-3.5 mr-1" />
                {label}
              </>
            )}
          </Button>
        </div>
      </div>

      {error && (
        <div className="rounded-md border border-destructive/50 bg-destructive/10 px-2 py-1 text-xs text-destructive">
          {error}
        </div>
      )}
      {success && (
        <div className="rounded-md border border-green-500/30 bg-green-500/10 px-2 py-1 text-xs text-green-700 dark:text-green-400">
          {success}
        </div>
      )}

      <div className="flex-1 min-h-0">
        <YamlEditor
          height="100%"
          showControls={false}
          content={yaml}
          onChange={setYaml}
        />
      </div>
    </div>
  );
}
