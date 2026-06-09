import React, { useState, useMemo } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Button } from "@/components/ui";
import { Eye, EyeOff, Copy, Check } from "lucide-react";
import * as yaml from "js-yaml";

interface SecretDataModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  secretName: string;
  secretYaml: string;
}

interface SecretData {
  [key: string]: string;
}

export function SecretDataModal({ open, onOpenChange, secretName, secretYaml }: SecretDataModalProps) {
  const [revealedKeys, setRevealedKeys] = useState<Set<string>>(new Set());
  const [copiedKey, setCopiedKey] = useState<string | null>(null);

  const secretData = useMemo<SecretData>(() => {
    try {
      const parsed = yaml.load(secretYaml) as { data?: SecretData };
      return parsed.data ?? {};
    } catch (err) {
      console.error("Failed to parse secret YAML:", err);
      return {};
    }
  }, [secretYaml]);

  const decodedData = useMemo(() => {
    const decoded: Record<string, string> = {};
    Object.entries(secretData).forEach(([key, value]) => {
      try {
        // Decode base64 using native atob
        decoded[key] = atob(value);
      } catch (err) {
        decoded[key] = `[Failed to decode: ${err instanceof Error ? err.message : String(err)}]`;
      }
    });
    return decoded;
  }, [secretData]);

  const toggleReveal = (key: string) => {
    setRevealedKeys((prev) => {
      const next = new Set(prev);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      return next;
    });
  };

  const copyToClipboard = async (key: string, value: string) => {
    try {
      await navigator.clipboard.writeText(value);
      setCopiedKey(key);
      setTimeout(() => setCopiedKey(null), 2000);
    } catch (err) {
      console.error("Failed to copy to clipboard:", err);
    }
  };

  const dataKeys = Object.keys(secretData);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Secret Data: {secretName}</DialogTitle>
          <DialogDescription>
            Decoded secret data. Click the eye icon to reveal values.
          </DialogDescription>
        </DialogHeader>

        {dataKeys.length === 0 ? (
          <p className="text-sm text-muted-foreground py-4">No data keys in this secret.</p>
        ) : (
          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Key</TableHead>
                  <TableHead>Value</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {dataKeys.map((key) => {
                  const isRevealed = revealedKeys.has(key);
                  const value = decodedData[key] ?? "";
                  const isCopied = copiedKey === key;

                  return (
                    <TableRow key={key}>
                      <TableCell className="font-medium font-mono text-sm">{key}</TableCell>
                      <TableCell className="font-mono text-sm max-w-md truncate">
                        {isRevealed ? value : "••••••••"}
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex items-center justify-end gap-2">
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => toggleReveal(key)}
                            title={isRevealed ? "Hide value" : "Reveal value"}
                          >
                            {isRevealed ? (
                              <EyeOff className="w-4 h-4" />
                            ) : (
                              <Eye className="w-4 h-4" />
                            )}
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => copyToClipboard(key, value)}
                            title="Copy to clipboard"
                          >
                            {isCopied ? (
                              <Check className="w-4 h-4 text-green-500" />
                            ) : (
                              <Copy className="w-4 h-4" />
                            )}
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
