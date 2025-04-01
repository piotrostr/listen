import { usePrivy } from "@privy-io/react-auth";
import * as Tooltip from "@radix-ui/react-tooltip";
import { Link } from "@tanstack/react-router";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { FiPlus, FiSend, FiShare2, FiStopCircle } from "react-icons/fi";
import { IoSwapHorizontal } from "react-icons/io5";
import { LuTelescope } from "react-icons/lu";
import { usePrivyWallets } from "../hooks/usePrivyWallet";
import { useSettingsStore } from "../store/settingsStore";

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

  const {
    researchEnabled,
    tradingEnabled,
    setResearchEnabled,
    setTradingEnabled,
    modelType,
  } = useSettingsStore();

  const { user } = usePrivy();

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

  // Toggle the research mode
  const toggleResearch = () => {
    setTradingEnabled(false);
    setResearchEnabled(!researchEnabled);
  };

  // Toggle the trading mode
  const toggleTrading = () => {
    setResearchEnabled(false);
    setTradingEnabled(!tradingEnabled);
  };

  const sendDisabled = modelType === "claude" && researchEnabled;

  return (
    <div className="flex flex-col rounded-3xl overflow-hidden border border-[#2D2D2D] bg-[#151518]/40 backdrop-blur-sm mb-2">
      {/* Textarea row */}
      <div className="flex items-center px-4 py-3">
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
            maxHeight: "80px", // Approximately 4 lines
          }}
        />
      </div>

      {/* Button row */}
      <div className="flex items-center px-2 py-2 gap-2">
        {/* Plus button (noop) */}
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

        {/* Search button */}
        <button
          onClick={toggleTrading}
          className={`flex items-center gap-2 px-4 py-2 rounded-full ${
            tradingEnabled
              ? "bg-blue-600/20 text-blue-400"
              : "bg-gray-600/20 text-gray-400"
          } hover:bg-gray-600/30 transition-colors text-sm hidden`} // tmp hidden
        >
          <IoSwapHorizontal size={18} />
          <span>{t("chat.trading")}</span>
        </button>

        {/* Research Feature */}
        <button
          onClick={toggleResearch}
          className={`flex items-center gap-2 px-4 py-2 rounded-full ${
            researchEnabled
              ? "bg-blue-600/20 text-blue-400"
              : "bg-gray-600/20 text-gray-400"
          } hover:bg-gray-600/30 transition-colors text-sm`}
        >
          <LuTelescope size={18} />
          <span>{t("chat.research")}</span>
        </button>

        {/* Arrow up button on the far right */}
        {isGenerating ? (
          <div className="flex items-center gap-2 ml-auto">
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
          <div className="flex items-center gap-2 ml-auto">
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
                        !inputMessage.trim() ||
                        !walletsReady ||
                        !user ||
                        sendDisabled
                      }
                      className={`p-2 rounded-full ${
                        inputMessage.trim() &&
                        walletsReady &&
                        user &&
                        !sendDisabled
                          ? "bg-[#FB2671]/20 hover:bg-[#FB2671]/40 text-[#FB2671]"
                          : "bg-gray-500/10 text-gray-500"
                      } transition-colors`}
                      aria-label="Send message"
                    >
                      <FiSend size={18} />
                    </button>
                  </div>
                </Tooltip.Trigger>
                {sendDisabled && (
                  <Tooltip.Portal>
                    <Tooltip.Content
                      className="rounded-md bg-[#2d2d2d] px-4 py-2 text-sm text-white"
                      sideOffset={5}
                    >
                      {t("chat.research_disabled")}
                      <Tooltip.Arrow className="fill-[#2d2d2d]" />
                    </Tooltip.Content>
                  </Tooltip.Portal>
                )}
              </Tooltip.Root>
            </Tooltip.Provider>
          </div>
        )}
      </div>
    </div>
  );
}
