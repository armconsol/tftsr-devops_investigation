import React, { useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  Terminal,
  Monitor,
  Network,
  Container,
  Database,
  Server,
  HardDrive,
  BarChart3,
} from "lucide-react";
import {
  Card,
  CardContent,
  Button,
  Input,
  Label,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@/components/ui";
import { DOMAINS } from "@/lib/domainPrompts";
import { createIssueCmd } from "@/lib/tauriCommands";
import { useSessionStore } from "@/stores/sessionStore";

const iconMap: Record<string, React.ElementType> = {
  Terminal,
  Monitor,
  Network,
  Container,
  Database,
  Server,
  HardDrive,
  BarChart3,
};

export default function NewIssue() {
  const navigate = useNavigate();
  const startSession = useSessionStore((s) => s.startSession);
  const [selectedDomain, setSelectedDomain] = useState<string | null>(null);
  const [title, setTitle] = useState("");
  const [severity, setSeverity] = useState("P3");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleStartTriage = async () => {
    if (!selectedDomain || !title.trim()) return;
    setIsSubmitting(true);
    setError(null);
    try {
      const issue = await createIssueCmd({ title: title.trim(), domain: selectedDomain, severity });
      startSession(issue);
      navigate(`/issue/${issue.id}/logs`);
    } catch (err) {
      setError(String(err));
      setIsSubmitting(false);
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold">New Issue</h1>
        <p className="text-muted-foreground mt-1">
          Select a domain, describe the issue, and begin triage.
        </p>
      </div>

      {/* Domain selection grid */}
      <div>
        <Label className="text-sm font-medium mb-3 block">Select Domain</Label>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          {DOMAINS.map((domain) => {
            const Icon = iconMap[domain.icon] ?? Terminal;
            const isSelected = selectedDomain === domain.id;
            return (
              <Card
                key={domain.id}
                className={`cursor-pointer transition-colors hover:border-primary ${
                  isSelected ? "border-primary bg-primary/5 ring-2 ring-primary" : ""
                }`}
                onClick={() => setSelectedDomain(domain.id)}
              >
                <CardContent className="p-4 text-center">
                  <Icon className={`w-8 h-8 mx-auto mb-2 ${isSelected ? "text-primary" : "text-muted-foreground"}`} />
                  <p className="text-sm font-medium">{domain.label}</p>
                  <p className="text-xs text-muted-foreground mt-1">{domain.description}</p>
                </CardContent>
              </Card>
            );
          })}
        </div>
      </div>

      {/* Title */}
      <div className="space-y-2">
        <Label htmlFor="title">Issue Title</Label>
        <Input
          id="title"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="Brief description of the issue..."
        />
      </div>

      {/* Severity */}
      <div className="space-y-2">
        <Label>Severity</Label>
        <Select value={severity} onValueChange={setSeverity}>
          <SelectTrigger>
            <SelectValue placeholder="Select severity" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="P1">P1 - Critical</SelectItem>
            <SelectItem value="P2">P2 - High</SelectItem>
            <SelectItem value="P3">P3 - Medium</SelectItem>
            <SelectItem value="P4">P4 - Low</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Error */}
      {error && (
        <div className="text-sm text-destructive bg-destructive/10 rounded-md p-3">
          {error}
        </div>
      )}

      {/* Submit */}
      <Button
        onClick={handleStartTriage}
        disabled={!selectedDomain || !title.trim() || isSubmitting}
        className="w-full"
        size="lg"
      >
        {isSubmitting ? "Creating..." : "Start Triage"}
      </Button>
    </div>
  );
}
