import React, { useEffect, useState } from "react";
import { Shield, RefreshCw } from "lucide-react";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  Badge,
  Separator,
} from "@/components/ui";
import { getAuditLogCmd, type AuditEntry } from "@/lib/tauriCommands";

const piiPatterns = [
  { id: "email", label: "Email Addresses", description: "Detect email addresses in logs" },
  { id: "ip_address", label: "IP Addresses", description: "Detect IPv4 and IPv6 addresses" },
  { id: "phone", label: "Phone Numbers", description: "Detect phone numbers in various formats" },
  { id: "ssn", label: "Social Security Numbers", description: "Detect US SSN patterns" },
  { id: "credit_card", label: "Credit Card Numbers", description: "Detect credit card number patterns" },
  { id: "hostname", label: "Hostnames", description: "Detect internal hostnames and FQDNs" },
  { id: "password", label: "Passwords in Logs", description: "Detect password= and secret= patterns" },
  { id: "api_key", label: "API Keys", description: "Detect common API key patterns" },
];

export default function Security() {
  const [enabledPatterns, setEnabledPatterns] = useState<Record<string, boolean>>(() =>
    Object.fromEntries(piiPatterns.map((p) => [p.id, true]))
  );
  const [auditEntries, setAuditEntries] = useState<AuditEntry[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadAuditLog();
  }, []);

  const loadAuditLog = async () => {
    setIsLoading(true);
    try {
      const entries = await getAuditLogCmd({ limit: 50 });
      setAuditEntries(entries);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const togglePattern = (id: string) => {
    setEnabledPatterns((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Security</h1>
        <p className="text-muted-foreground mt-1">
          Configure PII detection patterns and review the audit log.
        </p>
      </div>

      {/* PII Patterns */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Shield className="w-5 h-5" />
            PII Detection Patterns
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          {piiPatterns.map((pattern) => (
            <div
              key={pattern.id}
              className="flex items-center justify-between py-2"
            >
              <div>
                <p className="text-sm font-medium">{pattern.label}</p>
                <p className="text-xs text-muted-foreground">{pattern.description}</p>
              </div>
              <button
                type="button"
                role="switch"
                aria-checked={enabledPatterns[pattern.id]}
                onClick={() => togglePattern(pattern.id)}
                className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                  enabledPatterns[pattern.id] ? "bg-blue-500" : "bg-muted"
                }`}
              >
                <span
                  className={`inline-block h-5 w-5 rounded-full bg-white transition-transform ${
                    enabledPatterns[pattern.id] ? "translate-x-5" : "translate-x-0.5"
                  }`}
                />
              </button>
            </div>
          ))}
        </CardContent>
      </Card>

      {/* Audit Log */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg">Audit Log</CardTitle>
            <button
              onClick={loadAuditLog}
              disabled={isLoading}
              className="text-muted-foreground hover:text-foreground"
            >
              <RefreshCw className={`w-4 h-4 ${isLoading ? "animate-spin" : ""}`} />
            </button>
          </div>
        </CardHeader>
        <CardContent>
          {error && (
            <div className="text-sm text-destructive mb-3">{error}</div>
          )}
          {auditEntries.length === 0 ? (
            <p className="text-sm text-muted-foreground">No audit entries yet.</p>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b">
                    <th className="text-left text-xs font-medium text-muted-foreground px-3 py-2">
                      Event Type
                    </th>
                    <th className="text-left text-xs font-medium text-muted-foreground px-3 py-2">
                      Destination
                    </th>
                    <th className="text-left text-xs font-medium text-muted-foreground px-3 py-2">
                      Status
                    </th>
                    <th className="text-left text-xs font-medium text-muted-foreground px-3 py-2">
                      Date
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {auditEntries.map((entry) => (
                    <tr key={entry.id} className="border-b last:border-0">
                      <td className="px-3 py-2 text-sm">
                        <Badge variant="outline">{entry.action}</Badge>
                      </td>
                      <td className="px-3 py-2 text-sm text-muted-foreground">
                        {entry.entity_id}
                      </td>
                      <td className="px-3 py-2">
                        <Badge
                          variant={
                            entry.details.includes("success")
                              ? "default"
                              : entry.action === "blocked"
                                ? "destructive"
                                : "secondary"
                          }
                        >
                          {entry.action}
                        </Badge>
                      </td>
                      <td className="px-3 py-2 text-xs text-muted-foreground">
                        {new Date(entry.timestamp).toLocaleString()}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
