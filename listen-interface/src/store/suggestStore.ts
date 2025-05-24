import { create } from "zustand";
import { config } from "../config";
import { compactPortfolio } from "../lib/util";
import { Message } from "../types/message";
import { usePortfolioStore } from "./portfolioStore";
import { useSettingsStore } from "./settingsStore";

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
  lastMessageHadSpecialTags: boolean;
  fetchSuggestions: (
    chatId: string,
    messages: Message[],
    getAccessToken: () => Promise<string | null>,
    locale?: string
  ) => Promise<void>;
  clearSuggestions: (chatId?: string) => void;
  getSuggestions: (chatId: string) => string[];
  setLastMessageHadSpecialTags: (value: boolean) => void;
}

export const useSuggestStore = create<SuggestState>((set, get) => ({
  suggestionsPerChat: {},
  isLoading: false,
  error: null,
  lastMessageHadSpecialTags: false,

  getSuggestions: (chatId: string) => {
    return get().suggestionsPerChat[chatId]?.suggestions || [];
  },

  setLastMessageHadSpecialTags: (value: boolean) => {
    set({ lastMessageHadSpecialTags: value });
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

    // Instead of using the hook, access the store directly
    const chatType = useSettingsStore.getState().chatType;
    const portfolioStore = usePortfolioStore.getState();
    const solanaAssets = portfolioStore.getSolanaAssets();
    const evmAssets = portfolioStore.getEvmAssets();

    const portfolio = [];
    if (solanaAssets) {
      portfolio.push(...compactPortfolio(solanaAssets));
    }
    if (evmAssets && chatType === "omni") {
      portfolio.push(...compactPortfolio(evmAssets));
    }

    set({ isLoading: true, error: null });

    const chatHistory = messages.map((msg) => ({
      role: msg.direction === "outgoing" ? "user" : "assistant",
      content: msg.message,
    }));

    try {
      const token = await getAccessToken();
      const response = await fetch(`${config.kitEndpoint}/suggest`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({
          chat_history: chatHistory,
          locale,
          context: JSON.stringify(portfolio),
        }),
      });

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
