import React, { useState, useCallback, useEffect } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Upload, File, Trash2, ShieldCheck, AlertTriangle, Image as ImageIcon } from "lucide-react";
import { Button, Card, CardHeader, CardTitle, CardContent, Badge } from "@/components/ui";
import { PiiDiffViewer } from "@/components/PiiDiffViewer";
import { useSessionStore } from "@/stores/sessionStore";
import {
  uploadLogFileCmd,
  detectPiiCmd,
  uploadImageAttachmentCmd,
  uploadPasteImageCmd,
  listImageAttachmentsCmd,
  deleteImageAttachmentCmd,
  type LogFile,
  type PiiSpan,
  type PiiDetectionResult,
  type ImageAttachment,
} from "@/lib/tauriCommands";
import ImageGallery from "@/components/ImageGallery";

export default function LogUpload() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { piiSpans, approvedRedactions, setPiiSpans, setApprovedRedactions } = useSessionStore();

  const [files, setFiles] = useState<{ file: File; uploaded?: LogFile }[]>([]);
  const [images, setImages] = useState<ImageAttachment[]>([]);
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
          void await entry.file.text();
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

  const handleImageDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      const droppedFiles = Array.from(e.dataTransfer.files);
      const imageFiles = droppedFiles.filter((f) => f.type.startsWith("image/"));
      
      if (imageFiles.length > 0) {
        handleImagesUpload(imageFiles);
      }
    },
    [id]
  );

  const handleImageFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      const selected = Array.from(e.target.files).filter((f) => f.type.startsWith("image/"));
      if (selected.length > 0) {
        handleImagesUpload(selected);
      }
    }
  };

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const handlePaste = useCallback(
    async (e: React.ClipboardEvent) => {
      void e.clipboardData?.items;
      const imageItems = Array.from(e.clipboardData?.items || []).filter((item: DataTransferItem) => item.type.startsWith("image/"));
      
      for (const item of imageItems) {
        const file = item.getAsFile();
        if (file) {
          const reader = new FileReader();
          reader.onload = async () => {
            const base64Data = reader.result as string;
            try {
              const result = await uploadPasteImageCmd(id || "", base64Data, file.type);
              setImages((prev) => [...prev, result]);
            } catch (err) {
              setError(String(err));
            }
          };
          reader.readAsDataURL(file);
        }
      }
    },
    [id]
  );

  const handleImagesUpload = async (imageFiles: File[]) => {
    if (!id || imageFiles.length === 0) return;
    
    setIsUploading(true);
    setError(null);
    try {
      const uploaded = await Promise.all(
        imageFiles.map(async (file) => {
          const result = await uploadImageAttachmentCmd(id, file.name);
          return result;
        })
      );
      setImages((prev) => [...prev, ...uploaded]);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsUploading(false);
    }
  };

  const handleDeleteImage = async (image: ImageAttachment) => {
    try {
      await deleteImageAttachmentCmd(image.id);
      setImages((prev) => prev.filter((img) => img.id !== image.id));
    } catch (err) {
      setError(String(err));
    }
  };




  const allUploaded = files.length > 0 && files.every((f) => f.uploaded);
  const piiReviewed = piiResult != null;

  useEffect(() => {
    const handleGlobalPaste = (e: ClipboardEvent) => {
      if (document.activeElement?.tagName === "INPUT" || 
          document.activeElement?.tagName === "TEXTAREA" ||
          (document.activeElement as HTMLElement)?.isContentEditable || false) {
        return;
      }
      
      const items = e.clipboardData?.items;
      const imageItems = items ? Array.from(items).filter((item: DataTransferItem) => item.type.startsWith("image/")) : [];
      
      for (const item of imageItems) {
        const file = item.getAsFile();
        if (file) {
          e.preventDefault();
          const reader = new FileReader();
          reader.onload = async () => {
            const base64Data = reader.result as string;
            try {
              const result = await uploadPasteImageCmd(id || "", base64Data, file.type);
              setImages((prev) => [...prev, result]);
            } catch (err) {
              setError(String(err));
            }
          };
          reader.readAsDataURL(file);
          break;
        }
      }
    };

    window.addEventListener("paste", handleGlobalPaste);
    return () => window.removeEventListener("paste", handleGlobalPaste);
  }, [id]);

  useEffect(() => {
    if (id) {
      listImageAttachmentsCmd(id).then(setImages).catch(setError);
    }
  }, [id]);

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
          accept=".log,.txt,.out,.err,.syslog,.journal,.yaml,.yml,.json,.toml,.xml,.ini,.cfg,.conf,.config,.env,.properties,.md,.markdown,.rst,.csv,.tsv,.ndjson,.jsonl,.sql,.sh,.bash,.zsh,.py,.js,.ts,.rb,.go,.rs,.java,.html,.htm,.css,.diff,.patch,.pdf,.docx,.doc,.rtf,.pcap,.pcapng,.cap"
        />
        <details className="mt-2 text-sm text-gray-500 dark:text-gray-400">
          <summary className="cursor-pointer hover:text-gray-700 dark:hover:text-gray-200">
            Supported formats
          </summary>
          <div className="mt-1 pl-3 space-y-1">
            <div><span className="font-medium">Logs &amp; text:</span> .log, .txt, .out, .err, .syslog, .journal</div>
            <div><span className="font-medium">Config &amp; markup:</span> .yaml, .yml, .json, .toml, .xml, .ini, .cfg, .conf, .env, .properties</div>
            <div><span className="font-medium">Documents:</span> .pdf, .docx, .doc, .md, .rst, .rtf</div>
            <div><span className="font-medium">Data:</span> .csv, .tsv, .ndjson, .jsonl, .sql</div>
            <div><span className="font-medium">Code &amp; scripts:</span> .sh, .bash, .zsh, .py, .js, .ts, .rb, .go, .rs, .java, .html, .css, .diff, .patch</div>
            <div><span className="font-medium">Network captures:</span> .pcap, .pcapng, .cap (requires tshark or tcpdump)</div>
            <p className="mt-1 italic">Binary formats (PDF, DOCX, PCAP) will have their text extracted automatically. XLSX/XLS files are NOT supported - export as CSV instead.</p>
          </div>
        </details>
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

      {/* Image Upload */}
      {id && (
        <>
          <div>
            <h2 className="text-2xl font-semibold flex items-center gap-2">
              <ImageIcon className="w-6 h-6" />
              Image Attachments
            </h2>
            <p className="text-muted-foreground mt-1">
              Upload or paste screenshots and images.
            </p>
          </div>

          {/* Image drop zone */}
          <div
            onDragOver={(e) => e.preventDefault()}
            onDrop={handleImageDrop}
            className="border-2 border-dashed border-primary/30 rounded-lg p-8 text-center hover:border-primary transition-colors cursor-pointer bg-primary/5"
            onClick={() => document.getElementById("image-input")?.click()}
          >
            <Upload className="w-8 h-8 mx-auto text-primary mb-2" />
            <p className="text-sm text-muted-foreground">
              Drag and drop images here, or click to browse
            </p>
            <p className="text-xs text-muted-foreground mt-2">
              Supported: PNG, JPEG, GIF, WebP, SVG
            </p>
            <input
              id="image-input"
              type="file"
              accept="image/*"
              className="hidden"
              onChange={handleImageFileSelect}
            />
          </div>

          {/* Paste button */}
          <div className="flex items-center gap-2">
            <Button
              onClick={async (e) => {
                e.preventDefault();
                document.execCommand("paste");
              }}
              variant="secondary"
            >
              Paste from Clipboard
            </Button>
            <span className="text-xs text-muted-foreground">
              Use Ctrl+V / Cmd+V or the button above to paste images
            </span>
          </div>

          {/* PII warning for images */}
          <div className="bg-amber-50 border border-amber-200 rounded-md p-3">
            <AlertTriangle className="w-5 h-5 text-amber-600 inline mr-2" />
            <span className="text-sm text-amber-800">
              ⚠️ PII cannot be automatically redacted from images. Use at your own risk.
            </span>
          </div>

          {/* Image Gallery */}
          {images.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="text-lg flex items-center gap-2">
                  <ImageIcon className="w-5 h-5" />
                  Attached Images ({images.length})
                </CardTitle>
              </CardHeader>
              <CardContent>
                <ImageGallery
                  images={images}
                  onDelete={handleDeleteImage}
                  showWarning={false}
                />
              </CardContent>
            </Card>
          )}
        </>
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
