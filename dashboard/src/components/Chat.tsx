import { useState } from "react";
import "@chatscope/chat-ui-kit-styles/dist/default/styles.min.css";
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
    <div className="h-[600px] w-full">
      <MainContainer>
        <ChatContainer>
          <MessageList typingIndicator={isLoading ? <TypingIndicator /> : null}>
            {messages.map((msg) => (
              <Message
                key={msg.id}
                model={{
                  message: msg.message,
                  direction: msg.direction,
                  position: "single",
                }}
              />
            ))}
          </MessageList>
          <MessageInput
            placeholder="Type message here..."
            value={inputMessage}
            onChange={(val) => setInputMessage(val)}
            onSend={handleSend}
            attachButton={false}
          />
        </ChatContainer>
      </MainContainer>
    </div>
  );
}
