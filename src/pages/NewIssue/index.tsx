import React, { useState, useEffect } from "react";
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
  Phone,
  Lock,
  PhoneCall,
  Code,
  Workflow,
  CircuitBoard,
  ServerCog,
  Users,
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
  Phone,
  Lock,
  PhoneCall,
  Code,
  Workflow,
  CircuitBoard,
  ServerCog,
  Users,
};

export default function NewIssue() {
  const navigate = useNavigate();
  const startSession = useSessionStore((s) => s.startSession);
  const [selectedDomain, setSelectedDomain] = useState<string | null>(null);
  const [title, setTitle] = useState("");
  const [severity, setSeverity] = useState("P3");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showDisclaimer, setShowDisclaimer] = useState(false);

  useEffect(() => {
    const hasAcceptedDisclaimer = localStorage.getItem("tftsr-ai-disclaimer-accepted");
    if (!hasAcceptedDisclaimer) {
      setShowDisclaimer(true);
    }
  }, []);

  const handleAcceptDisclaimer = () => {
    localStorage.setItem("tftsr-ai-disclaimer-accepted", "true");
    setShowDisclaimer(false);
  };

  const handleStartTriage = async () => {
    const hasAcceptedDisclaimer = localStorage.getItem("tftsr-ai-disclaimer-accepted");
    if (!hasAcceptedDisclaimer) {
      setShowDisclaimer(true);
      return;
    }

    if (!selectedDomain || !title.trim()) return;
    setIsSubmitting(true);
    setError(null);
    try {
      const issue = await createIssueCmd({ title: title.trim(), domain: selectedDomain, severity });
      startSession(issue);
      navigate(`/issue/${issue.id}/triage`);
    } catch (err) {
      setError(String(err));
      setIsSubmitting(false);
    }
  };

  return (
    <>
      {/* AI Disclaimer Modal */}
      {showDisclaimer && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-background border rounded-lg max-w-2xl w-full p-6 space-y-4 shadow-lg">
            <h2 className="text-2xl font-bold text-foreground">AI-Assisted Triage Disclaimer</h2>

            <div className="space-y-3 text-sm text-foreground/90 max-h-[60vh] overflow-y-auto">
              <p className="font-medium text-destructive">
                ⚠️ IMPORTANT: Read Carefully Before Proceeding
              </p>

              <p>
                This application uses artificial intelligence (AI) to assist with IT issue triage, root cause analysis,
                and troubleshooting recommendations. While AI can be a powerful tool, <strong>you must understand its
                limitations and your responsibilities</strong>.
              </p>

              <div className="bg-destructive/10 border border-destructive/20 rounded p-3 space-y-2">
                <p className="font-semibold">AI Can Make Mistakes:</p>
                <ul className="list-disc list-inside space-y-1 ml-2">
                  <li>AI models may provide <strong>incorrect</strong>, <strong>incomplete</strong>, or <strong>outdated</strong> information</li>
                  <li>AI can <strong>hallucinate</strong> — generating plausible-sounding but entirely false information</li>
                  <li>Recommendations may not apply to your specific environment or configuration</li>
                  <li>Commands suggested by AI may have <strong>unintended consequences</strong> including data loss, system downtime, or security vulnerabilities</li>
                </ul>
              </div>

              <div className="bg-yellow-500/10 border border-yellow-500/20 rounded p-3 space-y-2">
                <p className="font-semibold">You Are Fully Responsible:</p>
                <ul className="list-disc list-inside space-y-1 ml-2">
                  <li><strong>You</strong> are solely responsible for any commands you execute, changes you make, or actions you take based on AI recommendations</li>
                  <li><strong>Always verify</strong> AI suggestions against official documentation, best practices, and your organization's policies</li>
                  <li><strong>Test in non-production</strong> environments before applying changes to production systems</li>
                  <li><strong>Understand commands</strong> before executing them — never blindly run suggested commands</li>
                  <li>Have <strong>backups and rollback plans</strong> in place before making system changes</li>
                </ul>
              </div>

              <p>
                <strong>Best Practices:</strong>
              </p>
              <ul className="list-disc list-inside space-y-1 ml-2">
                <li>Treat AI recommendations as a starting point for investigation, not definitive answers</li>
                <li>Consult with senior engineers or subject matter experts when dealing with critical systems</li>
                <li>Review all AI-generated content for accuracy and relevance to your specific situation</li>
                <li>Maintain proper change control and approval processes</li>
                <li>Document your actions and decision-making process</li>
              </ul>

              <p className="pt-2 border-t">
                By clicking "I Understand and Accept," you acknowledge that you have read and understood this disclaimer,
                and you accept full responsibility for any actions taken based on information provided by this AI-assisted
                system. <strong>Use at your own risk.</strong>
              </p>
            </div>

            <div className="flex gap-3 pt-2">
              <Button
                onClick={handleAcceptDisclaimer}
                className="flex-1"
              >
                I Understand and Accept
              </Button>
              <Button
                onClick={() => navigate("/")}
                variant="secondary"
                className="flex-1"
              >
                Cancel
              </Button>
            </div>
          </div>
        </div>
      )}

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
    </>
  );
}
