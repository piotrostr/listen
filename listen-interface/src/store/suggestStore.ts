import { create } from "zustand";
import { Message } from "../types/message";

interface SuggestState {
  suggestions: string[];
  isLoading: boolean;
  error: string | null;
  lastMessageId: string | null;
  retryCount: number;
  hasFailedForMessage: string | null;
  fetchSuggestions: (
    messages: Message[],
    getAccessToken: () => Promise<string | null>,
    locale?: string
  ) => Promise<void>;
  clearSuggestions: () => void;
}

export const useSuggestStore = create<SuggestState>((set, get) => ({
  suggestions: [],
  isLoading: false,
  error: null,
  lastMessageId: null,
  retryCount: 0,
  hasFailedForMessage: null,
  fetchSuggestions: async (messages, getAccessToken, locale = "en") => {
    if (messages.length === 0) return;

    const lastMessage = messages[messages.length - 1];

    // Don't refetch if we already have suggestions for this message
    // or if we've already failed for this message
    if (
      lastMessage.id === get().lastMessageId ||
      lastMessage.id === get().hasFailedForMessage
    ) {
      return;
    }

    set({ isLoading: true, error: null });

    const chatHistory = messages.map((msg) => ({
      role: msg.direction === "outgoing" ? "user" : "assistant",
      content: msg.message,
    }));

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
            chat_history: chatHistory,
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
        hasFailedForMessage: null,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Unknown error",
        isLoading: false,
        hasFailedForMessage: lastMessage.id,
      });
      console.error("Failed to fetch suggestions:", error);
    }
  },
  clearSuggestions: () =>
    set({
      suggestions: [],
      lastMessageId: null,
      retryCount: 0,
      error: null,
      hasFailedForMessage: null,
    }),
}));
