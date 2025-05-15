import { ReactNode } from "react";
import { useSettingsStore } from "../store/settingsStore";
import { useSuggestStore } from "../store/suggestStore";
import { ChatInput } from "./ChatInput";
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
  const { getSuggestions, lastMessageHadSpecialTags } = useSuggestStore();
  const { displaySuggestions } = useSettingsStore();
  const suggestions = chatId ? getSuggestions(chatId) : [];

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
      <div className="mt-auto sticky bottom-0 left-0 right-0 bg-[#151518]/80 backdrop-blur-sm pb-1 pt-3">
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
