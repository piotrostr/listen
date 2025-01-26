import { usePrivy } from "@privy-io/react-auth";
import { useState, useCallback } from "react";
// import { usePortfolio } from "./usePortfolio";
// import { introPrompt } from "./prompts";

export type MessageDirection = "incoming" | "outgoing";

export interface ChatMessage {
  id: string;
  message: string;
  direction: MessageDirection;
  timestamp: Date;
}

export interface ToolOutput {
  name: string;
  result: string;
}

export interface StreamResponse {
  type: "Message" | "ToolCall" | "Error";
  content: string | ToolOutput;
}

export function useChat() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  // const { data: portfolio } = usePortfolio();
  const { getAccessToken } = usePrivy();
  const [toolOutput, setToolOutput] = useState<ToolOutput | null>(null);

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

        // Send the initial request
        const body = JSON.stringify({
          prompt: userMessage,
          chat_history: messageHistory.filter((msg) => msg.content !== ""),
        });
        console.log("body", body);
        const response = await fetch("http://localhost:8080/v1/stream", {
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
          console.log(chunk);
          const lines = chunk.split("\n");

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
                  case "ToolCall":
                    setToolOutput(data.content as ToolOutput); // TODO zod
                    break;
                  case "Error":
                    console.error("Stream error:", data.content);
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
      } finally {
        setIsLoading(false);
      }
    },
    [messages, updateAssistantMessage, getAccessToken],
  );

  return {
    messages,
    isLoading,
    sendMessage,
    setMessages,
    toolOutput,
  };
}
