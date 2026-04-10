import React, { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { ChevronRight } from "lucide-react";
import { DocEditor } from "@/components/DocEditor";
import { useSettingsStore } from "@/stores/settingsStore";
import {
  generatePostmortemCmd,
  
  updateDocumentCmd,
  exportDocumentCmd,
  type Document_,
} from "@/lib/tauriCommands";

export default function Postmortem() {
  const { id } = useParams<{ id: string }>();
  void useSettingsStore((s) => s.getActiveProvider);

  const [doc, setDoc] = useState<Document_ | null>(null);
  const [content, setContent] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    const load = async () => {
      setIsLoading(true);
      try {
        const generated = await generatePostmortemCmd(id);
        setDoc(generated);
        setContent(generated.content_md);
      } catch (err) {
        setError(String(err));
      } finally {
        setIsLoading(false);
      }
    };
    load();
  }, [id]);

  const handleContentChange = async (newContent: string) => {
    setContent(newContent);
    if (doc) {
      try {
        const updated = await updateDocumentCmd(doc.id, newContent);
        setDoc(updated);
      } catch (err) {
        setError(String(err));
      }
    }
  };

  const handleExport = async (format: "md" | "pdf" | "docx") => {
    if (!doc) return;
    try {
      const path = await exportDocumentCmd(doc.id, doc.title, content, format, "");
      setError(`Document exported to: ${path}`);
      setTimeout(() => setError(null), 5000);
    } catch (err) {
      setError(`Export failed: ${String(err)}`);
    }
  };

  return (
    <div className="p-6 space-y-4 h-full flex flex-col">
      {/* Breadcrumb */}
      <nav className="flex items-center gap-1 text-sm text-muted-foreground">
        <Link to={`/issue/${id}/triage`} className="hover:text-foreground">
          Triage
        </Link>
        <ChevronRight className="w-3 h-3" />
        <Link to={`/issue/${id}/resolution`} className="hover:text-foreground">
          Resolution
        </Link>
        <ChevronRight className="w-3 h-3" />
        <Link to={`/issue/${id}/rca`} className="hover:text-foreground">
          RCA
        </Link>
        <ChevronRight className="w-3 h-3" />
        <span className="text-foreground font-medium">Post-Mortem</span>
      </nav>

      <div>
        <h1 className="text-3xl font-bold">Post-Mortem Report</h1>
        <p className="text-muted-foreground mt-1">
          Document the incident timeline, impact, and lessons learned.
        </p>
      </div>

      {error && (
        <div className="text-sm text-destructive bg-destructive/10 rounded-md p-3">
          {error}
        </div>
      )}

      {isLoading ? (
        <div className="flex-1 flex items-center justify-center">
          <p className="text-muted-foreground">Generating post-mortem document...</p>
        </div>
      ) : (
        <div className="flex-1 min-h-0">
          <DocEditor
            content={content}
            onChange={handleContentChange}
            
            updatedAt={doc?.updated_at?.toString()}
            onExport={handleExport}
          />
        </div>
      )}
    </div>
  );
}
