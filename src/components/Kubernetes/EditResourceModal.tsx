import React from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui";
import { Button } from "@/components/ui";
import { Input } from "@/components/ui";
import { Label } from "@/components/ui";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { YamlEditor } from "./YamlEditor";

interface EditResourceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (resource: { name: string; namespace: string }) => void;
  initialData?: { name?: string; namespace?: string };
}

export function EditResourceModal({ isOpen, onClose, onSubmit, initialData }: EditResourceModalProps) {
  const [activeTab, setActiveTab] = React.useState("form");
  const [name, setName] = React.useState(initialData?.name || "");
  const [namespace, setNamespace] = React.useState(initialData?.namespace || "default");

  const handleSubmit = () => {
    onSubmit({
      name,
      namespace,
    });
    onClose();
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
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
                <h4 className="text-sm font-medium mb-2">Resource Details</h4>
                <div className="space-y-2 text-sm text-muted-foreground">
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
              Save Changes
            </Button>
          </DialogFooter>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
