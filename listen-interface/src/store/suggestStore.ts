import { create } from "zustand";
import { Message } from "../types/message";

interface Suggestion {
  text: string;
}

interface SuggestState {
  suggestions: Suggestion[];
  isLoading: boolean;
  error: string | null;
  lastMessageId: string | null;
  fetchSuggestions: (
    messages: Message[],
    getAccessToken: () => Promise<string | null>,
    locale?: string
  ) => Promise<void>;
  clearSuggestions: () => void;
}

export const useSuggestStore = create<SuggestState>((set) => ({
  suggestions: [],
  isLoading: false,
  error: null,
  lastMessageId: null,
  fetchSuggestions: async (messages, getAccessToken, locale = "en") => {
    if (messages.length === 0) return;

    const lastMessage = messages[messages.length - 1];

    // Don't refetch if we already have suggestions for this message
    if (lastMessage.id === useSuggestStore.getState().lastMessageId) {
      return;
    }

    set({ isLoading: true, error: null });
    try {
      const token = await getAccessToken();
      const response = await fetch(
        process.env.NODE_ENV === "production"
          ? "https://api.listen-rs.com/v1/suggest"
          : "http://localhost:6969/suggest",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
          },
          body: JSON.stringify({
            chat_history: messages,
            locale,
          }),
        }
      );

      if (!response.ok) {
        throw new Error("Failed to fetch suggestions");
      }

      const data = await response.json();
      set({
        suggestions: data.suggestions,
        isLoading: false,
        lastMessageId: lastMessage.id,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Unknown error",
        isLoading: false,
      });
    }
  },
  clearSuggestions: () => set({ suggestions: [], lastMessageId: null }),
}));
