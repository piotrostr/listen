// src/hooks/useChat.ts
import { useState, useCallback } from "react";

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

  const sendMessage = useCallback(async (userMessage: string) => {
    setIsLoading(true);

    // Add user message to chat
    const userChatMessage: ChatMessage = {
      id: crypto.randomUUID(),
      message: userMessage,
      direction: "outgoing",
      timestamp: new Date(),
    };

    setMessages((prev) => [...prev, userChatMessage]);

    try {
      // Simulate AI response (replace this with actual API call later)
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const assistantMessage: ChatMessage = {
        id: crypto.randomUUID(),
        message: `You said: ${userMessage}`,
        direction: "incoming",
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, assistantMessage]);
    } catch (error) {
      console.error("Error sending message:", error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { messages, isLoading, sendMessage };
}
