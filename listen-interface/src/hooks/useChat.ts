import { usePrivy } from "@privy-io/react-auth";
import { useCallback, useState } from "react";
import { z } from "zod";
import { config } from "../config";
import { introPrompt } from "./prompts";
import { useEvmPortfolio } from "./useEvmPortfolioAlchemy";
import { useSolanaPortfolio } from "./useSolanaPortfolio";

export type MessageDirection = "incoming" | "outgoing";

export interface Message {
  id: string;
  message: string;
  direction: MessageDirection;
  timestamp: Date;
  isToolCall: boolean;
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

class JsonChunkReader {
  private buffer = "";

  append(chunk: string): StreamResponse[] {
    this.buffer += chunk;
    const messages: StreamResponse[] = [];
    const lines = this.buffer.split("\n");

    // Keep the last line in the buffer if it doesn't end with newline
    this.buffer = lines[lines.length - 1];

    // Process all complete lines except the last one
    for (let i = 0; i < lines.length - 1; i++) {
      const line = lines[i];
      if (line.startsWith("data: ")) {
        try {
          const jsonStr = line.slice(6);
          const data = JSON.parse(jsonStr);
          messages.push(data);
        } catch (e) {
          console.warn("Failed to parse JSON from line:", line, e);
        }
      }
    }

    return messages;
  }
}

export function useChat() {
  const { data: solanaPortfolio } = useSolanaPortfolio();
  const { data: evmPortfolio } = useEvmPortfolio();
  const { user } = usePrivy();
  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { getAccessToken } = usePrivy();

  const updateAssistantMessage = useCallback(
    (assistantMessageId: string, newContent: string) => {
      setMessages((prev) => {
        const updatedMessages = [...prev];
        const assistantMessageIndex = updatedMessages.findIndex(
          (msg) => msg.id === assistantMessageId
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
    []
  );

  const sendMessage = useCallback(
    async (userMessage: string) => {
      setIsLoading(true);

      const userChatMessage: Message = {
        id: crypto.randomUUID(),
        message: userMessage,
        direction: "outgoing",
        timestamp: new Date(),
        isToolCall: false,
      };

      setMessages((prev) => [...prev, userChatMessage]);

      try {
        const messageHistory = messages.map((msg) => ({
          role: msg.direction === "outgoing" ? "user" : "assistant",
          content: msg.message,
        }));

        let currentAssistantMessageId = crypto.randomUUID();
        setMessages((prev) => [
          ...prev,
          {
            id: currentAssistantMessageId,
            message: "",
            direction: "incoming",
            timestamp: new Date(),
            isToolCall: false,
          },
        ]);

        if (
          !user ||
          solanaPortfolio === undefined ||
          evmPortfolio === undefined
        ) {
          console.error("User or portfolio not available");
        }

        const chat_history = messageHistory.filter((msg) => msg.content !== "");
        if (chat_history.length == 0) {
          userMessage +=
            " " +
            introPrompt(
              [...solanaPortfolio!, ...evmPortfolio!],
              user?.wallet?.address || ""
            );
        }

        const body = JSON.stringify({
          prompt: userMessage,
          chat_history: chat_history,
          chain: "omni",
        });

        const response = await fetch(config.API_BASE_URL + "/v1/kit/stream", {
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
        const jsonReader = new JsonChunkReader();

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          const chunk = decoder.decode(value);
          const messages = jsonReader.append(chunk);

          for (const data of messages) {
            switch (data.type) {
              case "Message":
                updateAssistantMessage(
                  currentAssistantMessageId,
                  data.content as string
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
                    isToolCall: true,
                  },
                ]);
                // Start a new assistant message after tool call
                currentAssistantMessageId = crypto.randomUUID();
                setMessages((prev) => [
                  ...prev,
                  {
                    id: currentAssistantMessageId,
                    message: "",
                    direction: "incoming",
                    timestamp: new Date(),
                    isToolCall: false,
                  },
                ]);
                break;
              }
              case "Error":
                console.error("Stream error:", data.content);
                setMessages((prev) => [
                  ...prev,
                  {
                    id: crypto.randomUUID(),
                    message: `Error: ${data.content}`,
                    direction: "incoming",
                    timestamp: new Date(),
                    isToolCall: false,
                  },
                ]);
                break;
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
            message: `An error occurred: ${
              error instanceof Error ? error.message : "Unknown error"
            }`,
            direction: "incoming",
            timestamp: new Date(),
            isToolCall: false,
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
      solanaPortfolio,
      evmPortfolio,
      user,
    ]
  );

  return {
    messages,
    isLoading,
    sendMessage,
    setMessages,
  };
}
