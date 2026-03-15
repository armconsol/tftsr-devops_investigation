import React from "react";
import type { PiiSpan } from "@/lib/tauriCommands";
import { Badge } from "@/components/ui";

interface PiiDiffViewerProps {
  originalText: string;
  redactedText: string;
  spans: PiiSpan[];
  approvedSpans: PiiSpan[];
  onToggleSpan: (span: PiiSpan, approved: boolean) => void;
}

export function PiiDiffViewer({
  originalText,
  redactedText,
  spans,
  approvedSpans,
  onToggleSpan,
}: PiiDiffViewerProps) {
  const isApproved = (span: PiiSpan) =>
    approvedSpans.some((s) => s.start === span.start && s.end === span.end);

  return (
    <div className="space-y-4">
      {/* Side-by-side diff */}
      <div className="grid grid-cols-2 gap-4">
        <div>
          <h4 className="text-sm font-medium mb-2 text-muted-foreground">Original</h4>
          <div className="rounded-md border bg-card p-3 text-sm font-mono whitespace-pre-wrap max-h-64 overflow-y-auto">
            {highlightSpans(originalText, spans, "original")}
          </div>
        </div>
        <div>
          <h4 className="text-sm font-medium mb-2 text-muted-foreground">Redacted</h4>
          <div className="rounded-md border bg-card p-3 text-sm font-mono whitespace-pre-wrap max-h-64 overflow-y-auto">
            {redactedText || <span className="text-muted-foreground italic">No redactions applied</span>}
          </div>
        </div>
      </div>

      {/* PII span list */}
      {spans.length > 0 && (
        <div>
          <h4 className="text-sm font-medium mb-2">Detected PII ({spans.length} items)</h4>
          <div className="space-y-2 max-h-48 overflow-y-auto">
            {spans.map((span, idx) => (
              <div
                key={`${span.start}-${span.end}-${idx}`}
                className="flex items-center justify-between rounded-md border p-2"
              >
                <div className="flex items-center gap-2">
                  <Badge variant={piiTypeBadgeVariant(span.pii_type)}>
                    {span.pii_type}
                  </Badge>
                  <span className="text-sm font-mono truncate max-w-[200px]">
                    {span.original}
                  </span>
                  <span className="text-muted-foreground text-xs">-&gt;</span>
                  <span className="text-sm font-mono text-muted-foreground truncate max-w-[200px]">
                    {span.replacement}
                  </span>
                </div>
                <label className="flex items-center gap-2 cursor-pointer">
                  <span className="text-xs text-muted-foreground">
                    {isApproved(span) ? "Redact" : "Keep"}
                  </span>
                  <button
                    type="button"
                    role="switch"
                    aria-checked={isApproved(span)}
                    onClick={() => onToggleSpan(span, !isApproved(span))}
                    className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
                      isApproved(span) ? "bg-green-600" : "bg-muted"
                    }`}
                  >
                    <span
                      className={`inline-block h-4 w-4 rounded-full bg-white transition-transform ${
                        isApproved(span) ? "translate-x-4" : "translate-x-0.5"
                      }`}
                    />
                  </button>
                </label>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function highlightSpans(text: string, spans: PiiSpan[], _mode: "original" | "redacted") {
  if (spans.length === 0) return text;

  const sorted = [...spans].sort((a, b) => a.start - b.start);
  const parts: React.ReactNode[] = [];
  let lastEnd = 0;

  sorted.forEach((span, idx) => {
    if (span.start > lastEnd) {
      parts.push(text.slice(lastEnd, span.start));
    }
    parts.push(
      <mark key={idx} className="bg-yellow-200 dark:bg-yellow-800 rounded px-0.5">
        {text.slice(span.start, span.end)}
      </mark>
    );
    lastEnd = span.end;
  });

  if (lastEnd < text.length) {
    parts.push(text.slice(lastEnd));
  }

  return <>{parts}</>;
}

function piiTypeBadgeVariant(piiType: string): "default" | "secondary" | "destructive" | "outline" {
  switch (piiType.toLowerCase()) {
    case "email":
    case "phone":
      return "default";
    case "ip_address":
    case "hostname":
      return "secondary";
    case "ssn":
    case "credit_card":
    case "password":
      return "destructive";
    default:
      return "outline";
  }
}
