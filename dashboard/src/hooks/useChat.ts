import { useState, useCallback } from "react";
import Anthropic from "@anthropic-ai/sdk";

export type MessageDirection = "incoming" | "outgoing";

export interface ChatMessage {
  id: string;
  message: string;
  direction: MessageDirection;
  timestamp: Date;
}

export function useChat() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const anthropic = new Anthropic({
    apiKey: import.meta.env.VITE_ANTHROPIC_API_KEY,
    dangerouslyAllowBrowser: true, // TODO this has to be removed in prod
  });

  const sendMessage = useCallback(
    async (userMessage: string) => {
      setIsLoading(true);

      const userChatMessage: ChatMessage = {
        id: crypto.randomUUID(),
        message: userMessage,
        direction: "outgoing",
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, userChatMessage]);

      try {
        // Convert chat history to Anthropic's format with proper typing
        const messageHistory: Anthropic.MessageParam[] = messages.map(
          (msg) => ({
            role:
              msg.direction === "outgoing"
                ? ("user" as const)
                : ("assistant" as const),
            content: msg.message,
          }),
        );

        const response = await anthropic.messages.create({
          model: "claude-3-opus-20240229",
          max_tokens: 1024,
          messages: [
            ...messageHistory,
            {
              role: "user" as const,
              content: userMessage,
            },
          ],
        });

        const assistantMessage: ChatMessage = {
          id: crypto.randomUUID(),
          message:
            response.content[0].type === "text"
              ? response.content[0].text
              : "Received non-text response",
          direction: "incoming",
          timestamp: new Date(),
        };

        setMessages((prev) => [...prev, assistantMessage]);
      } catch (error) {
        console.error("Error sending message:", error);
      } finally {
        setIsLoading(false);
      }
    },
    [messages, anthropic.messages],
  );

  // Optional: Add a function to clear chat history
  const clearChat = useCallback(() => {
    setMessages([]);
  }, []);

  // Optional: Add a function to save chat history to localStorage
  const saveChat = useCallback(() => {
    localStorage.setItem("chatHistory", JSON.stringify(messages));
  }, [messages]);

  // Optional: Add a function to load chat history from localStorage
  const loadChat = useCallback(() => {
    const savedChat = localStorage.getItem("chatHistory");
    if (savedChat) {
      const parsedChat = JSON.parse(savedChat) as ChatMessage[];
      // Convert string dates back to Date objects
      const chatWithDates = parsedChat.map((msg) => ({
        ...msg,
        timestamp: new Date(msg.timestamp),
      }));
      setMessages(chatWithDates);
    }
  }, []);

  return {
    messages,
    isLoading,
    sendMessage,
    clearChat,
    saveChat,
    loadChat,
  };
}
