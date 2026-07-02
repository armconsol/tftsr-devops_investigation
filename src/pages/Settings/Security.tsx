import React, { useEffect, useState } from "react";
import { Shield, RefreshCw, Lock } from "lucide-react";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  Badge,
} from "@/components/ui";
import {
  getAuditLogCmd,
  getSudoConfigStatusCmd,
  setSudoPasswordCmd,
  testSudoPasswordCmd,
  clearSudoPasswordCmd,
  type AuditEntry,
  type SudoConfigStatus,
} from "@/lib/tauriCommands";
import { useSettingsStore } from "@/stores/settingsStore";

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
  const {
    pii_enabled_patterns,
    setPiiPattern,
    debug_logging_enabled,
    setDebugLoggingEnabled,
  } = useSettingsStore();
  const [auditEntries, setAuditEntries] = useState<AuditEntry[]>([]);
  const [expandedRows, setExpandedRows] = useState<Set<string>>(new Set());
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [sudoPassword, setSudoPassword] = useState("");
  const [sudoUsername, setSudoUsername] = useState("");
  const [sudoStatus, setSudoStatus] = useState<SudoConfigStatus | null>(null);
  const [sudoMessage, setSudoMessage] = useState("");
  const [sudoTesting, setSudoTesting] = useState(false);

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

  const loadSudoStatus = async () => {
    try {
      const status = await getSudoConfigStatusCmd();
      setSudoStatus(status);
    } catch {
      // ignore — table may not exist yet
    }
  };

  useEffect(() => {
    loadAuditLog();
    loadSudoStatus();
  }, []);

  const handleSaveSudo = async () => {
    setSudoMessage("");
    try {
      await setSudoPasswordCmd(sudoPassword, sudoUsername || undefined);
      setSudoPassword("");
      setSudoMessage("Saved successfully");
      await loadSudoStatus();
    } catch (err) {
      setSudoMessage(`Error: ${String(err)}`);
    }
  };

  const handleTestSudo = async () => {
    setSudoTesting(true);
    setSudoMessage("");
    try {
      const ok = await testSudoPasswordCmd();
      setSudoMessage(ok ? "Password verified" : "Authentication failed");
    } catch (err) {
      setSudoMessage(`Authentication failed: ${String(err)}`);
    } finally {
      setSudoTesting(false);
    }
  };

  const handleClearSudo = async () => {
    setSudoMessage("");
    try {
      await clearSudoPasswordCmd();
      setSudoMessage("Credentials cleared");
      await loadSudoStatus();
    } catch (err) {
      setSudoMessage(`Error: ${String(err)}`);
    }
  };

  const toggleRow = (entryId: string) => {
    setExpandedRows((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(entryId)) {
        newSet.delete(entryId);
      } else {
        newSet.add(entryId);
      }
      return newSet;
    });
  };

  const toggleDebugLogging = () => {
    setDebugLoggingEnabled(!debug_logging_enabled);
  };

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Security</h1>
        <p className="text-muted-foreground mt-1">
          Configure PII detection patterns and review the audit log.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Shield className="w-5 h-5" />
            Backend Logging
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between py-1">
            <div>
              <p className="text-sm font-medium">Enable debug logging</p>
              <p className="text-xs text-muted-foreground">
                Off by default (normal logging). Enable for detailed backend diagnostics.
              </p>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={debug_logging_enabled}
              onClick={toggleDebugLogging}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                debug_logging_enabled ? "bg-blue-500" : "bg-muted"
              }`}
            >
              <span
                className={`inline-block h-5 w-5 rounded-full bg-background transition-transform ${
                  debug_logging_enabled ? "translate-x-5" : "translate-x-0.5"
                }`}
              />
            </button>
          </div>
        </CardContent>
      </Card>

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
                aria-checked={pii_enabled_patterns[pattern.id]}
                onClick={() => setPiiPattern(pattern.id, !pii_enabled_patterns[pattern.id])}
                className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                  pii_enabled_patterns[pattern.id] ? "bg-blue-500" : "bg-muted"
                }`}
              >
                <span
                  className={`inline-block h-5 w-5 rounded-full bg-background transition-transform ${
                    pii_enabled_patterns[pattern.id] ? "translate-x-5" : "translate-x-0.5"
                  }`}
                />
              </button>
            </div>
          ))}
        </CardContent>
      </Card>

      {/* Sudo Credentials */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Lock className="w-5 h-5" />
            Sudo Credentials
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {sudoStatus?.configured && (
            <div className="text-sm text-green-600 space-y-0.5">
              <p>Configured for <strong>{sudoStatus.username}</strong></p>
              <p className="text-xs text-muted-foreground">Last updated: {sudoStatus.updated_at}</p>
            </div>
          )}
          {sudoStatus && !sudoStatus.configured && (
            <p className="text-sm text-muted-foreground">Not configured</p>
          )}
          <div className="space-y-3">
            <div>
              <label className="text-sm font-medium" htmlFor="sudo-username">
                Username
              </label>
              <input
                id="sudo-username"
                type="text"
                value={sudoUsername}
                onChange={(e) => setSudoUsername(e.target.value)}
                placeholder="Defaults to current OS user"
                className="mt-1 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              />
              <p className="text-xs text-muted-foreground mt-1">
                Credentials are scoped to this user. Leave blank to use the current OS user.
              </p>
            </div>
            <div>
              <label className="text-sm font-medium" htmlFor="sudo-password">
                Password
              </label>
              <input
                id="sudo-password"
                type="password"
                value={sudoPassword}
                onChange={(e) => setSudoPassword(e.target.value)}
                placeholder="Enter sudo password"
                className="mt-1 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              />
            </div>
          </div>
          <div className="flex gap-2">
            <button
              onClick={handleSaveSudo}
              disabled={!sudoPassword}
              className="px-3 py-1.5 text-sm rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              Save
            </button>
            <button
              onClick={handleTestSudo}
              disabled={sudoTesting || !sudoStatus?.configured}
              className="px-3 py-1.5 text-sm rounded-md border border-input hover:bg-accent disabled:opacity-50"
            >
              {sudoTesting ? "Testing..." : "Test"}
            </button>
            <button
              onClick={handleClearSudo}
              disabled={!sudoStatus?.configured}
              className="px-3 py-1.5 text-sm rounded-md border border-destructive text-destructive hover:bg-destructive/10 disabled:opacity-50"
            >
              Clear
            </button>
          </div>
          {sudoMessage && (
            <p className={`text-sm ${sudoMessage.startsWith("Error") || sudoMessage.includes("failed") ? "text-destructive" : "text-green-600"}`}>
              {sudoMessage}
            </p>
          )}
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
                    <th className="text-center text-xs font-medium text-muted-foreground px-3 py-2">
                      Details
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {auditEntries.map((entry) => {
                    const isExpanded = expandedRows.has(entry.id);
                    return (
                      <React.Fragment key={entry.id}>
                        <tr className="border-b hover:bg-accent/50">
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
                          <td className="px-3 py-2 text-center">
                            <button
                              onClick={() => toggleRow(entry.id)}
                              className="text-xs text-primary hover:underline"
                            >
                              {isExpanded ? "Hide" : "View"}
                            </button>
                          </td>
                        </tr>
                        {isExpanded && (
                          <tr className="border-b bg-accent/20">
                            <td colSpan={5} className="px-3 py-3">
                              <div className="text-xs space-y-2">
                                <p className="font-medium text-foreground">Transmitted Data:</p>
                                <pre className="bg-background/50 p-3 rounded text-xs overflow-x-auto text-foreground/80 whitespace-pre-wrap">
                                  {JSON.stringify(JSON.parse(entry.details), null, 2)}
                                </pre>
                                <div className="flex items-center gap-2 text-muted-foreground pt-1">
                                  <span>Entry ID: {entry.id}</span>
                                  <span>•</span>
                                  <span>Type: {entry.entity_type}</span>
                                </div>
                              </div>
                            </td>
                          </tr>
                        )}
                      </React.Fragment>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
