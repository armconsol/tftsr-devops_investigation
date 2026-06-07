import React from "react";
import { Button } from "@/components/ui";
import { Badge } from "@/components/ui";

interface YamlEditorProps {
  onChange: (value: string) => void;
}

export function YamlEditor({ onChange }: YamlEditorProps) {
  return (
    <div className="h-full flex flex-col">
      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <h2 className="text-xl font-semibold">YAML Editor</h2>
          <Badge variant="default" className="bg-green-600">Ready</Badge>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => onChange("")}>
            Clear
          </Button>
          <Button className="bg-primary">
            Apply
          </Button>
        </div>
      </div>

      <div className="flex-1 rounded-md border overflow-hidden flex items-center justify-center">
        <div className="text-center">
          <p className="text-sm text-muted-foreground">YAML Editor would be displayed here</p>
          <p className="text-xs mt-2">Requires @monaco-editor/react dependency</p>
        </div>
      </div>
    </div>
  );
}
