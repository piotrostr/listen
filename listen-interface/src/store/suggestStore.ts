import { create } from "zustand";
import { Message } from "../types/message";

interface Suggestion {
  text: string;
}

interface SuggestState {
  suggestions: Suggestion[];
  isLoading: boolean;
  error: string | null;
  fetchSuggestions: (
    chatHistory: Message[],
    getAccessToken: () => Promise<string>,
    locale?: string
  ) => Promise<void>;
}

export const useSuggestStore = create<SuggestState>((set) => ({
  suggestions: [],
  isLoading: false,
  error: null,
  fetchSuggestions: async (
    chatHistory: Message[],
    getAccessToken: () => Promise<string>,
    locale = "en"
  ) => {
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
            chat_history: chatHistory,
            locale,
          }),
        }
      );

      if (!response.ok) {
        throw new Error("Failed to fetch suggestions");
      }

      const data = await response.json();
      set({ suggestions: data.suggestions, isLoading: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Unknown error",
        isLoading: false,
      });
    }
  },
}));
