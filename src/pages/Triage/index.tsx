import { useEffect, useRef, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { CheckCircle, ChevronRight } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { ChatWindow } from "@/components/ChatWindow";
import { TriageProgress } from "@/components/TriageProgress";
import { useSessionStore } from "@/stores/sessionStore";
import { useSettingsStore } from "@/stores/settingsStore";
import {
  chatMessageCmd,
  detectPiiCmd,
  getIssueCmd,
  getIssueMessagesCmd,
  uploadLogFileCmd,
  updateIssueCmd,
  addFiveWhyCmd,
} from "@/lib/tauriCommands";
import { getDomainPrompt, detectDomain } from "@/lib/domainPrompts";
import type { TriageMessage } from "@/lib/tauriCommands";

const CLOSE_PATTERNS = [
  "close this issue",
  "please close",
  "mark as resolved",
  "mark resolved",
  "issue is fixed",
  "issue is resolved",
  "resolve this",
  "this is resolved",
];

function isCloseIntent(message: string): boolean {
  const lower = message.toLowerCase();
  return CLOSE_PATTERNS.some((p) => lower.includes(p));
}

type PendingFile = { name: string; logFileId: string };

export default function Triage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [pendingFiles, setPendingFiles] = useState<PendingFile[]>([]);
  // Track the last user message so we can save it as a resolution step when why level advances
  const lastUserMsgRef = useRef<string>("");
  const initialized = useRef(false);

  const { currentIssue, messages, currentWhyLevel, activeDomain, startSession, addMessage, updateMessageContent, setWhyLevel, setActiveDomain } =
    useSessionStore();
  const { getActiveProvider } = useSettingsStore();

  useEffect(() => {
    if (!id || initialized.current) return;
    initialized.current = true;

    Promise.all([getIssueCmd(id), getIssueMessagesCmd(id)])
      .then(([detail, pastMessages]) => {
        startSession(detail.issue);
      setActiveDomain(detail.issue.category);

        if (pastMessages.length > 0) {
          // Restore conversation history from DB
          pastMessages.forEach((m, i) => {
            addMessage({
              id: `hist-${i}`,
              issue_id: id,
              role: m.role,
              content: m.content,
              why_level: 0,
              created_at: Date.now() - (pastMessages.length - i) * 1000,
            });
          });
        } else if (detail.resolution_steps.length === 0) {
          // Fresh issue — show welcome prompt
          addMessage({
            id: "welcome",
            issue_id: id,
            role: "assistant",
            content: `I'll guide you through a 5-Whys root cause analysis for: **"${detail.issue.title}"**\n\nDomain: **${detail.issue.category || "General"}**\n\nDescribe the symptoms you're observing — error messages, affected services, and when the issue started.`,
            why_level: 0,
            created_at: Date.now(),
          });
        }
      })
      .catch((e) => setError(String(e)));
  }, [id]);

  const handleAttach = async () => {
    if (!id) return;
    try {
      const selected = await open({
        multiple: true,
        filters: [
          { name: "Log & Text Files", extensions: ["log", "txt", "json", "xml", "yaml", "yml", "csv", "out", "err"] },
          { name: "Images", extensions: ["png", "jpg", "jpeg", "gif", "bmp", "webp"] },
          { name: "All Files", extensions: ["*"] },
        ],
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      for (const filePath of paths) {
        const logFile = await uploadLogFileCmd(id, filePath);
        setPendingFiles((prev) => [...prev, { name: logFile.file_name, logFileId: logFile.id }]);
      }
    } catch (e) {
      setError(`Attachment failed: ${String(e)}`);
    }
  };

  const handleSend = async (message: string) => {
    if (!id) return;

    // Close intent: works regardless of whether issue is fully loaded in session.
    // Save the user's reason as a resolution step so the Resolution page is never empty.
    if (isCloseIntent(message) && pendingFiles.length === 0) {
      try {
        await addFiveWhyCmd(id, 1, "Resolution", message, "Self-resolved by user");
        await updateIssueCmd(id, { status: "resolved" });
        navigate("/");
      } catch (e) {
        setError(String(e));
      }
      return;
    }

    if (!currentIssue) return;

    const provider = getActiveProvider();
    if (!provider) {
      setError("No AI provider configured. Go to Settings > AI Providers.");
      return;
    }

    setIsLoading(true);
    setError(null);
    setNotice(null);

    // Pre-send attachment PII scan: surface a notice to the user about what will be
    // auto-redacted. The send is NOT blocked — the backend performs the actual redaction.
    const piiNotices: string[] = [];
    for (const f of pendingFiles) {
      try {
        const result = await detectPiiCmd(f.logFileId);
        if (result.total_pii_found > 0) {
          const types = [...new Set(result.detections.map((d) => d.pii_type))].join(", ");
          piiNotices.push(`"${f.name}" (${types})`);
        }
      } catch {
        // Non-fatal — backend will still scan before sending to AI
      }
    }
    if (piiNotices.length > 0) {
      setNotice(`PII auto-redacted before sending: ${piiNotices.join("; ")}`);
    }

    const fileNames = pendingFiles.map((f) => f.name);
    const displayContent =
      pendingFiles.length > 0
        ? `${message}${message ? "\n" : ""}📎 ${fileNames.join(", ")}`
        : message;

    const userMsg: TriageMessage = {
      id: `user-${Date.now()}`,
      issue_id: id,
      role: "user",
      content: displayContent,
      why_level: currentWhyLevel,
      created_at: Date.now(),
    };
    lastUserMsgRef.current = message;
    addMessage(userMsg);
    const logFileIds = pendingFiles.map((f) => f.logFileId);
    setPendingFiles([]);

    try {
      // Detect domain from conversation messages
      const messageContents = messages.map((m) => m.content);
      const detectedDomain = detectDomain(messageContents);

      // Update active domain if it has changed
      if (detectedDomain !== activeDomain && detectedDomain !== "general") {
        setActiveDomain(detectedDomain);
      }

      // Use the active domain for the system prompt
      const systemPrompt = activeDomain ? getDomainPrompt(activeDomain) : undefined;
      // Backend auto-redacts PII in both message text and attachments before sending to AI.
      const response = await chatMessageCmd(id, message, logFileIds, provider, systemPrompt);

      // Update the user bubble with what was actually stored (may be auto-redacted).
      if (response.user_message) {
        const suffix = fileNames.length > 0 ? `\n📎 ${fileNames.join(", ")}` : "";
        updateMessageContent(userMsg.id, response.user_message + suffix);
      }

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
      let nextLevel = currentWhyLevel;
      if (lower.includes("why 2") || (currentWhyLevel === 1 && lower.includes("why is that"))) nextLevel = 2;
      else if (lower.includes("why 3")) nextLevel = 3;
      else if (lower.includes("why 4")) nextLevel = 4;
      else if (lower.includes("why 5")) nextLevel = 5;
      if (lower.includes("root cause") && (lower.includes("identified") || lower.includes("the root cause is"))) nextLevel = 6;

      // Auto-save the completed why step as a resolution step
      if (nextLevel > currentWhyLevel && currentWhyLevel >= 1 && currentWhyLevel <= 5) {
        addFiveWhyCmd(
          id,
          currentWhyLevel,
          `Why ${currentWhyLevel}: ${lastUserMsgRef.current}`,
          response.content.slice(0, 500),
          ""
        ).catch(() => {}); // non-blocking, best-effort
      }
      if (nextLevel !== currentWhyLevel) setWhyLevel(nextLevel);
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

      {notice && (
        <div className="mx-6 mt-3 p-3 bg-amber-50 border border-amber-200 rounded-md text-sm text-amber-700">
          ℹ️ {notice}
        </div>
      )}
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
          pendingFiles={pendingFiles}
          onAttach={handleAttach}
          onRemoveFile={(i) => setPendingFiles((prev) => prev.filter((_, idx) => idx !== i))}
        />
      </div>
    </div>
  );
}
