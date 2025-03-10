import { usePrivy } from "@privy-io/react-auth";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useChat } from "../hooks/useChat";
import { ChatContainer } from "./ChatContainer";
import { MessageRenderer } from "./MessageRenderer";
import { NewChatCarousel } from "./NewChatCarousel";
import { ShareModal } from "./ShareModal";

const IS_DISABLED = false;

const LoadingIndicator = () => (
  <div className="bg-purple-900/20 text-purple-300 rounded px-4 py-2">...</div>
);

export function Chat({ selectedChatId }: { selectedChatId?: string }) {
  // Add useEffect to update urlParams when location changes
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    setUrlParams({
      chatId: params.get("chatId") || undefined,
      isNewChat: params.get("new") === "true",
      isSharedChat: params.get("shared") === "true",
    });
  }, [window.location.search]);

  // Update the state declaration
  const [urlParams, setUrlParams] = useState(() => {
    const params = new URLSearchParams(window.location.search);
    return {
      chatId: params.get("chatId") || undefined,
      isNewChat: params.get("new") === "true",
      isSharedChat: params.get("shared") === "true",
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
  } = useChat();

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [inputMessage, setInputMessage] = useState("");
  const [isShareModalOpen, setIsShareModalOpen] = useState(false);
  const [shareUrl, setShareUrl] = useState("");
  const { getAccessToken } = usePrivy();
  const [hasLoadedSharedChat, setHasLoadedSharedChat] = useState(false);
  const { t } = useTranslation();

  const RECOMMENDED_QUESTIONS_TILES = [
    {
      question: t(
        "chat.recommended_questions.what_actions_can_you_perform_for_me"
      ),
      enabled: true,
    },
    {
      question: t(
        "chat.recommended_questions.how_do_pipelines_work_and_what_pipelines_can_you_create_for_me"
      ),
      enabled: true,
    },
    {
      question: t("chat.recommended_questions.what_chains_are_supported"),
      enabled: true,
    },
    {
      question: t(
        "chat.recommended_questions.what_tokens_have_received_largest_inflows_outflows_in_the_past_days"
      ),
      enabled: true,
    },
  ];

  const RECOMMENDED_QUESTIONS_CAROUSEL = [
    {
      question: "whats the most viral token right now?",
      enabled: true,
    },
    {
      question: "what does LP mean?",
      enabled: true,
    },
    {
      question: "how to manage risk when trading memecoins?",
      enabled: true,
    },
    {
      question: "buy the Solana dip",
      enabled: true,
    },
    {
      question: "research arcdotfun for me", // TODO X search
      enabled: true,
    },
    {
      question: "what is the best way to buy a new token?",
      enabled: true,
    },
  ];

  const handleSendMessage = useCallback(
    (message: string) => {
      if (message.trim() === "clear") {
        setMessages([]);
      } else {
        sendMessage(message);
      }
      setInputMessage("");
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
      setShareUrl(url);
      setIsShareModalOpen(true);
    } catch (error) {
      console.error("Failed to share chat:", error);
    }
  };

  if (IS_DISABLED) {
    return (
      <ChatContainer inputMessage="" isGenerating={false}>
        <div className="text-purple-300 px-4 py-2">disabled</div>
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
      >
        <div className="h-full flex flex-col">
          {messages.length === 0 && (
            <div className="flex flex-col items-center justify-center py-12 px-4">
              <div className="flex flex-col items-center justify-center">
                <NewChatCarousel
                  questions={RECOMMENDED_QUESTIONS_CAROUSEL}
                  onSelect={handleQuestionClick}
                />
              </div>
            </div>
          )}
          {messages.map((message) => (
            <MessageRenderer key={message.id} message={message} />
          ))}
          {isLoading && <LoadingIndicator />}
          <div ref={messagesEndRef} />
        </div>
      </ChatContainer>

      {/* Share Modal */}
      {isShareModalOpen && (
        <ShareModal url={shareUrl} onClose={() => setIsShareModalOpen(false)} />
      )}
    </>
  );
}
