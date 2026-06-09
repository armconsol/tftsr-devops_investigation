import React from "react";
import Editor from "@monaco-editor/react";
import { Button } from "@/components/ui";
import { Loader2 } from "lucide-react";

interface YamlEditorProps {
  content?: string;
  onChange?: (yaml: string) => void;
  onApply?: (yaml: string) => void;
  onCancel?: () => void;
  readOnly?: boolean;
  height?: string;
  showControls?: boolean;
}

export function YamlEditor({
  content = "",
  onChange,
  onApply,
  onCancel,
  readOnly = false,
  height = "400px",
  showControls = true,
}: YamlEditorProps) {
  const [value, setValue] = React.useState(content);
  const [isLoading, setIsLoading] = React.useState(true);
  const [isMonacoReady, setIsMonacoReady] = React.useState(false);

  React.useEffect(() => {
    // Only update value when Monaco is ready to prevent race condition
    if (isMonacoReady) {
      setValue(content);
    }
  }, [content, isMonacoReady]);

  // Initialize value when Monaco mounts
  React.useEffect(() => {
    if (isMonacoReady && content) {
      setValue(content);
    }
  }, [isMonacoReady, content]);

  const handleChange = (v: string | undefined) => {
    const next = v ?? "";
    setValue(next);
    // When there are no controls, propagate every change immediately to the parent.
    if (!showControls) {
      onChange?.(next);
    }
  };

  const handleApply = () => {
    onChange?.(value);
    onApply?.(value);
  };

  return (
    <div className="flex flex-col gap-2 h-full">
      <div
        className="rounded-md border overflow-hidden bg-[#1e1e1e]"
        style={{ height }}
      >
        {isLoading && (
          <div className="flex items-center justify-center h-full bg-[#1e1e1e]">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" role="status" />
          </div>
        )}
        <Editor
          language="yaml"
          theme="vs-dark"
          value={value}
          onChange={handleChange}
          onMount={() => {
            setIsLoading(false);
            setIsMonacoReady(true);
          }}
          options={{
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            fontSize: 13,
            wordWrap: "on",
            readOnly,
          }}
        />
      </div>

      {showControls && (
        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={onCancel}>
            Cancel
          </Button>
          <Button
            className="bg-primary"
            onClick={handleApply}
            disabled={readOnly}
          >
            Apply
          </Button>
        </div>
      )}
    </div>
  );
}
