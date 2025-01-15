import { useState, useCallback } from "react";
import Anthropic from "@anthropic-ai/sdk";
import { useTools } from "./useTools";
import { PortfolioData, usePortfolio } from "./usePortfolio";

export type MessageDirection = "incoming" | "outgoing";

export interface ChatMessage {
  id: string;
  message: string;
  direction: MessageDirection;
  timestamp: Date;
}

function systemPrompt(portfolio?: PortfolioData) {
  return `as a crypto AI memecoins investor, you are focused on 
projects that have the bleeding edge tech, use informal language but at the same
time extremely sophisticated and mysterious; 
you dont care about shitters, you are looking for real potential - e/acc all the
fucking way - not some grifter-ass dipshits impersonating with fake githubs,
our current portfolio looks like this: ${JSON.stringify(portfolio)}, no need to
outline your thought process in case you can perform the actions straight-away,
just let the user know`;
}

export function useChat() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { data: portfolio } = usePortfolio();
  const { tools } = useTools();

  const anthropic = new Anthropic({
    apiKey: import.meta.env.VITE_ANTHROPIC_API_KEY,
    dangerouslyAllowBrowser: true,
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
        const messageHistory: Anthropic.MessageParam[] = messages.map(
          (msg) => ({
            role:
              msg.direction === "outgoing"
                ? ("user" as const)
                : ("assistant" as const),
            content: msg.message,
          }),
        );

        // Create a temporary message for streaming
        const assistantMessageId = crypto.randomUUID();
        const assistantMessage: ChatMessage = {
          id: assistantMessageId,
          message: "",
          direction: "incoming",
          timestamp: new Date(),
        };

        setMessages((prev) => [...prev, assistantMessage]);

        const stream = anthropic.messages.stream({
          model: "claude-3-opus-20240229",
          max_tokens: 1024,
          system: systemPrompt(portfolio),
          messages: [
            ...messageHistory,
            {
              role: "user" as const,
              content: userMessage,
            },
          ],
          tools,
        });

        stream.on("message", (message) => {
          alert(JSON.stringify(message));
        });

        stream.on("text", (text) => {
          setMessages((prev) => {
            const updatedMessages = [...prev];
            const assistantMessageIndex = updatedMessages.findIndex(
              (msg) => msg.id === assistantMessageId,
            );
            if (assistantMessageIndex !== -1) {
              updatedMessages[assistantMessageIndex] = {
                ...updatedMessages[assistantMessageIndex],
                message: updatedMessages[assistantMessageIndex].message + text,
              };
            }
            return updatedMessages;
          });
        });
      } catch (error) {
        console.error("Error sending message:", error);
      } finally {
        setIsLoading(false);
      }
    },
    [messages, anthropic.messages, tools, portfolio],
  );

  return {
    messages,
    isLoading,
    sendMessage,
  };
}
