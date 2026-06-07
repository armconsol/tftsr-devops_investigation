import React from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui";
import { Badge } from "@/components/ui";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui";
import { Button } from "@/components/ui";
import { X } from "lucide-react";
import { YamlEditor } from "./YamlEditor";

interface SecretDetailProps {
  secretName: string;
  namespace: string;
  _clusterId: string;
  onClose: () => void;
}

export function SecretDetail({ secretName, namespace, _clusterId, onClose }: SecretDetailProps) {
  const [activeTab, setActiveTab] = React.useState("data");
  const [showValues, setShowValues] = React.useState(false);

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <h2 className="text-xl font-semibold">Secret: {secretName}</h2>
          <Badge variant="destructive">Secret</Badge>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <X className="w-4 h-4" />
        </Button>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid grid-cols-3 mb-4">
          <TabsTrigger value="data">Data</TabsTrigger>
          <TabsTrigger value="yaml">YAML</TabsTrigger>
          <TabsTrigger value="metadata">Metadata</TabsTrigger>
        </TabsList>

        <div className="flex-1 overflow-hidden">
          <TabsContent value="data" className="h-full overflow-y-auto">
            <Card className="h-full flex flex-col">
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle>Secret Data</CardTitle>
                  <Button variant="outline" size="sm" onClick={() => setShowValues(!showValues)}>
                    {showValues ? "Hide Values" : "Show Values"}
                  </Button>
                </div>
              </CardHeader>
              <CardContent className="flex-1 bg-slate-900 rounded-md p-4 overflow-auto font-mono text-sm">
                <div className="space-y-2">
                  <div>
                    <span className="text-blue-400">username:</span>
                    <span className="text-green-400 ml-2">
                      {showValues ? "admin" : "****"}
                    </span>
                  </div>
                  <div>
                    <span className="text-blue-400">password:</span>
                    <span className="text-green-400 ml-2">
                      {showValues ? "secret123" : "****"}
                    </span>
                  </div>
                  <div>
                    <span className="text-blue-400">api-key:</span>
                    <span className="text-green-400 ml-2">
                      {showValues ? "sk-abc123xyz" : "****"}
                    </span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="yaml" className="h-full">
            <YamlEditor onChange={() => {}} />
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
                    <span className="font-mono">{secretName}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Namespace</span>
                    <span className="font-mono">{namespace}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Type</span>
                    <Badge variant="secondary">Opaque</Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Created</span>
                    <span className="text-sm">2 hours ago</span>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Labels</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="flex flex-wrap gap-2">
                    <Badge variant="secondary">app=web</Badge>
                    <Badge variant="secondary">tier=frontend</Badge>
                  </div>
                </CardContent>
              </Card>
            </div>
          </TabsContent>
        </div>
      </Tabs>
    </div>
  );
}
