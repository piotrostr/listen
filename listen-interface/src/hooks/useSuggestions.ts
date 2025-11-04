import { useCallback } from "react";
import { usePrivy } from "@privy-io/react-auth";
import { useSuggestStore } from "../store/suggestStore";
import { Message } from "../types/message";
import { PortfolioItem } from "../lib/types";

export function useSuggestions(chatId: string) {
  const { getAccessToken } = usePrivy();
  const {
    getSuggestions,
    fetchSuggestions,
    clearSuggestions,
    isLoading,
    error,
    lastMessageHadSpecialTags,
    setLastMessageHadSpecialTags,
  } = useSuggestStore();

  const suggestions = getSuggestions(chatId);

  const fetchSuggestionsWithPortfolio = useCallback(
    async (
      messages: Message[],
      portfolio: PortfolioItem[],
      chatType: string,
      locale?: string
    ) => {
      return fetchSuggestions(
        chatId,
        messages,
        getAccessToken,
        portfolio,
        chatType,
        locale
      );
    },
    [chatId, fetchSuggestions, getAccessToken]
  );

  const clear = useCallback(() => {
    clearSuggestions(chatId);
  }, [chatId, clearSuggestions]);

  return {
    suggestions,
    fetchSuggestions: fetchSuggestionsWithPortfolio,
    clearSuggestions: clear,
    isLoading,
    error,
    lastMessageHadSpecialTags,
    setLastMessageHadSpecialTags,
  };
}