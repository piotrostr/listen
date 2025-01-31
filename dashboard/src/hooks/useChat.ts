import { usePrivy } from "@privy-io/react-auth";
import { useState, useCallback } from "react";
import { z } from "zod";
import { introPrompt } from "./prompts";
import { usePortfolio } from "./usePortfolio";
import { config } from "../config";
import { useChatType } from "./useChatType";

export type MessageDirection = "incoming" | "outgoing";

export interface Message {
  id: string;
  message: string;
  direction: MessageDirection;
  timestamp: Date;
}

const ToolOutputSchema = z.object({
  name: z.string(),
  result: z.string(),
});

export type ToolOutput = z.infer<typeof ToolOutputSchema>;

export interface StreamResponse {
  type: "Message" | "ToolCall" | "Error";
  content: string | ToolOutput;
}

export function useChat() {
  const { data: portfolio } = usePortfolio();
  const { user } = usePrivy();
  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { chatType } = useChatType();
  const { getAccessToken } = usePrivy();

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

      const userChatMessage: Message = {
        id: crypto.randomUUID(),
        message: userMessage,
        direction: "outgoing",
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, userChatMessage]);

      try {
        const messageHistory = messages.map((msg) => ({
          role: msg.direction === "outgoing" ? "user" : "assistant",
          content: msg.message,
        }));

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

        const chat_history = messageHistory.filter((msg) => msg.content !== "");
        if (chat_history.length > 0) {
          chat_history[0].content +=
            " " + introPrompt(portfolio, user?.wallet?.address || "");
        }

        const body = JSON.stringify({
          prompt: userMessage,
          chat_history: chat_history,
          chain: chatType,
        });

        const response = await fetch(config.API_BASE_URL + "/v1/stream", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: "Bearer " + (await getAccessToken()),
          },
          body,
        });

        if (!response.ok) {
          throw new Error("Failed to initialize stream");
        }

        const reader = response.body?.getReader();
        if (!reader) throw new Error("No reader available");

        const decoder = new TextDecoder();

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          const chunk = decoder.decode(value);
          const lines = chunk.split("\n");

          console.log(chunk);

          for (const line of lines) {
            if (line.startsWith("data: ")) {
              const jsonStr = line.slice(6);
              try {
                const data: StreamResponse = JSON.parse(jsonStr);

                switch (data.type) {
                  case "Message":
                    updateAssistantMessage(
                      assistantMessageId,
                      data.content as string,
                    );
                    break;
                  case "ToolCall": {
                    const toolOutput = ToolOutputSchema.parse(data.content);
                    setMessages((prev) => [
                      ...prev,
                      {
                        id: crypto.randomUUID(),
                        message: `Tool ${toolOutput.name}: ${toolOutput.result}`,
                        direction: "incoming",
                        timestamp: new Date(),
                      },
                    ]);
                    break;
                  }
                  case "Error":
                    console.error("Stream error:", data.content);
                    // Optionally add error as a message
                    setMessages((prev) => [
                      ...prev,
                      {
                        id: crypto.randomUUID(),
                        message: `Error: ${data.content}`,
                        direction: "incoming",
                        timestamp: new Date(),
                      },
                    ]);
                    break;
                }
              } catch (e) {
                console.error("Failed to parse SSE data:", e);
              }
            }
          }
        }
      } catch (error) {
        console.error("Error sending message:", error);
        // Add error message to chat
        setMessages((prev) => [
          ...prev,
          {
            id: crypto.randomUUID(),
            message: `An error occurred: ${error instanceof Error ? error.message : "Unknown error"}`,
            direction: "incoming",
            timestamp: new Date(),
          },
        ]);
      } finally {
        setIsLoading(false);
      }
    },
    [
      messages,
      updateAssistantMessage,
      getAccessToken,
      portfolio,
      user,
      chatType,
    ],
  );

  return {
    messages,
    isLoading,
    sendMessage,
    setMessages,
  };
}
