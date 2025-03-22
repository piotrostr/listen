import { usePrivy } from "@privy-io/react-auth";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useChat } from "../contexts/ChatContext";
import { useModal } from "../contexts/ModalContext";
import { ToolCall, ToolCallSchema } from "../types/message";
import { ChatContainer } from "./ChatContainer";
import { MessageRenderer } from "./MessageRenderer";
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
  } = useChat();

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [inputMessage, setInputMessage] = useState("");
  const { getAccessToken } = usePrivy();
  const [hasLoadedSharedChat, setHasLoadedSharedChat] = useState(false);
  const { t } = useTranslation();
  const { openShareModal } = useModal();

  const [toolBeingCalled, setToolBeingCalled] = useState<ToolCall | null>(null);

  const RECOMMENDED_QUESTIONS_CAROUSEL = [
    {
      question: t("recommended_questions.whats_the_most_viral_token_right_now"),
      enabled: true,
    },
    {
      question: t("recommended_questions.what_does_lp_mean"),
      enabled: true,
    },
    {
      question: t(
        "recommended_questions.how_to_manage_risk_when_trading_memecoins"
      ),
      enabled: true,
    },
    {
      question: t("recommended_questions.buy_the_solana_dip"),
      enabled: true,
    },
    {
      question: t("recommended_questions.research_arcdotfun_for_me"), // TODO X search
      enabled: true,
    },
  ];

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  const handleSendMessage = useCallback(
    (message: string) => {
      if (message.trim() === "clear") {
        setMessages([]);
      } else {
        sendMessage(message);
      }
      setInputMessage("");

      if (messages?.length > 0) {
        scrollToBottom();
      }
    },
    [sendMessage, setMessages]
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
      if (lastMessage.type === "ToolCall") {
        try {
          const toolCall = ToolCallSchema.parse(
            JSON.parse(lastMessage.message)
          );
          setToolBeingCalled(toolCall);
        } catch (error) {
          console.error("Failed to parse tool call:", error);
        }
      } else {
        setToolBeingCalled(null);
      }
    }
  }, [messages]);

  if (IS_DISABLED) {
    return (
      <ChatContainer inputMessage="" isGenerating={false}>
        <div className="text-white px-4 py-2">disabled</div>
      </ChatContainer>
    );
  }

  console.log(isLoading, toolBeingCalled, isLastMessageOutgoing);

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
            />
          ))}
          <div className="flex flex-row items-center gap-2 pl-3 mt-2">
            {isLoading && <ThinkingIndicator />}
            {isLoading && !toolBeingCalled && isLastMessageOutgoing && (
              <ToolCallMessage
                toolCall={{
                  id: "non-relevant",
                  params: "non-relevant",
                  name: "thinking",
                }}
              />
            )}
            {toolBeingCalled && <ToolCallMessage toolCall={toolBeingCalled} />}
          </div>
        </div>
        <div ref={messagesEndRef} />
        {messages.length !== 0 && (
          <div className="flex-grow min-h-[68vh] md:min-h-[85vh]" />
        )}
      </ChatContainer>
    </>
  );
}
