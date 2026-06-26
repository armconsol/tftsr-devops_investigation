import { create } from "zustand";
import type { Issue, TriageMessage, PiiSpan, ResolutionStep } from "@/lib/tauriCommands";

interface SessionState {
  currentIssue: Issue | null;
  messages: TriageMessage[];
  piiSpans: PiiSpan[];
  approvedRedactions: PiiSpan[];
  currentWhyLevel: number;
  activeDomain: string;
  resolutionSteps: ResolutionStep[];
  isLoading: boolean;
  error: string | null;

  startSession: (issue: Issue) => void;
  addMessage: (message: TriageMessage) => void;
  updateMessageContent: (id: string, content: string) => void;
  setPiiSpans: (spans: PiiSpan[]) => void;
  setApprovedRedactions: (spans: PiiSpan[]) => void;
  setWhyLevel: (level: number) => void;
  setActiveDomain: (domain: string) => void;
  setResolutionSteps: (steps: ResolutionStep[]) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

const initialState = {
  currentIssue: null,
  messages: [],
  piiSpans: [],
  approvedRedactions: [],
  currentWhyLevel: 0,
  activeDomain: "general",
  resolutionSteps: [],
  isLoading: false,
  error: null,
};

export const useSessionStore = create<SessionState>((set) => ({
  ...initialState,
  startSession: (issue) => set({ currentIssue: issue, messages: [], currentWhyLevel: 1, activeDomain: issue.category }),
  addMessage: (message) => set((state) => ({ messages: [...state.messages, message] })),
  updateMessageContent: (id, content) =>
    set((state) => ({
      messages: state.messages.map((m) => (m.id === id ? { ...m, content } : m)),
    })),
  setPiiSpans: (spans) => set({ piiSpans: spans }),
  setApprovedRedactions: (spans) => set({ approvedRedactions: spans }),
  setWhyLevel: (level) => set({ currentWhyLevel: level }),
  setActiveDomain: (domain) => set({ activeDomain: domain }),
  setResolutionSteps: (steps) => set({ resolutionSteps: steps }),
  setLoading: (loading) => set({ isLoading: loading }),
  setError: (error) => set({ error }),
  reset: () => set(initialState),
}));
