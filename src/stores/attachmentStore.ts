import { create } from "zustand";
import type { LogFileSummary, ImageAttachmentSummary } from "@/lib/tauriCommands";
import {
  listAllLogFilesCmd,
  listAllImageAttachmentsCmd,
} from "@/lib/tauriCommands";

interface AttachmentState {
  logFiles: LogFileSummary[];
  images: ImageAttachmentSummary[];
  isLoading: boolean;
  error: string | null;
  searchQuery: string;

  loadAttachments(filter?: { issueId?: string }): Promise<void>;
  searchAttachments(query: string): Promise<void>;
  setSearchQuery(q: string): void;
}

export const useAttachmentStore = create<AttachmentState>((set, get) => ({
  logFiles: [],
  images: [],
  isLoading: false,
  error: null,
  searchQuery: "",

  loadAttachments: async (filter = {}) => {
    set({ isLoading: true, error: null });
    try {
      const [logFiles, images] = await Promise.all([
        listAllLogFilesCmd(undefined, filter.issueId),
        listAllImageAttachmentsCmd(undefined, filter.issueId),
      ]);
      set({ logFiles, images, isLoading: false });
    } catch (err) {
      set({ error: String(err), isLoading: false });
    }
  },

  searchAttachments: async (query: string) => {
    const search = query || undefined;
    set({ isLoading: true, searchQuery: query, error: null });
    try {
      const [logFiles, images] = await Promise.all([
        listAllLogFilesCmd(search),
        listAllImageAttachmentsCmd(search),
      ]);
      set({ logFiles, images, isLoading: false });
    } catch (err) {
      set({ error: String(err), isLoading: false });
    }
  },

  setSearchQuery: (q) => set({ searchQuery: q }),
}));
