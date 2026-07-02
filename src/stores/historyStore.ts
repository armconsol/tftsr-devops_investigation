// Copyright (c) 2025 Shaun Arman
// MIT License - see LICENSE file for details

import { create } from "zustand";
import type { IssueSummary } from "@/lib/tauriCommands";
import { listIssuesCmd, searchIssuesCmd } from "@/lib/tauriCommands";

interface HistoryState {
  issues: IssueSummary[];
  isLoading: boolean;
  error: string | null;
  searchQuery: string;

  loadIssues: (filter?: { status?: string; domain?: string }) => Promise<void>;
  searchIssues: (query: string) => Promise<void>;
  setSearchQuery: (query: string) => void;
}

export const useHistoryStore = create<HistoryState>((set) => ({
  issues: [],
  isLoading: false,
  error: null,
  searchQuery: "",

  loadIssues: async (filter = {}) => {
    set({ isLoading: true, error: null });
    try {
      const issues = await listIssuesCmd({ ...filter, limit: 50 });
      set({ issues, isLoading: false });
    } catch (err) {
      set({ error: String(err), isLoading: false });
    }
  },

  searchIssues: async (query: string) => {
    set({ isLoading: true, searchQuery: query });
    try {
      const issues = await searchIssuesCmd(query);
      set({ issues, isLoading: false });
    } catch (err) {
      set({ error: String(err), isLoading: false });
    }
  },

  setSearchQuery: (query) => set({ searchQuery: query }),
}));
