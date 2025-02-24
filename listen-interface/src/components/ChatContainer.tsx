import { ReactNode, useState } from "react";
import { FiSend, FiStopCircle } from "react-icons/fi";

interface ChatContainerProps {
  inputMessage: string;
  isGenerating?: boolean;
  onSendMessage?: (message: string) => void;
  onStopGeneration?: () => void;
  children: ReactNode;
}

export function ChatContainer({
  inputMessage,
  isGenerating = false,
  onSendMessage = () => {},
  onStopGeneration = () => {},
  children,
}: ChatContainerProps) {
  return (
    <div className="flex flex-col h-full max-h-[100vh]">
      <div className="flex-grow overflow-y-auto pb-4 space-y-4 scrollbar-thin scrollbar-thumb-purple-500/30 scrollbar-track-transparent">
        <div className="p-4">{children}</div>
      </div>
      <div className="sticky bottom-0 left-0 right-0 p-4 bg-black/80 backdrop-blur-sm border-t border-purple-500/20">
        <ChatInput
          inputMessage={inputMessage}
          isGenerating={isGenerating}
          onSendMessage={onSendMessage}
          onStopGeneration={onStopGeneration}
        />
      </div>
    </div>
  );
}

interface ChatInputProps {
  inputMessage: string;
  isGenerating: boolean;
  onSendMessage: (message: string) => void;
  onStopGeneration: () => void;
}

export function ChatInput({
  inputMessage,
  isGenerating,
  onSendMessage,
  onStopGeneration,
}: ChatInputProps) {
  const [isFocused, setIsFocused] = useState(false);

  const handleSend = () => {
    if (inputMessage.trim()) {
      onSendMessage(inputMessage);
    }
  };

  return (
    <div
      className={`min-h-12 border-2 ${isFocused ? "border-purple-500/60" : "border-purple-500/30"} 
                 rounded-lg bg-black/40 backdrop-blur-sm px-3 py-3 flex items-center`}
      onClick={() => setIsFocused(true)}
      onBlur={() => setIsFocused(false)}
      tabIndex={0}
    >
      <div className="flex-grow">
        <span className="text-white whitespace-pre-wrap break-words">
          {inputMessage}
        </span>
        {!isGenerating && (
          <span className="w-2 h-5 bg-white terminal-blink ml-[1px] inline-block align-middle" />
        )}
      </div>

      <div className="flex-shrink-0 ml-2">
        {isGenerating ? (
          <button
            onClick={(e) => {
              e.stopPropagation();
              onStopGeneration();
            }}
            className="p-2 rounded-full bg-red-500/20 hover:bg-red-500/40 transition-colors"
            aria-label="Stop generating"
          >
            <FiStopCircle className="text-red-400" size={18} />
          </button>
        ) : (
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleSend();
            }}
            disabled={!inputMessage.trim()}
            className={`p-2 rounded-full ${
              inputMessage.trim()
                ? "bg-purple-500/20 hover:bg-purple-500/40 text-purple-300"
                : "bg-gray-500/10 text-gray-500"
            } transition-colors`}
            aria-label="Send message"
          >
            <FiSend size={18} />
          </button>
        )}
      </div>
    </div>
  );
}
