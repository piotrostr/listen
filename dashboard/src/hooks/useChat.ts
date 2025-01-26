import { useState, useCallback } from "react";
import Anthropic from "@anthropic-ai/sdk";
import { usePortfolio } from "./usePortfolio";
import { PortfolioData } from "./types";

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
our current portfolio looks like this: ${JSON.stringify(portfolio)}

before you execute any larger swaps, anything over 0.5 solana or roughly 100 usd,
confirm with the user the exact amount you are going to run through, as well as the
token mint with https://solscan.io/account/{<insert token address>} so they validate
`;
}

export function useChat() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { data: portfolio } = usePortfolio();

  const anthropic = new Anthropic({
    apiKey: import.meta.env.VITE_ANTHROPIC_API_KEY,
    dangerouslyAllowBrowser: true,
  });

  const updateAssistantMessage = useCallback(
    (assistantMessageId: string, newContent: string) => {
      setMessages((prev) => {
        const updatedMessages = [...prev];
        const assistantMessageIndex = updatedMessages.findIndex(
          (msg) => msg.id === assistantMessageId,
        );
        if (assistantMessageIndex !== -1) {
          updatedMessages[assistantMessageIndex] = {
            ...updatedMessages[assistantMessageIndex],
            message:
              updatedMessages[assistantMessageIndex].message + newContent,
          };
        }
        return updatedMessages;
      });
    },
    [],
  );

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
            role: msg.direction === "outgoing" ? "user" : "assistant",
            content: msg.message,
          }),
        );

        const assistantMessageId = crypto.randomUUID();
        setMessages((prev) => [
          ...prev,
          {
            id: assistantMessageId,
            message: "",
            direction: "incoming",
            timestamp: new Date(),
          },
        ]);

        const stream = anthropic.messages.stream({
          model: "claude-3-5-sonnet-latest",
          max_tokens: 1024,
          system: systemPrompt(portfolio),
          messages: [...messageHistory, { role: "user", content: userMessage }],
        });

        stream.on("text", (text) =>
          updateAssistantMessage(assistantMessageId, text),
        );

        stream.on("message", async (message) => {
          if (message.content) {
            for (const content of message.content) {
              if (content.type === "tool_use") {
                // await handleToolExecution(content.name, content.input);
              }
            }
          }
        });
      } catch (error) {
        console.error("Error sending message:", error);
      } finally {
        setIsLoading(false);
      }
    },
    [messages, anthropic.messages, portfolio, updateAssistantMessage],
  );

  return {
    messages,
    isLoading,
    sendMessage,
  };
}
