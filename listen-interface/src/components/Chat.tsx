import { useEffect, useRef } from "react";
import { useChat } from "../hooks/useChat";
import { useKeyboardInput } from "../hooks/useKeyboardInput";
import { ChatContainer } from "./ChatContainer";
import { MessageRenderer } from "./MessageRenderer";

const IS_DISABLED = process.env.NODE_ENV === "production";

const LoadingIndicator = () => (
  <div className="bg-purple-900/20 text-purple-300 rounded px-4 py-2">...</div>
);

export function Chat() {
  const { messages, isLoading, sendMessage, setMessages, stopGeneration } =
    useChat();
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const { inputMessage, submitMessage } = useKeyboardInput({
    onSubmit: sendMessage,
    onClear: () => setMessages([]),
    onStopGeneration: stopGeneration,
    isGenerating: isLoading,
    isDisabled: IS_DISABLED,
  });

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

  if (IS_DISABLED) {
    return (
      <ChatContainer inputMessage="" isGenerating={false}>
        <div className="text-purple-300 px-4 py-2">disabled</div>
      </ChatContainer>
    );
  }

  return (
    <ChatContainer
      inputMessage={inputMessage}
      isGenerating={isLoading}
      onSendMessage={submitMessage}
      onStopGeneration={stopGeneration}
    >
      {messages.map((message) => (
        <MessageRenderer key={message.id} message={message} />
      ))}
      {isLoading && <LoadingIndicator />}
      <div ref={messagesEndRef} />
    </ChatContainer>
  );
}
