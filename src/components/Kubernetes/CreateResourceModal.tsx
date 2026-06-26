import React from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui";
import { Button } from "@/components/ui";
import { Input } from "@/components/ui";
import { Label } from "@/components/ui";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { YamlEditor } from "./YamlEditor";
import { createResourceCmd } from "@/lib/tauriCommands";
import { Loader2 } from "lucide-react";

interface CreateResourceModalProps {
  isOpen: boolean;
  clusterId: string;
  namespace: string;
  onClose?: () => void;
}

const RESOURCE_TYPES = [
  { value: "pod", label: "Pod" },
  { value: "deployment", label: "Deployment" },
  { value: "service", label: "Service" },
  { value: "configmap", label: "ConfigMap" },
  { value: "secret", label: "Secret" },
  { value: "ingress", label: "Ingress" },
  { value: "pvc", label: "PersistentVolumeClaim" },
  { value: "pv", label: "PersistentVolume" },
];

function buildYaml(
  resourceType: string,
  name: string,
  namespace: string
): string {
  const kindMap: Record<string, string> = {
    pod: "Pod",
    deployment: "Deployment",
    service: "Service",
    configmap: "ConfigMap",
    secret: "Secret",
    ingress: "Ingress",
    pvc: "PersistentVolumeClaim",
    pv: "PersistentVolume",
  };
  const apiVersionMap: Record<string, string> = {
    pod: "v1",
    deployment: "apps/v1",
    service: "v1",
    configmap: "v1",
    secret: "v1",
    ingress: "networking.k8s.io/v1",
    pvc: "v1",
    pv: "v1",
  };
  const kind = kindMap[resourceType] ?? resourceType;
  const apiVersion = apiVersionMap[resourceType] ?? "v1";
  const needsNamespace = !["pv"].includes(resourceType);

  return [
    `apiVersion: ${apiVersion}`,
    `kind: ${kind}`,
    "metadata:",
    `  name: ${name || "my-resource"}`,
    ...(needsNamespace ? [`  namespace: ${namespace}`] : []),
    "spec: {}",
  ].join("\n");
}

export function CreateResourceModal({
  isOpen,
  clusterId,
  namespace: initialNamespace,
  onClose,
}: CreateResourceModalProps) {
  const [activeTab, setActiveTab] = React.useState("form");
  const [resourceType, setResourceType] = React.useState("pod");
  const [name, setName] = React.useState("");
  const [namespace, setNamespace] = React.useState(initialNamespace);
  const [yamlContent, setYamlContent] = React.useState("");
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    setNamespace(initialNamespace);
  }, [initialNamespace]);

  const handleSubmit = async () => {
    setIsLoading(true);
    setError(null);
    try {
      if (activeTab === "yaml") {
        await createResourceCmd(clusterId, namespace, resourceType, yamlContent);
      } else {
        const yaml = buildYaml(resourceType, name, namespace);
        await createResourceCmd(clusterId, namespace, resourceType, yaml);
      }
      onClose?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const isFormTabDisabled = activeTab === "form" && !name;

  return (
    <Dialog open={isOpen} onOpenChange={() => onClose?.()}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle>Create Kubernetes Resource</DialogTitle>
        </DialogHeader>

        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList className="grid grid-cols-2 mb-4">
            <TabsTrigger value="form">Form</TabsTrigger>
            <TabsTrigger value="yaml">YAML</TabsTrigger>
          </TabsList>

          <div className="max-h-[60vh] overflow-y-auto">
            <TabsContent value="form" className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="resourceType">Resource Type</Label>
                  <Select value={resourceType} onValueChange={setResourceType}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select type" />
                    </SelectTrigger>
                    <SelectContent>
                      {RESOURCE_TYPES.map((rt) => (
                        <SelectItem key={rt.value} value={rt.value}>
                          {rt.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="name">Name</Label>
                  <Input
                    id="name"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="Enter resource name"
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="namespace">Namespace</Label>
                  <Select value={namespace} onValueChange={setNamespace}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select namespace" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="default">default</SelectItem>
                      <SelectItem value="kube-system">kube-system</SelectItem>
                      <SelectItem value="kube-public">kube-public</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div className="p-4 bg-muted rounded-md">
                <h4 className="text-sm font-medium mb-2">Configuration</h4>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <p>Resource Type: {resourceType}</p>
                  <p>Name: {name || "not specified"}</p>
                  <p>Namespace: {namespace}</p>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="yaml">
              <div className="space-y-4">
                <div className="space-y-2">
                  <Label>Resource YAML</Label>
                  <YamlEditor
                    height="300px"
                    showControls={false}
                    content={yamlContent}
                    onChange={setYamlContent}
                  />
                </div>
              </div>
            </TabsContent>
          </div>

          {error && (
            <p className="text-sm text-destructive mt-2">{error}</p>
          )}

          <DialogFooter>
            <Button variant="outline" onClick={onClose} disabled={isLoading}>
              Cancel
            </Button>
            <Button
              onClick={handleSubmit}
              disabled={isLoading || isFormTabDisabled}
            >
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Creating...
                </>
              ) : (
                "Create Resource"
              )}
            </Button>
          </DialogFooter>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
