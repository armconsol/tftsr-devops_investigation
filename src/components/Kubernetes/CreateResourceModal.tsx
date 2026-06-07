import React from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui";
import { Button } from "@/components/ui";
import { Input } from "@/components/ui";
import { Label } from "@/components/ui";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { YamlEditor } from "./YamlEditor";

interface CreateResourceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (resource: { type: string; name: string; namespace: string }) => void;
}

export function CreateResourceModal({ isOpen, onClose, onSubmit }: CreateResourceModalProps) {
  const [activeTab, setActiveTab] = React.useState("form");
  const [resourceType, setResourceType] = React.useState("pod");
  const [name, setName] = React.useState("");
  const [namespace, setNamespace] = React.useState("default");

  const handleSubmit = () => {
    onSubmit({
      type: resourceType,
      name,
      namespace,
    });
    onClose();
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
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
                      <SelectItem value="pod">Pod</SelectItem>
                      <SelectItem value="deployment">Deployment</SelectItem>
                      <SelectItem value="service">Service</SelectItem>
                      <SelectItem value="configmap">ConfigMap</SelectItem>
                      <SelectItem value="secret">Secret</SelectItem>
                      <SelectItem value="ingress">Ingress</SelectItem>
                      <SelectItem value="pvc">PersistentVolumeClaim</SelectItem>
                      <SelectItem value="pv">PersistentVolume</SelectItem>
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
                  <div className="h-64">
                    <YamlEditor onChange={() => {}} />
                  </div>
                </div>
                <div className="p-4 bg-muted rounded-md">
                  <h4 className="text-sm font-medium mb-2">Preview</h4>
                  <div className="text-sm text-muted-foreground">
                    YAML validation will be performed on submit
                  </div>
                </div>
              </div>
            </TabsContent>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button onClick={handleSubmit} disabled={!name}>
              Create Resource
            </Button>
          </DialogFooter>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
