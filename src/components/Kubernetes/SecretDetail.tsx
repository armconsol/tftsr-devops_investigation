import React from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { Button } from "@/components/ui";
import { X } from "lucide-react";
import { YamlEditor } from "./YamlEditor";
import type { SecretInfo } from "@/lib/tauriCommands";

interface SecretDetailProps {
  clusterId: string;
  namespace: string;
  secret: SecretInfo;
  onClose?: () => void;
}

export function SecretDetail({ namespace: _namespace, secret, onClose }: SecretDetailProps) {
  const [activeTab, setActiveTab] = React.useState("data");

  const keyCount = secret.data_keys;

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <h2 className="text-xl font-semibold">Secret: {secret.name}</h2>
          <Badge variant="destructive">Secret</Badge>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <X className="w-4 h-4" />
        </Button>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid grid-cols-3 mb-4">
          <TabsTrigger value="data">Data</TabsTrigger>
          <TabsTrigger value="metadata">Metadata</TabsTrigger>
          <TabsTrigger value="yaml">YAML</TabsTrigger>
        </TabsList>

        <div className="flex-1 overflow-hidden">
          <TabsContent value="data" className="h-full overflow-y-auto">
            <Card className="h-full flex flex-col">
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle>Secret Data</CardTitle>
                  <span
                    data-testid="secret-key-count"
                    className="text-sm text-muted-foreground"
                  >
                    {keyCount} key{keyCount !== 1 ? "s" : ""}
                  </span>
                </div>
              </CardHeader>
              <CardContent className="flex-1 bg-slate-900 rounded-md p-4 overflow-auto font-mono text-sm">
                {keyCount === 0 ? (
                  <span className="text-muted-foreground">No keys in this secret.</span>
                ) : (
                  <div className="space-y-2">
                    {Array.from({ length: keyCount }, (_, i) => (
                      <div key={i} className="flex items-center gap-2">
                        <span className="text-blue-400">key-{i + 1}:</span>
                        <span className="text-green-400">*****</span>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="metadata" className="h-full overflow-y-auto">
            <div className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle>Metadata</CardTitle>
                </CardHeader>
                <CardContent className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Name</span>
                    <span className="font-mono">{secret.name}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Namespace</span>
                    <span className="font-mono">{secret.namespace}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Type</span>
                    <Badge variant="secondary">{secret.type}</Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Data Keys</span>
                    <span>{secret.data_keys}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Age</span>
                    <span className="text-sm">{secret.age}</span>
                  </div>
                </CardContent>
              </Card>
            </div>
          </TabsContent>

          <TabsContent value="yaml" className="h-full">
            <YamlEditor
              readOnly
              showControls={false}
              content={JSON.stringify(secret, null, 2)}
            />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  );
}
