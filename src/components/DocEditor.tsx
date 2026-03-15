import React, { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { FileText, Eye, Edit3, Download } from "lucide-react";
import { Button } from "@/components/ui";

interface DocEditorProps {
  content: string;
  onChange: (content: string) => void;
  version?: number;
  updatedAt?: string;
  onExport?: (format: "md" | "pdf" | "docx") => void;
}

export function DocEditor({ content, onChange, version, updatedAt, onExport }: DocEditorProps) {
  const [mode, setMode] = useState<"edit" | "preview">("edit");

  return (
    <div className="flex flex-col h-full border rounded-lg overflow-hidden">
      {/* Toolbar */}
      <div className="flex items-center justify-between border-b px-4 py-2 bg-card">
        <div className="flex items-center gap-2">
          <button
            onClick={() => setMode("edit")}
            className={`flex items-center gap-1 px-2 py-1 rounded text-sm ${
              mode === "edit"
                ? "bg-primary text-primary-foreground"
                : "text-muted-foreground hover:bg-accent"
            }`}
          >
            <Edit3 className="w-3 h-3" />
            Edit
          </button>
          <button
            onClick={() => setMode("preview")}
            className={`flex items-center gap-1 px-2 py-1 rounded text-sm ${
              mode === "preview"
                ? "bg-primary text-primary-foreground"
                : "text-muted-foreground hover:bg-accent"
            }`}
          >
            <Eye className="w-3 h-3" />
            Preview
          </button>
        </div>

        <div className="flex items-center gap-2">
          {version != null && (
            <span className="text-xs text-muted-foreground">
              v{version}
              {updatedAt && ` | ${new Date(updatedAt).toLocaleDateString()}`}
            </span>
          )}
          {onExport && (
            <div className="flex items-center gap-1">
              <Button size="sm" variant="outline" onClick={() => onExport("md")}>
                <FileText className="w-3 h-3 mr-1" />
                MD
              </Button>
              <Button size="sm" variant="outline" onClick={() => onExport("pdf")}>
                <Download className="w-3 h-3 mr-1" />
                PDF
              </Button>
              <Button size="sm" variant="outline" onClick={() => onExport("docx")}>
                <Download className="w-3 h-3 mr-1" />
                DOCX
              </Button>
            </div>
          )}
        </div>
      </div>

      {/* Editor / Preview */}
      <div className="flex-1 overflow-y-auto">
        {mode === "edit" ? (
          <textarea
            value={content}
            onChange={(e) => onChange(e.target.value)}
            className="w-full h-full min-h-[400px] p-4 bg-background text-sm font-mono resize-none focus:outline-none"
            placeholder="Start writing your document..."
          />
        ) : (
          <div className="p-4 prose prose-sm dark:prose-invert max-w-none">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
          </div>
        )}
      </div>
    </div>
  );
}
