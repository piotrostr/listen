import { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { useSettingsStore } from "../store/settingsStore";
import { useSuggestStore } from "../store/suggestStore";
import { ChatInput } from "./ChatInput";
import { NewChatTiles } from "./NewChatTiles";
import { SuggestionTiles } from "./SuggestionTiles";

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
  chatId?: string;
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
  chatId,
}: ChatContainerProps) {
  const { t } = useTranslation();
  const { getSuggestions, lastMessageHadSpecialTags } = useSuggestStore();
  const { displaySuggestions } = useSettingsStore();
  const suggestions = chatId ? getSuggestions(chatId) : [];

  const RECOMMENDED_QUESTIONS_TILES = [
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
      <div className="mt-auto sticky bottom-0 left-0 right-0 bg-[#151518]/80 backdrop-blur-sm pb-2 pt-3">
        {!isGenerating &&
          handleQuestionClick &&
          displaySuggestions &&
          suggestions.length > 0 &&
          !displayTiles &&
          !lastMessageHadSpecialTags && (
            <div className="absolute bottom-full w-full">
              <SuggestionTiles
                suggestions={suggestions}
                handleQuestionClick={handleQuestionClick}
              />
            </div>
          )}
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
