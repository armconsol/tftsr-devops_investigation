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
import { editResourceCmd } from "@/lib/tauriCommands";
import { Loader2 } from "lucide-react";

interface EditResourceModalProps {
  isOpen: boolean;
  clusterId: string;
  namespace: string;
  resourceType: string;
  resourceName: string;
  initialYaml?: string;
  onClose?: () => void;
}

export function EditResourceModal({
  isOpen,
  clusterId,
  namespace,
  resourceType,
  resourceName,
  initialYaml = "",
  onClose,
}: EditResourceModalProps) {
  const [activeTab, setActiveTab] = React.useState("yaml");
  const [name, setName] = React.useState(resourceName);
  const [currentNamespace, setCurrentNamespace] = React.useState(namespace);
  const [yamlContent, setYamlContent] = React.useState(initialYaml);
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const [yamlReady, setYamlReady] = React.useState(false);

  React.useEffect(() => {
    setName(resourceName);
    setCurrentNamespace(namespace);
    setYamlContent(initialYaml);
    // Mark YAML as ready once we have content
    if (initialYaml) {
      setYamlReady(true);
    }
  }, [resourceName, namespace, initialYaml]);

  const handleSubmit = async () => {
    setIsLoading(true);
    setError(null);
    try {
      await editResourceCmd(
        clusterId,
        currentNamespace,
        resourceType,
        name,
        yamlContent
      );
      onClose?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={() => onClose?.()}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle>Edit Kubernetes Resource</DialogTitle>
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
                  <Select
                    value={currentNamespace}
                    onValueChange={setCurrentNamespace}
                  >
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
                <h4 className="text-sm font-medium mb-2">Resource Details</h4>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <p>Name: {name || "not specified"}</p>
                  <p>Namespace: {currentNamespace}</p>
                  <p>Type: {resourceType}</p>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="yaml">
              <div className="space-y-2">
                <Label>Resource YAML</Label>
                {yamlReady ? (
                  <YamlEditor
                    height="300px"
                    showControls={false}
                    content={yamlContent}
                    onChange={setYamlContent}
                  />
                ) : (
                  <div className="flex items-center justify-center h-[300px] bg-muted rounded-md">
                    <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                  </div>
                )}
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
              disabled={isLoading || !name}
            >
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Saving...
                </>
              ) : (
                "Save Changes"
              )}
            </Button>
          </DialogFooter>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
