import { usePrivy } from "@privy-io/react-auth";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useChat } from "../contexts/ChatContext";
import { useModal } from "../contexts/ModalContext";
import { useSettingsStore } from "../store/settingsStore";
import { useSuggestStore } from "../store/suggestStore";
import {
  ParToolCallSchema,
  RigToolCall,
  ToolCallSchema,
} from "../types/message";
import { ChatContainer } from "./ChatContainer";
import { MessageRenderer } from "./MessageRenderer";
import { NestedAgentOutputDisplay } from "./NestedAgentOutputDisplay";
import { NewChatCarousel } from "./NewChatCarousel";
import { ThinkingIndicator } from "./ThinkingIndicator";
import { ToolCallMessage } from "./ToolCallMessage";

const IS_DISABLED = false;

export function Chat({ selectedChatId }: { selectedChatId?: string }) {
  // Add useEffect to update urlParams when location changes
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    setUrlParams({
      chatId: params.get("chatId") || undefined,
      isNewChat: params.get("new") === "true",
      isSharedChat: params.get("shared") === "true",
      message: params.get("message") || undefined,
    });
  }, [window.location.search]);

  // Update the state declaration
  const [urlParams, setUrlParams] = useState(() => {
    const params = new URLSearchParams(window.location.search);
    return {
      chatId: params.get("chatId") || undefined,
      isNewChat: params.get("new") === "true",
      isSharedChat: params.get("shared") === "true",
      message: params.get("message") || undefined,
    };
  });

  const {
    messages,
    isLoading,
    sendMessage,
    setMessages,
    stopGeneration,
    shareChat,
    loadSharedChat,
    isSharedChat,
    isLastMessageOutgoing,
    nestedAgentOutput,
  } = useChat();

  const {
    getSuggestions,
    isLoading: isSuggestionsLoading,
    fetchSuggestions,
  } = useSuggestStore();

  const { displaySuggestions } = useSettingsStore();

  // Memoize the suggestions selector
  const suggestions = useMemo(() => {
    return urlParams.chatId ? getSuggestions(urlParams.chatId) : [];
  }, [urlParams.chatId, getSuggestions]);

  const lastUserMessageRef = useRef<HTMLDivElement>(null);
  const [inputMessage, setInputMessage] = useState("");
  const { getAccessToken } = usePrivy();
  const [hasLoadedSharedChat, setHasLoadedSharedChat] = useState(false);
  const { i18n } = useTranslation();
  const { openShareModal } = useModal();

  const [activeToolCalls, setActiveToolCalls] = useState<Record<
    string,
    RigToolCall
  > | null>(null);

  const [justSentMessage, setJustSentMessage] = useState(false);

  const RECOMMENDED_QUESTIONS_CAROUSEL = [
    {
      question: "What are the most popular tokens available on World?",
      enabled: true,
      display: "Trade Top Tokens",
    },
    {
      question: "I would like to learn how to invest in crypto effectively.",
      enabled: true,
      display: "Learn to Invest in Crypto",
    },
    {
      question: "Which Mini-Apps offer claiming tokens?",
      enabled: true,
      display: "Claim Daily Tokens",
    },
    {
      question: "What are the most popular tokens available on World?",
      enabled: true,
      display: "Trade Top Tokens",
    },
    {
      question: "I am looking for some cool Mini-Apps, any recommendations?",
      enabled: true,
      display: "Find New Mini-Apps",
    },
    {
      question: "I would like to find ways to passively earn with my $WLD",
      enabled: true,
      display: "Put your $WLD to work",
    },
  ];

  useEffect(() => {
    if (messages.length > 0 && lastUserMessageRef.current && justSentMessage) {
      setTimeout(() => {
        lastUserMessageRef.current?.scrollIntoView({ behavior: "smooth" });
      }, 100);
      // Reset the flag after scrolling
      setJustSentMessage(false);
    }
  }, [messages, justSentMessage]);

  const handleSendMessage = useCallback(
    (message: string) => {
      if (message.trim() === "clear") {
        setMessages([]);
      } else {
        sendMessage(message);
        // Set the flag when sending a new message
        setJustSentMessage(true);
      }
      setInputMessage("");
      if (urlParams.chatId) {
        useSuggestStore.getState().clearSuggestions(urlParams.chatId);
      }
    },
    [sendMessage, setMessages, urlParams.chatId]
  );

  // Focus the input field when creating a new chat
  useEffect(() => {
    if (urlParams.isNewChat) {
      const inputElement = document.querySelector(".chat-input");
      if (inputElement instanceof HTMLTextAreaElement) {
        inputElement.focus();
      }
    }
  }, [urlParams.isNewChat]);

  // Load shared chat if shared parameter is true
  useEffect(() => {
    const fetchSharedChat = async () => {
      if (urlParams.isSharedChat && urlParams.chatId && !hasLoadedSharedChat) {
        try {
          await loadSharedChat(urlParams.chatId);
          setHasLoadedSharedChat(true);
        } catch (error) {
          console.error("Failed to load shared chat:", error);
        }
      }
    };

    fetchSharedChat();
  }, [
    urlParams.isSharedChat,
    urlParams.chatId,
    loadSharedChat,
    getAccessToken,
    hasLoadedSharedChat,
  ]);

  const handleQuestionClick = (question: string) => {
    setInputMessage(question);
    handleSendMessage(question);
  };

  const handleShareChat = async () => {
    const currentChatId = urlParams.chatId || selectedChatId;
    if (!currentChatId || messages.length === 0) return;

    try {
      const sharedChatId = await shareChat(currentChatId);
      // Create a shareable URL that always uses the root path
      const url = `${window.location.origin}/?chatId=${sharedChatId}&shared=true`;
      openShareModal(url);
    } catch (error) {
      console.error("Failed to share chat:", error);
    }
  };

  useEffect(() => {
    if (messages.length > 0) {
      const lastMessage = messages[messages.length - 1];
      let newActiveToolCalls: Record<string, RigToolCall> | null = null;

      if (lastMessage.type === "ToolCall") {
        try {
          const toolCall = ToolCallSchema.parse(
            JSON.parse(lastMessage.message)
          );
          // For a single tool call, represent it within the RigToolCall structure
          const rigToolCall: RigToolCall = {
            id: toolCall.id,
            function: {
              name: toolCall.name,
              // Attempt to parse params, default to raw string if not JSON
              arguments: (() => {
                try {
                  return JSON.parse(toolCall.params);
                } catch {
                  return { rawParams: toolCall.params }; // Keep raw params if parsing fails
                }
              })(),
            },
          };
          newActiveToolCalls = { [toolCall.id]: rigToolCall };
        } catch (error) {
          console.error("Failed to parse tool call:", error);
        }
      } else if (lastMessage.type === "ParToolCall") {
        try {
          const parToolCall = ParToolCallSchema.parse(
            JSON.parse(lastMessage.message)
          );
          newActiveToolCalls = parToolCall.tool_calls.reduce(
            (acc, toolCall) => {
              acc[toolCall.id] = toolCall;
              return acc;
            },
            {} as Record<string, RigToolCall>
          );
        } catch (error) {
          console.error("Failed to parse parallel tool call:", error);
        }
      }

      // Reset states if the last message is not a relevant tool call type
      if (
        lastMessage.type !== "ToolCall" &&
        lastMessage.type !== "ParToolCall"
      ) {
        newActiveToolCalls = null;
      }

      setActiveToolCalls(newActiveToolCalls);
    } else {
      // Clear states if there are no messages
      setActiveToolCalls(null);
    }
  }, [messages]);

  // Combine the suggestion fetching effects into one
  useEffect(() => {
    if (!urlParams.chatId) return;

    const shouldFetchSuggestions =
      displaySuggestions &&
      messages.length > 0 &&
      !isLoading &&
      !isSuggestionsLoading &&
      suggestions.length === 0;

    if (shouldFetchSuggestions) {
      fetchSuggestions(
        urlParams.chatId,
        messages,
        getAccessToken,
        i18n.language
      );
    }
  }, [
    urlParams.chatId,
    messages,
    isLoading,
    isSuggestionsLoading,
    suggestions.length,
    getAccessToken,
    i18n.language,
  ]);

  if (IS_DISABLED) {
    return (
      <ChatContainer inputMessage="" isGenerating={false}>
        <div className="text-white px-4 py-2">disabled</div>
      </ChatContainer>
    );
  }

  return (
    <>
      <ChatContainer
        inputMessage={inputMessage}
        isGenerating={isLoading}
        onSendMessage={handleSendMessage}
        onInputChange={setInputMessage}
        onStopGeneration={stopGeneration}
        onShareChat={messages.length > 0 ? handleShareChat : undefined}
        isSharedChat={isSharedChat || urlParams.isSharedChat}
        handleQuestionClick={handleQuestionClick}
        displayTiles={messages.length === 0}
        hasMessages={messages.length > 0}
        chatId={urlParams.chatId}
      >
        <div className="h-full flex flex-col">
          {messages.length === 0 && (
            <div className="flex flex-col items-center justify-center py-12 px-4">
              <div className="flex flex-col items-center justify-center gap-8 mt-16">
                <NewChatCarousel
                  questions={RECOMMENDED_QUESTIONS_CAROUSEL}
                  onSelect={handleQuestionClick}
                />
              </div>
            </div>
          )}
          {messages.map((message) => (
            <MessageRenderer
              key={message.id}
              message={message}
              messages={messages}
              lastUserMessageRef={lastUserMessageRef}
            />
          ))}
          <div className="flex flex-row items-center gap-2 pl-3 mt-2 flex-wrap justify-start">
            {isLoading && <ThinkingIndicator />}
            {/* Render thinking indicator if loading and no specific tools are active yet */}
            {isLoading && !activeToolCalls && isLastMessageOutgoing && (
              <ToolCallMessage
                toolCall={{
                  id: "thinking-indicator", // Use a distinct ID
                  name: "thinking",
                  params: "non-relevant",
                }}
              />
            )}
            {/* Render active tool calls */}
            <div className="flex flex-col gap-2">
              {activeToolCalls &&
                Object.values(activeToolCalls).map((rigToolCall) => (
                  <ToolCallMessage
                    key={rigToolCall.id}
                    // Adapt RigToolCall to the ToolCall shape expected by ToolCallMessage
                    toolCall={{
                      id: rigToolCall.id,
                      name: rigToolCall.function.name,
                      params: JSON.stringify(rigToolCall.function.arguments), // Stringify arguments for ToolCallMessage
                    }}
                  />
                ))}
            </div>
          </div>
          {nestedAgentOutput && isLoading && (
            <NestedAgentOutputDisplay content={nestedAgentOutput.content} />
          )}
        </div>
        {messages.length !== 0 && <div className="flex-grow min-h-[75vh]" />}
      </ChatContainer>
    </>
  );
}
