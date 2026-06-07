import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Button } from "@/components/ui";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui";
import { Input } from "@/components/ui";
import { Label } from "@/components/ui";
import { Alert, AlertDescription } from "@/components/ui";
import { AlertCircle, RotateCcw, Scale } from "lucide-react";
import type { DeploymentInfo } from "@/lib/tauriCommands";

interface DeploymentListProps {
  deployments: DeploymentInfo[];
  clusterId: string;
  namespace: string;
}

export function DeploymentList({ deployments, clusterId, namespace }: DeploymentListProps) {
  const [scalingDeployment, setScalingDeployment] = useState<DeploymentInfo | null>(null);
  const [replicas, setReplicas] = useState<string>("");
  const [isScaling, setIsScaling] = useState(false);
  const [scaleError, setScaleError] = useState<string | null>(null);

  const [restartingDeployment, setRestartingDeployment] = useState<DeploymentInfo | null>(null);
  const [isRestarting, setIsRestarting] = useState(false);
  const [restartError, setRestartError] = useState<string | null>(null);

  const handleScaleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setReplicas(e.target.value);
    setScaleError(null);
  };

  const handleScaleSubmit = async () => {
    if (!scalingDeployment) return;

    const newReplicas = parseInt(replicas, 10);
    if (isNaN(newReplicas) || newReplicas < 0) {
      setScaleError("Invalid replica count");
      return;
    }

    setIsScaling(true);
    setScaleError(null);

    try {
      await invoke<void>("scale_deployment", {
        clusterId,
        namespace,
        deploymentName: scalingDeployment.name,
        replicas: newReplicas,
      });

      setScalingDeployment(null);
      setReplicas("");
    } catch (err) {
      console.error("Failed to scale deployment:", err);
      setScaleError(err instanceof Error ? err.message : "Failed to scale deployment");
    } finally {
      setIsScaling(false);
    }
  };

  const handleRestartSubmit = async () => {
    if (!restartingDeployment) return;

    setIsRestarting(true);
    setRestartError(null);

    try {
      await invoke<void>("restart_deployment", {
        clusterId,
        namespace,
        deploymentName: restartingDeployment.name,
      });

      setRestartingDeployment(null);
    } catch (err) {
      console.error("Failed to restart deployment:", err);
      setRestartError(err instanceof Error ? err.message : "Failed to restart deployment");
    } finally {
      setIsRestarting(false);
    }
  };

  return (
    <>
      <div className="overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Ready</TableHead>
              <TableHead>Up-to-date</TableHead>
              <TableHead>Available</TableHead>
              <TableHead>Replicas</TableHead>
              <TableHead>Age</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {deployments.length === 0 ? (
              <TableRow>
                <TableCell colSpan={7} className="text-center text-muted-foreground">
                  No deployments found
                </TableCell>
              </TableRow>
            ) : (
              deployments.map((deployment) => (
                <TableRow key={deployment.name}>
                  <TableCell className="font-medium">{deployment.name}</TableCell>
                  <TableCell>{deployment.ready}</TableCell>
                  <TableCell>{deployment.up_to_date}</TableCell>
                  <TableCell>{deployment.available}</TableCell>
                  <TableCell>{deployment.replicas}</TableCell>
                  <TableCell className="text-muted-foreground">{deployment.age}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end gap-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setScalingDeployment(deployment)}
                      >
                        <Scale className="w-4 h-4" />
                        Scale
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setRestartingDeployment(deployment)}
                      >
                        <RotateCcw className="w-4 h-4" />
                        Restart
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      {/* Scale Dialog */}
      <Dialog open={!!scalingDeployment} onOpenChange={() => setScalingDeployment(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Scale Deployment</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="replicas">Replica Count</Label>
              <Input
                id="replicas"
                type="number"
                value={replicas}
                onChange={handleScaleChange}
                placeholder="Enter replica count"
                min="0"
              />
              {scaleError && (
                <Alert variant="destructive" className="mt-2">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{scaleError}</AlertDescription>
                </Alert>
              )}
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setScalingDeployment(null)}>
              Cancel
            </Button>
            <Button onClick={handleScaleSubmit} disabled={isScaling}>
              {isScaling ? "Scaling..." : "Scale"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Restart Dialog */}
      <Dialog open={!!restartingDeployment} onOpenChange={() => setRestartingDeployment(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Restart Deployment</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <p className="text-sm text-muted-foreground">
              This will trigger a rolling restart of the deployment.
            </p>
            {restartError && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{restartError}</AlertDescription>
              </Alert>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setRestartingDeployment(null)}>
              Cancel
            </Button>
            <Button onClick={handleRestartSubmit} disabled={isRestarting}>
              {isRestarting ? "Restarting..." : "Restart"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
