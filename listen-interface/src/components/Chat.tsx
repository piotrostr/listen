import { usePrivy } from "@privy-io/react-auth";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useChat } from "../hooks/useChat";
import { ChatContainer } from "./ChatContainer";
import { MessageRenderer } from "./MessageRenderer";
import { ShareModal } from "./ShareModal";

const IS_DISABLED = false;

const LoadingIndicator = () => (
  <div className="bg-purple-900/20 text-purple-300 rounded px-4 py-2">...</div>
);

export function Chat({ selectedChatId }: { selectedChatId?: string }) {
  // Parse URL query parameters manually instead of using router
  const [urlParams] = useState(() => {
    const params = new URLSearchParams(window.location.search);
    return {
      chatId: params.get("chatId") || undefined,
      isNewChat: params.get("new") === "true",
      isSharedChat: params.get("shared") === "false",
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
  } = useChat();

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [inputMessage, setInputMessage] = useState("");
  const [isShareModalOpen, setIsShareModalOpen] = useState(false);
  const [shareUrl, setShareUrl] = useState("");
  const { getAccessToken } = usePrivy();
  const [hasLoadedSharedChat, setHasLoadedSharedChat] = useState(false);
  const { t } = useTranslation();

  const RECOMMENDED_QUESTIONS = [
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

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Also scroll to bottom when loading state changes
  useEffect(() => {
    if (!isLoading) {
      messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [isLoading]);

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
      // Create a shareable URL with query parameters
      const url = `${window.location.origin}?chatId=${sharedChatId}&shared=true`;
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
        isSharedChat={!!urlParams.isSharedChat}
      >
        {messages.length === 0 && (
          <div className="flex flex-col items-center justify-center py-12 px-4">
            <h2 className="text-xl font-medium text-white mb-6">
              {t("chat.start_a_conversation")}
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 w-full max-w-3xl">
              {RECOMMENDED_QUESTIONS.map((question, index) => (
                <button
                  key={index}
                  disabled={!question.enabled}
                  onClick={() => handleQuestionClick(question.question)}
                  className="bg-purple-900/30 hover:bg-purple-800/40 text-left p-4 rounded-lg border border-purple-500/30 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <p className="text-white">{question.question}</p>
                </button>
              ))}
            </div>
          </div>
        )}
        {messages.map((message) => (
          <MessageRenderer key={message.id} message={message} />
        ))}
        {isLoading && <LoadingIndicator />}
        <div ref={messagesEndRef} />
      </ChatContainer>

      {/* Share Modal */}
      {isShareModalOpen && (
        <ShareModal url={shareUrl} onClose={() => setIsShareModalOpen(false)} />
      )}
    </>
  );
}
