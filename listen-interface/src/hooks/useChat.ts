import { usePrivy } from "@privy-io/react-auth";
import { useNavigate, useSearch } from "@tanstack/react-router";
import { useCallback, useEffect, useRef, useState } from "react";
import { v4 as uuidv4 } from "uuid";
import { config } from "../config";
import { chatCache } from "./localStorage";
import { systemPrompt } from "./prompts";
import { Chat, Message, StreamResponse, ToolOutputSchema } from "./types";
import { useChatType } from "./useChatType";
import { useDebounce } from "./useDebounce";
import { useEvmPortfolio } from "./useEvmPortfolioAlchemy";
import { useSolanaPortfolio } from "./useSolanaPortfolio";

class JsonChunkReader {
  private buffer = "";

  append(chunk: string): StreamResponse[] {
    this.buffer += chunk;
    const messages: StreamResponse[] = [];
    const lines = this.buffer.split("\n");

    this.buffer = lines[lines.length - 1];

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
  const { user, getAccessToken } = usePrivy();
  const { chatType } = useChatType();
  const { chatId, new: isNewChat } = useSearch({ from: "/chat" });
  const navigate = useNavigate();

  const [chat, setChat] = useState<Chat | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const abortControllerRef = useRef<AbortController | null>(null);

  // Load existing chat if chatId is present and not creating a new chat
  useEffect(() => {
    const loadChat = async () => {
      if (!chatId || isNewChat) return;
      const existingChat = await chatCache.get(chatId);
      if (existingChat) {
        setChat(existingChat);
      }
    };
    loadChat();
  }, [chatId, isNewChat]);

  // If isNewChat is true, clear the current chat
  useEffect(() => {
    if (isNewChat) {
      setChat(null);
      // Remove the 'new' parameter but keep the URL at /chat
      navigate({ to: "/chat", search: {}, replace: true });
    }
  }, [isNewChat, navigate]);

  // Replace the existing backup effect with this debounced version
  const debouncedBackup = useDebounce(async (chatToBackup: Chat) => {
    try {
      await chatCache.set(chatToBackup.id, chatToBackup);
      console.log("Chat backed up successfully:", chatToBackup.id);
    } catch (error) {
      console.error("Failed to backup chat:", error);
    }
  }, 1000); // 2 second delay

  useEffect(() => {
    if (!chat?.id) return;
    debouncedBackup(chat);
  }, [chat, debouncedBackup]);

  const updateAssistantMessage = useCallback(
    (assistantMessageId: string, newContent: string) => {
      setChat((prev) => {
        if (!prev) return prev;
        const updatedMessages = [...prev?.messages];
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
        return {
          ...prev,
          messages: updatedMessages,
          lastMessageAt: new Date(),
        };
      });
    },
    []
  );

  const sendMessage = useCallback(
    async (userMessage: string) => {
      setIsLoading(true);

      // Create a new abort controller for this request
      abortControllerRef.current = new AbortController();
      const signal = abortControllerRef.current.signal;

      const userChatMessage: Message = {
        id: crypto.randomUUID(),
        message: userMessage,
        direction: "outgoing",
        timestamp: new Date(),
        isToolCall: false,
      };

      // Initialize new chat if none exists
      if (!chat) {
        const newChatId = chatId || uuidv4();
        const newChat: Chat = {
          id: newChatId,
          messages: [userChatMessage],
          createdAt: new Date(),
          lastMessageAt: new Date(),
          title: userMessage.slice(0, 50),
        };
        setChat(newChat);

        // Only navigate if this is truly a new chat (no chatId in URL)
        if (!chatId) {
          navigate({
            to: "/chat",
            search: { chatId: newChatId },
            replace: true,
          });
        }
      } else {
        setChat((prev) => ({
          ...prev!,
          messages: [...prev!.messages, userChatMessage],
          lastMessageAt: new Date(),
        }));
      }

      try {
        const messageHistory =
          chat?.messages.map((msg) => ({
            role: msg.direction === "outgoing" ? "user" : "assistant",
            content: msg.message,
          })) || [];

        let currentAssistantMessageId = crypto.randomUUID();
        setChat((prev) => ({
          ...prev!,
          messages: [
            ...prev!.messages,
            {
              id: currentAssistantMessageId,
              message: "",
              direction: "incoming",
              timestamp: new Date(),
              isToolCall: false,
            },
          ],
          lastMessageAt: new Date(),
        }));

        if (
          !user ||
          solanaPortfolio === undefined ||
          evmPortfolio === undefined
        ) {
          // TODO display "portfolio loading" and disable chat prior
          alert("User or portfolio not available");
        }

        const chat_history = messageHistory.filter((msg) => msg.content !== "");
        const preamble = systemPrompt(
          [...solanaPortfolio!, ...evmPortfolio!],
          user?.wallet?.address || "",
          user?.wallet?.address || ""
        );

        const body = JSON.stringify({
          prompt: userMessage,
          chat_history: chat_history,
          chain: chatType,
          preamble,
        });

        const response = await fetch(config.API_BASE_URL + "/v1/kit/stream", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: "Bearer " + (await getAccessToken()),
          },
          body,
          signal,
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
                setChat((prev) => ({
                  ...prev!,
                  messages: [
                    ...prev!.messages,
                    {
                      id: crypto.randomUUID(),
                      message: `Tool ${toolOutput.name}: ${toolOutput.result}`,
                      direction: "incoming",
                      timestamp: new Date(),
                      isToolCall: true,
                    },
                  ],
                  lastMessageAt: new Date(),
                }));
                // Start a new assistant message after tool call
                currentAssistantMessageId = crypto.randomUUID();
                setChat((prev) => ({
                  ...prev!,
                  messages: [
                    ...prev!.messages,
                    {
                      id: currentAssistantMessageId,
                      message: "",
                      direction: "incoming",
                      timestamp: new Date(),
                      isToolCall: false,
                    },
                  ],
                  lastMessageAt: new Date(),
                }));
                break;
              }
              case "Error":
                console.error("Stream error:", data.content);
                setChat((prev) => ({
                  ...prev!,
                  messages: [
                    ...prev!.messages,
                    {
                      id: crypto.randomUUID(),
                      message: `Error: ${data.content}`,
                      direction: "incoming",
                      timestamp: new Date(),
                      isToolCall: false,
                    },
                  ],
                  lastMessageAt: new Date(),
                }));
                break;
            }
          }
        }
      } catch (error) {
        // Check if this was an abort error
        if (error instanceof DOMException && error.name === "AbortError") {
          console.log("Request was aborted");
          // You might want to add a message indicating the generation was stopped
          setChat((prev) => ({
            ...prev!,
            messages: [
              ...prev!.messages.slice(0, -1), // Remove the incomplete assistant message
            ],
            lastMessageAt: new Date(),
          }));
        } else {
          // Handle other errors as before
          console.error("Error sending message:", error);
          setChat((prev) => ({
            ...prev!,
            messages: [
              ...prev!.messages,
              {
                id: crypto.randomUUID(),
                message: `An error occurred: ${error instanceof Error ? error.message : "Unknown error"}`,
                direction: "incoming",
                timestamp: new Date(),
                isToolCall: false,
              },
            ],
            lastMessageAt: new Date(),
          }));
        }
      } finally {
        setIsLoading(false);
        abortControllerRef.current = null;
      }
    },
    [
      chat,
      chatId,
      updateAssistantMessage,
      getAccessToken,
      solanaPortfolio,
      evmPortfolio,
      user,
      chatType,
      navigate,
    ]
  );

  const stopGeneration = () => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
      setIsLoading(false);
    }
  };

  return {
    messages: chat?.messages || [],
    isLoading,
    sendMessage,
    setMessages: (messages: Message[]) =>
      setChat((prev) =>
        prev
          ? {
              ...prev,
              messages,
              lastMessageAt: new Date(),
            }
          : {
              id: chatId || uuidv4(),
              messages,
              createdAt: new Date(),
              lastMessageAt: new Date(),
              title: messages[0]?.message.slice(0, 50),
            }
      ),
    stopGeneration,
  };
}
