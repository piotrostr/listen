import { useState, useEffect } from "react";
import {
  MainContainer,
  ChatContainer,
  MessageList,
  Message,
  TypingIndicator,
} from "@chatscope/chat-ui-kit-react";
import { useChat } from "../hooks/useChat";

export function Chat() {
  const { messages, isLoading, sendMessage } = useChat();
  const [inputMessage, setInputMessage] = useState("");

  useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      if (e.key === "Enter") {
        if (inputMessage.trim()) {
          sendMessage(inputMessage);
          setInputMessage("");
        }
      } else if (e.key === "Backspace") {
        setInputMessage((prev) => prev.slice(0, -1));
      } else if (e.key.length === 1) {
        setInputMessage((prev) => prev + e.key);
      }
    };

    window.addEventListener("keydown", handleKeyPress);
    return () => window.removeEventListener("keydown", handleKeyPress);
  }, [inputMessage, sendMessage]);

  return (
    <div className="flex flex-col gap-4 h-[600px] w-full max-w-4xl mx-auto px-4 font-mono">
      {/* Chat Container */}
      <div className="flex-1">
        <MainContainer className="h-full border-2 border-purple-500/30 rounded-lg overflow-hidden bg-black/40 backdrop-blur-sm">
          <ChatContainer className="px-2">
            <MessageList
              className="py-4 scrollbar-thin scrollbar-thumb-purple-500/30 scrollbar-track-transparent"
              typingIndicator={
                isLoading ? (
                  <TypingIndicator
                    className="bg-purple-900/20 text-purple-300 rounded px-4 py-2"
                    content="..."
                  />
                ) : null
              }
            >
              {messages.map((msg) => (
                <Message
                  key={msg.id}
                  model={{
                    message: msg.message,
                    direction: msg.direction,
                    position: "single",
                  }}
                  className={`
                    ${
                      msg.direction === "incoming"
                        ? "bg-blue-900/20 text-blue-300"
                        : "bg-purple-900/20 text-purple-300"
                    }
                    rounded-lg px-4 py-2 my-2 backdrop-blur-sm
                    border border-opacity-20
                    ${
                      msg.direction === "incoming"
                        ? "border-blue-500"
                        : "border-purple-500"
                    }
                  `}
                />
              ))}
            </MessageList>
          </ChatContainer>
        </MainContainer>
      </div>

      {/* Terminal Input Display */}
      <div className="h-12 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3">
        <span className="text-white whitespace-pre">{inputMessage}</span>
        <span className="w-2 h-5 bg-white terminal-blink ml-[1px]" />
      </div>
    </div>
  );
}
