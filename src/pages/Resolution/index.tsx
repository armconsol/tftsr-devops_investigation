import React, { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { CheckSquare, Square, FileText, ChevronRight } from "lucide-react";
import { getIssueCmd } from "@/lib/tauriCommands";
import type { ResolutionStep } from "@/lib/tauriCommands";

interface TrackedStep extends ResolutionStep {
  done: boolean;
}

export default function Resolution() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [steps, setSteps] = useState<TrackedStep[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    getIssueCmd(id)
      .then((detail) => {
        setSteps(
          detail.resolution_steps.map((s) => ({ ...s, done: false }))
        );
      })
      .catch((e) => setError(String(e)))
      .finally(() => setIsLoading(false));
  }, [id]);

  const toggleDone = (stepId: string) => {
    setSteps((prev) =>
      prev.map((s) => (s.id === stepId ? { ...s, done: !s.done } : s))
    );
  };

  const doneCount = steps.filter((s) => s.done).length;

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-muted-foreground">Loading resolution steps...</div>
      </div>
    );
  }

  return (
    <div className="max-w-3xl mx-auto p-6" data-testid="resolution-page">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold">Resolution Steps</h1>
          <p className="text-muted-foreground">Review each root cause and confirm resolution.</p>
        </div>
        <button
          onClick={() => navigate(`/issue/${id}/rca`)}
          className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm hover:opacity-90"
        >
          <FileText className="w-4 h-4" />
          Generate RCA
          <ChevronRight className="w-4 h-4" />
        </button>
      </div>

      {error && (
        <div className="p-3 mb-4 bg-destructive/10 border border-destructive/20 rounded-md text-sm text-destructive">
          {error}
        </div>
      )}

      {steps.length > 0 && (
        <div className="mb-6 p-4 bg-card rounded-lg border">
          <div className="flex justify-between text-sm mb-2">
            <span className="font-medium">Progress</span>
            <span className="text-muted-foreground">{doneCount} / {steps.length} resolved</span>
          </div>
          <div className="w-full bg-secondary rounded-full h-2">
            <div
              className="bg-primary h-2 rounded-full transition-all"
              style={{ width: `${(doneCount / steps.length) * 100}%` }}
            />
          </div>
        </div>
      )}

      {steps.length === 0 ? (
        <div className="text-center py-12 text-muted-foreground">
          <p>No resolution steps recorded yet.</p>
          <p className="text-sm mt-1">Complete the triage session to generate steps.</p>
          <button
            onClick={() => navigate(`/issue/${id}/triage`)}
            className="mt-4 px-4 py-2 bg-secondary text-secondary-foreground rounded-md text-sm"
          >
            Go to Triage
          </button>
        </div>
      ) : (
        <div className="space-y-3">
          {steps.map((step) => (
            <div
              key={step.id}
              className={`p-4 rounded-lg border cursor-pointer transition-colors ${
                step.done ? "bg-primary/5 border-primary/20" : "bg-card border-border hover:border-primary/40"
              }`}
              onClick={() => toggleDone(step.id)}
            >
              <div className="flex items-start gap-3">
                <div className="mt-0.5 shrink-0 text-primary">
                  {step.done ? (
                    <CheckSquare className="w-5 h-5" />
                  ) : (
                    <Square className="w-5 h-5 text-muted-foreground" />
                  )}
                </div>
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-xs font-medium px-2 py-0.5 bg-secondary rounded-full">
                      Step {step.step_order}
                    </span>
                    {step.done && <span className="text-xs text-primary font-medium">Resolved</span>}
                  </div>
                  <p className="text-sm font-medium">{step.why_question}</p>
                  {step.answer && <p className="text-sm text-muted-foreground mt-1">{step.answer}</p>}
                  {step.evidence && <p className="text-xs text-muted-foreground mt-1 italic">{step.evidence}</p>}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
