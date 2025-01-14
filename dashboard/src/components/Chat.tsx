import { useState } from "react";
import {
  MainContainer,
  ChatContainer,
  MessageList,
  Message,
  MessageInput,
  TypingIndicator,
} from "@chatscope/chat-ui-kit-react";
import { useChat } from "../hooks/useChat";

export function Chat() {
  const { messages, isLoading, sendMessage } = useChat();
  const [inputMessage, setInputMessage] = useState("");

  const handleSend = (message: string) => {
    sendMessage(message);
    setInputMessage("");
  };

  return (
    <div className="h-[600px] w-full max-w-4xl mx-auto px-4 font-mono">
      <MainContainer className="border-2 border-purple-500/30 rounded-lg overflow-hidden bg-black/40 backdrop-blur-sm">
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
          <MessageInput
            placeholder=">> Enter your message..."
            value={inputMessage}
            onChange={(val) => setInputMessage(val)}
            onSend={handleSend}
            attachButton={false}
            className="border-t border-purple-500/30 bg-black/40 text-purple-300 p-2"
            style={
              {
                "--message-input-bg": "transparent",
                "--message-input-border": "none",
                "--message-input-color": "inherit",
                "--message-input-placeholder-color": "#b794f4",
              } as React.CSSProperties
            }
          />
        </ChatContainer>
      </MainContainer>
    </div>
  );
}
