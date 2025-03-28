import { create } from "zustand";
import { Message } from "../types/message";

interface SuggestionsPerChat {
  [chatId: string]: {
    suggestions: string[];
    lastMessageId: string | null;
    hasFailedForMessage: string | null;
  };
}

interface SuggestState {
  suggestionsPerChat: SuggestionsPerChat;
  isLoading: boolean;
  error: string | null;
  fetchSuggestions: (
    chatId: string,
    messages: Message[],
    getAccessToken: () => Promise<string | null>,
    locale?: string
  ) => Promise<void>;
  clearSuggestions: (chatId?: string) => void;
  getSuggestions: (chatId: string) => string[];
}

export const useSuggestStore = create<SuggestState>((set, get) => ({
  suggestionsPerChat: {},
  isLoading: false,
  error: null,

  getSuggestions: (chatId: string) => {
    return get().suggestionsPerChat[chatId]?.suggestions || [];
  },

  fetchSuggestions: async (chatId, messages, getAccessToken, locale = "en") => {
    if (messages.length === 0) return;

    const lastMessage = messages[messages.length - 1];
    const currentChatSuggestions = get().suggestionsPerChat[chatId];

    // Don't refetch if we already have suggestions for this message
    // or if we've already failed for this message
    if (
      lastMessage.id === currentChatSuggestions?.lastMessageId ||
      lastMessage.id === currentChatSuggestions?.hasFailedForMessage
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
          ? "https://api.listen-rs.com/v1/kit/suggest"
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
      set((state) => ({
        suggestionsPerChat: {
          ...state.suggestionsPerChat,
          [chatId]: {
            suggestions: data.suggestions,
            lastMessageId: lastMessage.id,
            hasFailedForMessage: null,
          },
        },
        isLoading: false,
      }));
    } catch (error) {
      set((state) => ({
        error: error instanceof Error ? error.message : "Unknown error",
        isLoading: false,
        suggestionsPerChat: {
          ...state.suggestionsPerChat,
          [chatId]: {
            ...state.suggestionsPerChat[chatId],
            hasFailedForMessage: lastMessage.id,
          },
        },
      }));
      console.error("Failed to fetch suggestions:", error);
    }
  },

  clearSuggestions: (chatId?: string) =>
    set((state) => ({
      suggestionsPerChat: chatId
        ? {
            ...state.suggestionsPerChat,
            [chatId]: {
              suggestions: [],
              lastMessageId: null,
              hasFailedForMessage: null,
            },
          }
        : {}, // Clear all if no chatId provided
      error: null,
    })),
}));
