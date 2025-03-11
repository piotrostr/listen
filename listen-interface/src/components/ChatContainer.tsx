import { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { ChatInput } from "./ChatInput";
import { NewChatTiles } from "./NewChatTiles";

interface ChatContainerProps {
  inputMessage: string;
  isGenerating?: boolean;
  onSendMessage?: (message: string) => void;
  onInputChange?: (message: string) => void;
  onStopGeneration?: () => void;
  onShareChat?: () => void;
  isSharedChat?: boolean;
  children: ReactNode;
  handleQuestionClick?: (question: string) => void;
  displayTiles?: boolean;
  hasMessages?: boolean;
}

export function ChatContainer({
  inputMessage,
  isGenerating = false,
  onSendMessage = () => {},
  onInputChange = () => {},
  onStopGeneration = () => {},
  onShareChat,
  isSharedChat = false,
  children,
  handleQuestionClick,
  displayTiles = false,
  hasMessages = false,
}: ChatContainerProps) {
  const { t } = useTranslation();
  const RECOMMENDED_QUESTIONS_TILES = [
    {
      question: t("recommended_questions.what_actions_can_you_perform_for_me"),
      enabled: true,
      display: t("recommended_questions.learn_about_listen"),
    },
    {
      question: t(
        "recommended_questions.how_do_pipelines_work_and_what_pipelines_can_you_create_for_me"
      ),
      enabled: true,
      display: t("recommended_questions.complex_made_simple"),
    },
    {
      question: t("recommended_questions.what_chains_are_supported"),
      enabled: true,
      display: t("recommended_questions.supported_chains"),
    },
    {
      question: t(
        "recommended_questions.what_tokens_have_received_largest_inflows_outflows_in_the_past_days"
      ),
      enabled: true,
      display: t("recommended_questions.discover_coins"),
    },
  ];

  return (
    <div className="relative mx-auto flex h-full w-full max-w-3xl flex-col md:px-2">
      <div
        className="flex-1 overflow-y-auto scrollable-container"
        style={{
          WebkitOverflowScrolling: "touch",
          scrollBehavior: "smooth",
          maxHeight: "calc(100vh - 100px)",
        }}
      >
        <div className="flex flex-col gap-3 px-4 pt-1">{children}</div>
      </div>
      {displayTiles && (
        <NewChatTiles
          questions={RECOMMENDED_QUESTIONS_TILES}
          onSelect={handleQuestionClick || (() => {})}
        />
      )}
      <div className="sticky bottom-0 left-0 right-0 bg-[#151518]/80 backdrop-blur-sm pb-2 px-4 lg:px-0 pt-3">
        <ChatInput
          inputMessage={inputMessage}
          isGenerating={isGenerating}
          onSendMessage={onSendMessage}
          onInputChange={onInputChange}
          onStopGeneration={onStopGeneration}
          onShareChat={onShareChat}
          isSharedChat={isSharedChat}
          hasMessages={hasMessages}
        />
      </div>
    </div>
  );
}
