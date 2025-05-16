import { usePrivy } from "@privy-io/react-auth";
import * as Tooltip from "@radix-ui/react-tooltip";
import { Link } from "@tanstack/react-router";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { FiPlus, FiSend, FiShare2, FiStopCircle } from "react-icons/fi";
import { useKeyboard } from "../contexts/KeyboardContext";
import { useMobile } from "../contexts/MobileContext";
import { usePrivyWallets } from "../hooks/usePrivyWallet";

interface ChatInputProps {
  inputMessage: string;
  isGenerating: boolean;
  onSendMessage: (message: string) => void;
  onInputChange: (message: string) => void;
  onStopGeneration: () => void;
  onShareChat?: () => void;
  isSharedChat?: boolean;
  hasMessages?: boolean;
}

export function ChatInput({
  inputMessage,
  isGenerating,
  onSendMessage,
  onInputChange,
  onStopGeneration,
  onShareChat,
  isSharedChat = false,
  hasMessages = false,
}: ChatInputProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const { isMobile } = useMobile();

  const { user } = usePrivy();

  const { setIsKeyboardOpen } = useKeyboard();

  // const { data: listenBalance } = useListenBalance();

  // Function to auto-resize the textarea
  const autoResizeTextarea = () => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    // Reset height to measure the scrollHeight correctly
    textarea.style.height = "auto";

    // Calculate line height (approximately 20px per line)
    const lineHeight = 20;
    const maxLines = 4;

    // Get content height and limit to 4 lines
    const contentHeight = textarea.scrollHeight;
    const maxHeight = lineHeight * maxLines;

    // Set the height with a minimum of one line
    textarea.style.height =
      Math.min(Math.max(lineHeight, contentHeight), maxHeight) + "px";
  };

  // Auto-resize when input changes
  useEffect(() => {
    autoResizeTextarea();
  }, [inputMessage]);

  const handleSend = () => {
    if (inputMessage.trim()) {
      onSendMessage(inputMessage);
    }
  };

  const { data: wallets } = usePrivyWallets();

  const walletsReady =
    wallets?.evmWallet !== undefined && wallets?.solanaWallet !== undefined;

  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-2 pl-3">
      {/* Plus button outside of input */}
      {!hasMessages ? (
        <button className="p-2 rounded-full bg-gray-600/20 hover:bg-gray-600/30 transition-colors text-gray-400">
          <FiPlus size={18} />
        </button>
      ) : (
        <Link
          to="/"
          search={{ new: true }}
          className="p-2 rounded-full bg-gray-600/20 hover:bg-gray-600/30 transition-colors text-gray-400"
          title="New Chat"
        >
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
        </Link>
      )}

      {/* Main input container */}
      <div
        className={`flex flex-1 items-center rounded-3xl overflow-hidden border-[#2D2D2D] bg-[#151518]/40 backdrop-blur-sm border`}
      >
        <div className="flex-1 flex items-center px-4 py-3">
          <textarea
            ref={textareaRef}
            value={inputMessage}
            onChange={(e) => onInputChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                if (isGenerating) {
                  onStopGeneration();
                } else {
                  handleSend();
                }
              }
              if (e.key === "Escape" && isGenerating) {
                e.preventDefault();
                onStopGeneration();
              }
            }}
            rows={1}
            className="w-full bg-transparent text-white outline-none resize-none chat-input overflow-y-auto scrollbar-hide"
            placeholder={t("chat.placeholder")}
            style={{
              minHeight: "20px",
              maxHeight: "80px",
            }}
            onFocus={() => setIsKeyboardOpen(true)}
            onBlur={() => setIsKeyboardOpen(false)}
          />
        </div>

        <div className="flex items-center px-2 py-2 gap-2">
          {isGenerating ? (
            <div className="flex items-center gap-2">
              {!isSharedChat && onShareChat && (
                <button
                  onClick={onShareChat}
                  className="p-2 rounded-full bg-blue-500/20 hover:bg-blue-500/40 text-blue-300 transition-colors"
                  title="Share this chat"
                >
                  <FiShare2 size={18} />
                </button>
              )}
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
            </div>
          ) : (
            <div className="flex items-center gap-2">
              {!isSharedChat && onShareChat && (
                <button
                  onClick={onShareChat}
                  className="p-2 rounded-full bg-blue-500/20 hover:bg-blue-500/40 text-blue-300 transition-colors"
                  title="Share this chat"
                >
                  <FiShare2 size={18} />
                </button>
              )}
              <Tooltip.Provider>
                <Tooltip.Root>
                  <Tooltip.Trigger asChild>
                    <div className="relative">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleSend();
                        }}
                        disabled={
                          !inputMessage.trim() || !walletsReady || !user
                        }
                        className={`p-2 rounded-full ${
                          inputMessage.trim() && walletsReady && user
                            ? "bg-[#FB2671]/20 hover:bg-[#FB2671]/40 text-[#FB2671]"
                            : "bg-gray-500/10 text-gray-500"
                        } transition-colors`}
                        aria-label="Send message"
                      >
                        <FiSend size={18} />
                      </button>
                    </div>
                  </Tooltip.Trigger>
                </Tooltip.Root>
              </Tooltip.Provider>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
