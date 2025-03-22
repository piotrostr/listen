import { usePrivy } from "@privy-io/react-auth";
import { useNavigate, useSearch } from "@tanstack/react-router";
import {
  createContext,
  ReactNode,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { v4 as uuidv4 } from "uuid";
import { chatCache } from "../hooks/localStorage";
import { useDebounce } from "../hooks/useDebounce";
import { usePrivyWallets } from "../hooks/usePrivyWallet";
import { compactPortfolio } from "../hooks/util";
import { pickSystemPrompt } from "../prompts";
import { usePortfolioStore } from "../store/portfolioStore";
import { useSettingsStore } from "../store/settingsStore";
import {
  Chat,
  Message,
  ToolCallSchema,
  ToolResultSchema,
} from "../types/message";
import { JsonChunkReader } from "./chunk-reader";

interface ChatContextType {
  messages: Message[];
  isLoading: boolean;
  sendMessage: (message: string) => Promise<void>;
  setMessages: (messages: Message[]) => void;
  stopGeneration: () => void;
  shareChat: (chatId: string, cached?: boolean) => Promise<string>;
  loadSharedChat: (chatId: string) => Promise<Chat>;
  isSharedChat: boolean;
  editMessage: (messageId: string, newContent: string) => void;
  resendMessage: (messageId: string, content?: string) => Promise<void>;
  isLastMessageOutgoing: boolean;
}

const ChatContext = createContext<ChatContextType | null>(null);

export const ChatProvider = ({ children }: { children: ReactNode }) => {
  const {
    quickBuyAmount: defaultAmount,
    agentMode,
    chatType,
  } = useSettingsStore();
  const { getAccessToken, user } = usePrivy();
  const {
    chatId,
    new: isNewChat,
    message: initialMessage,
  } = useSearch({ from: "/" });
  const navigate = useNavigate();
  const { data: wallets, isLoading: isLoadingWallets } = usePrivyWallets();
  const { getSolanaAssets, getEvmAssets } = usePortfolioStore();

  const solanaAssets = getSolanaAssets();
  const evmAssets = getEvmAssets();

  const [chat, setChat] = useState<Chat | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const abortControllerRef = useRef<AbortController | null>(null);
  const [sentInitialMessage, setSentInitialMessage] = useState(false);

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

  // Replace the existing backup effect with this debounced version
  const debouncedBackup = useDebounce(async (chatToBackup: Chat) => {
    try {
      await chatCache.set(chatToBackup.id, chatToBackup);
      console.log("Chat backed up successfully:", chatToBackup.id);

      // Dispatch a custom event to notify about chat updates
      window.dispatchEvent(new Event("chatUpdated"));
    } catch (error) {
      console.error("Failed to backup chat:", error);
    }
  }, 1000); // 1 second delay

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
    async (
      userMessage: string,
      options?: { skipAddingUserMessage?: boolean; existingMessageId?: string }
    ) => {
      setIsLoading(true);

      // Create a new abort controller for this request
      abortControllerRef.current = new AbortController();
      const signal = abortControllerRef.current.signal;

      // Only add a user message if not skipping (for resend/edit cases)
      if (!options?.skipAddingUserMessage) {
        const userChatMessage: Message = {
          id: crypto.randomUUID(),
          message: userMessage,
          direction: "outgoing",
          timestamp: new Date(),
          type: "Message",
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
              to: "/",
              search: { chatId: newChatId },
              replace: true,
            });
          }
        } else {
          setChat((prev) => ({
            ...prev!,
            messages: [...(prev?.messages || []), userChatMessage],
            lastMessageAt: new Date(),
          }));
        }
      }

      try {
        // Calculate the history based on whether we're resending a message
        const messageHistory = options?.existingMessageId
          ? chat?.messages
              .slice(
                0,
                chat.messages.findIndex(
                  (msg) => msg.id === options.existingMessageId
                ) + 1
              )
              .map((msg) => ({
                role: msg.direction === "outgoing" ? "user" : "assistant",
                content: msg.message,
              })) || []
          : chat?.messages.map((msg) => ({
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
              type: "Message",
            },
          ],
          lastMessageAt: new Date(),
        }));

        const portfolio = [];
        if (solanaAssets) {
          portfolio.push(...compactPortfolio(solanaAssets));
        }
        if (evmAssets && chatType === "omni") {
          portfolio.push(...compactPortfolio(evmAssets));
        }
        const chat_history = messageHistory.filter((msg) => msg.content !== "");
        const preamble = pickSystemPrompt(
          chatType,
          agentMode,
          portfolio,
          defaultAmount.toString(),
          wallets?.solanaWallet?.toString() || null,
          wallets?.evmWallet?.toString() || null,
          user?.isGuest || false
        );

        const body = JSON.stringify({
          prompt: userMessage,
          chat_history: chat_history,
          chain: chatType,
          preamble,
          features: {
            autonomous: agentMode,
          },
        });

        const response = await fetch(
          process.env.NODE_ENV === "production"
            ? "https://api.listen-rs.com/v1/kit/stream"
            : "http://localhost:6969/stream",
          {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
              Authorization: "Bearer " + (await getAccessToken()),
            },
            body,
            signal,
          }
        );

        let _body = JSON.parse(body);
        delete _body.preamble;
        console.log("body", _body);

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
              case "ToolResult": {
                const toolResult = ToolResultSchema.parse(data.content);
                setChat((prev) => ({
                  ...prev!,
                  messages: [
                    ...prev!.messages,
                    {
                      id: crypto.randomUUID(),
                      message: JSON.stringify(toolResult),
                      direction: "incoming",
                      timestamp: new Date(),
                      type: "ToolResult",
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
                      type: "Message",
                    },
                  ],
                  lastMessageAt: new Date(),
                }));
                break;
              }
              case "ToolCall": {
                const toolCall = ToolCallSchema.parse(data.content);
                setChat((prev) => ({
                  ...prev!,
                  messages: [
                    ...prev!.messages,
                    {
                      id: crypto.randomUUID(),
                      message: JSON.stringify(toolCall),
                      direction: "incoming",
                      timestamp: new Date(),
                      type: "ToolCall",
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
                      type: "Error",
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
                type: "Error",
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
      solanaAssets,
      evmAssets,
      wallets,
      chatType,
      navigate,
    ]
  );

  // If isNewChat is true, clear the current chat
  useEffect(() => {
    if (isNewChat) {
      // Explicitly set chat to null first to ensure clean state
      setChat(null);

      // Generate a new chat ID for new chats
      const newChatId = uuidv4();

      // Navigate to clean URLsearchParams with new chat ID
      setSentInitialMessage(false);
      navigate({
        to: "/",
        search: {
          message: initialMessage,
          chatId: newChatId, // Add the new chat ID
        },
        replace: true,
      });
    }
  }, [isNewChat, navigate, setChat, initialMessage]);

  useEffect(() => {
    if (initialMessage && !sentInitialMessage) {
      setSentInitialMessage(true);
      sendMessage(initialMessage);

      // Clear only the message from URL after sending, keep the chatId
      navigate({
        to: "/",
        search: (prev) => ({
          ...prev,
          message: undefined,
          // Keep the chatId in the URL
        }),
        replace: true,
      });
    }
  }, [initialMessage, isNewChat, sendMessage, sentInitialMessage, navigate]);

  const stopGeneration = () => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
      setIsLoading(false);
    }
  };

  const shareChat = async (chatId: string, cached?: boolean) => {
    let _chat = chat;
    if (!_chat && !cached) {
      return chatId;
    }

    if (cached) {
      _chat = await fetchChatFromCache(chatId);
    }

    try {
      const response = await fetch(
        "https://api.listen-rs.com/v1/adapter/save-chat",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            chat_id: chatId,
            chat: _chat, // The entire chat object
          }),
        }
      );

      if (!response.ok) {
        throw new Error("Failed to share chat");
      }

      // Server should return the same chat ID or a new one if needed
      const result = await response.json();
      return result.chat_id || chatId;
    } catch (error) {
      console.error("Error sharing chat:", error);
      throw error;
    }
  };

  const fetchChatFromCache = async (chatId: string) => {
    const chat = await chatCache.get(chatId);
    return chat;
  };

  const loadSharedChat = async (chatId: string) => {
    try {
      const response = await fetch(
        `https://api.listen-rs.com/v1/adapter/get-chat?chat_id=${chatId}`,
        {
          method: "GET",
          headers: {
            "Content-Type": "application/json",
          },
        }
      );

      if (!response.ok) {
        throw new Error("Failed to load shared chat");
      }

      const sharedChat = await response.json();
      setChat(sharedChat);
      return sharedChat;
    } catch (error) {
      console.error("Error loading shared chat:", error);
      throw error;
    }
  };

  const editMessage = useCallback((messageId: string, newContent: string) => {
    setChat((prev) => {
      if (!prev) return prev;
      const updatedMessages = [...prev.messages];
      const messageIndex = updatedMessages.findIndex(
        (msg) => msg.id === messageId
      );
      if (messageIndex !== -1) {
        updatedMessages[messageIndex] = {
          ...updatedMessages[messageIndex],
          message: newContent,
          edited: true,
        };
      }
      return {
        ...prev,
        messages: updatedMessages,
      };
    });
  }, []);

  const resendMessage = useCallback(
    async (messageId: string, content?: string) => {
      // If content is provided, use it. Otherwise, find the message by ID
      let messageContent;
      if (content !== undefined) {
        messageContent = content;
      } else {
        const messageToResend = chat?.messages.find(
          (msg) => msg.id === messageId
        );
        if (!messageToResend) return;
        messageContent = messageToResend.message;
      }

      // Remove all messages after this one
      setChat((prev) => {
        if (!prev) return prev;
        const messageIndex = prev.messages.findIndex(
          (msg) => msg.id === messageId
        );
        if (messageIndex === -1) return prev;

        return {
          ...prev,
          messages: prev.messages.slice(0, messageIndex + 1),
        };
      });

      // Send the message content again, but skip adding a new user message
      await sendMessage(messageContent, {
        skipAddingUserMessage: true,
        existingMessageId: messageId,
      });
    },
    [chat, sendMessage]
  );

  const isLastMessageOutgoing = checkIfLastMessageIsOutgoing(
    chat?.messages || []
  );

  const value = {
    messages: chat?.messages || [],
    isLoading: isLoadingWallets || isLoading,
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
    shareChat,
    loadSharedChat,
    isSharedChat: !!useSearch({ from: "/" }).shared,
    editMessage,
    resendMessage,
    isLastMessageOutgoing,
  };

  return <ChatContext.Provider value={value}>{children}</ChatContext.Provider>;
};

const checkIfLastMessageIsOutgoing = (messages: Message[]) => {
  if (messages.length === 0) {
    return false;
  }

  const lastMessage = messages[messages.length - 1];

  if (lastMessage.direction === "outgoing") {
    return true;
  }

  if (lastMessage.direction === "incoming" && lastMessage.message === "") {
    return true;
  }

  return false;
};

export const useChat = () => {
  const context = useContext(ChatContext);
  if (!context) {
    throw new Error("useChat must be used within a ChatProvider");
  }
  return context;
};
