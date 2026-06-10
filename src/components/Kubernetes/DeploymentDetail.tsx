import React from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { Button } from "@/components/ui";
import { Network, X, Loader2 } from "lucide-react";
import { YamlEditor } from "./YamlEditor";
import { PortForwardDialog } from "./PortForwardDialog";
import { scaleDeploymentCmd, restartDeploymentCmd, rollbackDeploymentCmd } from "@/lib/tauriCommands";
import type { DeploymentInfo } from "@/lib/tauriCommands";

interface DeploymentDetailProps {
  clusterId: string;
  namespace: string;
  deployment: DeploymentInfo;
  onClose?: () => void;
}

export function DeploymentDetail({ clusterId, namespace, deployment, onClose }: DeploymentDetailProps) {
  const [activeTab, setActiveTab] = React.useState("overview");
  const [replicaCount, setReplicaCount] = React.useState(deployment.replicas);
  const [portForwardOpen, setPortForwardOpen] = React.useState(false);

  const [scaleLoading, setScaleLoading] = React.useState(false);
  const [scaleError, setScaleError] = React.useState<string | null>(null);
  const [scaleSuccess, setScaleSuccess] = React.useState(false);

  const [restartLoading, setRestartLoading] = React.useState(false);
  const [restartError, setRestartError] = React.useState<string | null>(null);

  const [rollbackLoading, setRollbackLoading] = React.useState(false);
  const [rollbackError, setRollbackError] = React.useState<string | null>(null);

  const handleScale = async () => {
    setScaleLoading(true);
    setScaleError(null);
    setScaleSuccess(false);
    try {
      await scaleDeploymentCmd(clusterId, namespace, deployment.name, replicaCount);
      setScaleSuccess(true);
    } catch (err) {
      setScaleError(err instanceof Error ? err.message : String(err));
    } finally {
      setScaleLoading(false);
    }
  };

  const handleRestart = async () => {
    setRestartLoading(true);
    setRestartError(null);
    try {
      await restartDeploymentCmd(clusterId, namespace, deployment.name);
    } catch (err) {
      setRestartError(err instanceof Error ? err.message : String(err));
    } finally {
      setRestartLoading(false);
    }
  };

  const handleRollback = async () => {
    setRollbackLoading(true);
    setRollbackError(null);
    try {
      await rollbackDeploymentCmd(clusterId, namespace, deployment.name);
    } catch (err) {
      setRollbackError(err instanceof Error ? err.message : String(err));
    } finally {
      setRollbackLoading(false);
    }
  };

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <h2 className="text-xl font-semibold">Deployment: {deployment.name}</h2>
          <Badge variant="outline">{namespace}</Badge>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={() => setPortForwardOpen(true)}>
            <Network className="w-4 h-4 mr-1.5" />
            Port Forward
          </Button>
          <Button variant="ghost" size="sm" onClick={onClose}>
            <X className="w-4 h-4" />
          </Button>
        </div>
      </div>

      <PortForwardDialog
        open={portForwardOpen}
        onOpenChange={setPortForwardOpen}
        clusterId={clusterId}
        namespace={namespace}
        podName={undefined}
      />

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid grid-cols-3 mb-4">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="actions">Actions</TabsTrigger>
          <TabsTrigger value="yaml">YAML</TabsTrigger>
        </TabsList>

        <div className="flex-1 overflow-hidden">
          <TabsContent value="overview" className="h-full overflow-y-auto">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
              <Card>
                <CardHeader>
                  <CardTitle>Deployment Information</CardTitle>
                </CardHeader>
                <CardContent className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Name</span>
                    <span className="font-mono">{deployment.name}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Namespace</span>
                    <span className="font-mono">{namespace}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Ready</span>
                    <span className="font-mono">{deployment.ready}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Replicas</span>
                    <span>{deployment.replicas}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Up-to-date</span>
                    <span>{deployment.up_to_date}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Available</span>
                    <span>{deployment.available}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Age</span>
                    <span className="text-sm">{deployment.age}</span>
                  </div>
                </CardContent>
              </Card>

              {Object.keys(deployment.labels).length > 0 && (
                <Card>
                  <CardHeader>
                    <CardTitle>Labels</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="flex flex-wrap gap-2">
                      {Object.entries(deployment.labels).map(([k, v]) => (
                        <Badge key={k} variant="secondary">
                          {k}={v}
                        </Badge>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              )}
            </div>
          </TabsContent>

          <TabsContent value="actions" className="h-full overflow-y-auto">
            <div className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle>Scale</CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                  <div className="flex items-center gap-3">
                    <label htmlFor="replica-input" className="text-sm text-muted-foreground">
                      Replicas
                    </label>
                    <input
                      id="replica-input"
                      type="number"
                      min={0}
                      value={replicaCount}
                      onChange={(e) => setReplicaCount(Number(e.target.value))}
                      className="w-24 border rounded px-2 py-1 text-sm bg-background"
                    />
                    <Button
                      data-testid="scale-button"
                      size="sm"
                      onClick={() => void handleScale()}
                      disabled={scaleLoading}
                    >
                      {scaleLoading ? (
                        <>
                          <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                          Scaling…
                        </>
                      ) : (
                        "Scale"
                      )}
                    </Button>
                  </div>
                  {scaleLoading && (
                    <div data-testid="scale-loading" className="flex items-center gap-2 text-sm text-muted-foreground">
                      <Loader2 className="w-4 h-4 animate-spin" />
                      Scaling deployment…
                    </div>
                  )}
                  {scaleError && (
                    <div data-testid="scale-error" className="text-sm text-red-500">
                      Scale failed: {scaleError}
                    </div>
                  )}
                  {scaleSuccess && (
                    <div className="text-sm text-green-500">
                      Scaled to {replicaCount} replica{replicaCount !== 1 ? "s" : ""}.
                    </div>
                  )}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Restart</CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                  <p className="text-sm text-muted-foreground">
                    Performs a rolling restart of all pods in this deployment.
                  </p>
                  <Button
                    data-testid="restart-button"
                    variant="outline"
                    size="sm"
                    onClick={() => void handleRestart()}
                    disabled={restartLoading}
                  >
                    {restartLoading ? (
                      <>
                        <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                        Restarting…
                      </>
                    ) : (
                      "Restart Deployment"
                    )}
                  </Button>
                  {restartError && (
                    <div className="text-sm text-red-500">Restart failed: {restartError}</div>
                  )}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Rollback</CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                  <p className="text-sm text-muted-foreground">
                    Roll back to the previous revision of this deployment.
                  </p>
                  <Button
                    data-testid="rollback-button"
                    variant="destructive"
                    size="sm"
                    onClick={() => void handleRollback()}
                    disabled={rollbackLoading}
                  >
                    {rollbackLoading ? (
                      <>
                        <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                        Rolling back…
                      </>
                    ) : (
                      "Rollback Deployment"
                    )}
                  </Button>
                  {rollbackError && (
                    <div className="text-sm text-red-500">Rollback failed: {rollbackError}</div>
                  )}
                </CardContent>
              </Card>
            </div>
          </TabsContent>

          <TabsContent value="yaml" className="h-full">
            <YamlEditor
              readOnly
              showControls={false}
              content={JSON.stringify(deployment, null, 2)}
            />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  );
}
