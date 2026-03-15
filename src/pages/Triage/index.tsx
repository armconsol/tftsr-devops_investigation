import { useEffect, useRef, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { CheckCircle, ChevronRight } from "lucide-react";
import { ChatWindow } from "@/components/ChatWindow";
import { TriageProgress } from "@/components/TriageProgress";
import { useSessionStore } from "@/stores/sessionStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { chatMessageCmd, getIssueCmd } from "@/lib/tauriCommands";
import type { TriageMessage } from "@/lib/tauriCommands";

export default function Triage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const initialized = useRef(false);

  const { currentIssue, messages, currentWhyLevel, startSession, addMessage, setWhyLevel } =
    useSessionStore();
  const { getActiveProvider } = useSettingsStore();

  useEffect(() => {
    if (!id || initialized.current) return;
    initialized.current = true;

    getIssueCmd(id)
      .then((detail) => {
        startSession(detail.issue);

        if (detail.resolution_steps.length === 0) {
          const welcome: TriageMessage = {
            id: "welcome",
            issue_id: id,
            role: "assistant",
            content: `I'll guide you through a 5-Whys root cause analysis for: **"${detail.issue.title}"**\n\nDomain: **${detail.issue.category || "General"}**\n\nDescribe the symptoms you're observing — error messages, affected services, and when the issue started.`,
            why_level: 0,
            created_at: Date.now(),
          };
          addMessage(welcome);
        }
      })
      .catch((e) => setError(String(e)));
  }, [id]);

  const handleSend = async (message: string) => {
    if (!id || !currentIssue) return;
    const provider = getActiveProvider();
    if (!provider) {
      setError("No AI provider configured. Go to Settings > AI Providers.");
      return;
    }

    setIsLoading(true);
    setError(null);

    const userMsg: TriageMessage = {
      id: `user-${Date.now()}`,
      issue_id: id,
      role: "user",
      content: message,
      why_level: currentWhyLevel,
      created_at: Date.now(),
    };
    addMessage(userMsg);

    try {
      const response = await chatMessageCmd(id, message, provider);
      const assistantMsg: TriageMessage = {
        id: `asst-${Date.now()}`,
        issue_id: id,
        role: "assistant",
        content: response.content,
        why_level: currentWhyLevel,
        created_at: Date.now(),
      };
      addMessage(assistantMsg);

      const lower = response.content.toLowerCase();
      if (lower.includes("why 2") || (currentWhyLevel === 1 && lower.includes("why is that"))) setWhyLevel(2);
      else if (lower.includes("why 3")) setWhyLevel(3);
      else if (lower.includes("why 4")) setWhyLevel(4);
      else if (lower.includes("why 5")) setWhyLevel(5);
      if (lower.includes("root cause") && (lower.includes("identified") || lower.includes("the root cause is"))) setWhyLevel(6);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  };

  const canProceed = currentWhyLevel >= 5 || messages.some(
    (m) => m.role === "assistant" && m.content.toLowerCase().includes("root cause")
  );

  return (
    <div className="flex flex-col h-full" data-testid="triage-page">
      <div className="border-b px-6 py-3 flex items-center justify-between">
        <div>
          <h1 className="font-semibold">{currentIssue?.title ?? "Loading..."}</h1>
          <p className="text-sm text-muted-foreground">5-Whys Root Cause Analysis</p>
        </div>
        <button
          onClick={() => navigate(`/issue/${id}/resolution`)}
          disabled={!canProceed}
          className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm disabled:opacity-50 hover:opacity-90"
        >
          <CheckCircle className="w-4 h-4" />
          Resolution Steps
          <ChevronRight className="w-4 h-4" />
        </button>
      </div>

      <div className="px-6 py-3 border-b">
        <TriageProgress currentLevel={Math.min(currentWhyLevel, 5)} />
      </div>

      {error && (
        <div className="mx-6 mt-3 p-3 bg-destructive/10 border border-destructive/20 rounded-md text-sm text-destructive">
          {error}
        </div>
      )}

      <div className="flex-1 overflow-hidden">
        <ChatWindow
          messages={messages}
          onSend={handleSend}
          isLoading={isLoading}
          placeholder="Describe the problem or answer the AI's question..."
        />
      </div>
    </div>
  );
}
