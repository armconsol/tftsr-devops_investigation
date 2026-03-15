import React, { useState, useCallback } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Upload, File, Trash2, ShieldCheck } from "lucide-react";
import { Button, Card, CardHeader, CardTitle, CardContent, Badge } from "@/components/ui";
import { PiiDiffViewer } from "@/components/PiiDiffViewer";
import { useSessionStore } from "@/stores/sessionStore";
import {
  uploadLogFileCmd,
  detectPiiCmd,
  type LogFile,
  type PiiSpan,
  type PiiDetectionResult,
} from "@/lib/tauriCommands";

export default function LogUpload() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { piiSpans, approvedRedactions, setPiiSpans, setApprovedRedactions } = useSessionStore();

  const [files, setFiles] = useState<{ file: File; uploaded?: LogFile }[]>([]);
  const [piiResult, setPiiResult] = useState<PiiDetectionResult | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [isDetecting, setIsDetecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      const droppedFiles = Array.from(e.dataTransfer.files);
      setFiles((prev) => [...prev, ...droppedFiles.map((f) => ({ file: f }))]);
    },
    []
  );

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      const selected = Array.from(e.target.files);
      setFiles((prev) => [...prev, ...selected.map((f) => ({ file: f }))]);
    }
  };

  const removeFile = (index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
  };

  const handleUpload = async () => {
    if (!id || files.length === 0) return;
    setIsUploading(true);
    setError(null);
    try {
      const uploaded = await Promise.all(
        files.map(async (entry) => {
          if (entry.uploaded) return entry;
          const content = await entry.file.text();
          const logFile = await uploadLogFileCmd(id, entry.file.name);
          return { ...entry, uploaded: logFile };
        })
      );
      setFiles(uploaded);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsUploading(false);
    }
  };

  const handleDetectPii = async () => {
    const allContent = files
      .map((f) => f.uploaded?.file_name)
      .filter(Boolean)
      .join("\n---\n");
    if (!allContent) return;
    setIsDetecting(true);
    setError(null);
    try {
      const result = files[0]?.uploaded ? await detectPiiCmd(files[0].uploaded.id) : null;
      setPiiResult(result);
      if (result) { setPiiSpans(result.detections);
      setApprovedRedactions(result.detections); };
    } catch (err) {
      setError(String(err));
    } finally {
      setIsDetecting(false);
    }
  };

  const handleToggleSpan = (span: PiiSpan, approved: boolean) => {
    if (approved) {
      setApprovedRedactions([...approvedRedactions, span]);
    } else {
      setApprovedRedactions(
        approvedRedactions.filter(
          (s) => !(s.start === span.start && s.end === span.end)
        )
      );
    }
  };

  const allUploaded = files.length > 0 && files.every((f) => f.uploaded);
  const piiReviewed = piiResult != null;

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Upload Logs</h1>
        <p className="text-muted-foreground mt-1">
          Upload log files for PII detection and redaction before triage.
        </p>
      </div>

      {/* Drop zone */}
      <div
        onDragOver={(e) => e.preventDefault()}
        onDrop={handleDrop}
        className="border-2 border-dashed rounded-lg p-8 text-center hover:border-primary transition-colors cursor-pointer"
        onClick={() => document.getElementById("file-input")?.click()}
      >
        <Upload className="w-8 h-8 mx-auto text-muted-foreground mb-2" />
        <p className="text-sm text-muted-foreground">
          Drag and drop log files here, or click to browse
        </p>
        <input
          id="file-input"
          type="file"
          multiple
          className="hidden"
          onChange={handleFileSelect}
          accept=".log,.txt,.json,.csv,.xml,.yaml,.yml"
        />
      </div>

      {/* File list */}
      {files.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Files ({files.length})</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {files.map((entry, idx) => (
              <div key={idx} className="flex items-center justify-between rounded-md border p-2">
                <div className="flex items-center gap-2">
                  <File className="w-4 h-4 text-muted-foreground" />
                  <span className="text-sm">{entry.file.name}</span>
                  <span className="text-xs text-muted-foreground">
                    ({(entry.file.size / 1024).toFixed(1)} KB)
                  </span>
                  {entry.uploaded && (
                    <Badge variant="outline">Uploaded</Badge>
                  )}
                </div>
                <button
                  onClick={() => removeFile(idx)}
                  className="text-muted-foreground hover:text-destructive"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            ))}
            {!allUploaded && (
              <Button onClick={handleUpload} disabled={isUploading} className="mt-2">
                {isUploading ? "Uploading..." : "Upload Files"}
              </Button>
            )}
          </CardContent>
        </Card>
      )}

      {/* PII Detection */}
      {allUploaded && (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <ShieldCheck className="w-5 h-5" />
              PII Detection
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {!piiResult && (
              <Button onClick={handleDetectPii} disabled={isDetecting}>
                {isDetecting ? "Detecting PII..." : "Detect PII"}
              </Button>
            )}
            {piiResult && (
              <PiiDiffViewer
                originalText="[original log content]"
                redactedText="[redacted log content]"
                spans={piiSpans}
                approvedSpans={approvedRedactions}
                onToggleSpan={handleToggleSpan}
              />
            )}
          </CardContent>
        </Card>
      )}

      {/* Error */}
      {error && (
        <div className="text-sm text-destructive bg-destructive/10 rounded-md p-3">
          {error}
        </div>
      )}

      {/* Continue */}
      <Button
        onClick={() => navigate(`/issue/${id}/triage`)}
        disabled={!piiReviewed}
        className="w-full"
        size="lg"
      >
        Continue to Triage
      </Button>
    </div>
  );
}
