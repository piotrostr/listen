import { useSearch } from "@tanstack/react-router";
import { useEffect, useRef } from "react";
import { useChat } from "../hooks/useChat";
import { useKeyboardInput } from "../hooks/useKeyboardInput";
import { ChatContainer } from "./ChatContainer";
import { MessageRenderer } from "./MessageRenderer";

const IS_DISABLED = process.env.NODE_ENV === "production";

const LoadingIndicator = () => (
  <div className="bg-purple-900/20 text-purple-300 rounded px-4 py-2">...</div>
);

// Recommended conversation starters
const RECOMMENDED_QUESTIONS = [
  {
    question: "What actions can you perform for me?",
    enabled: true,
  },
  {
    question: "How do pipelines work and what pipelines can you create for me?",
    enabled: true,
  },
  {
    question: "What chains are supported?",
    enabled: true,
  },
  {
    question:
      "What tokens have received largest inflows/outflows in the past days?",
    enabled: true,
  },
];

export function Chat() {
  const { messages, isLoading, sendMessage, setMessages, stopGeneration } =
    useChat();
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const { new: isNewChat } = useSearch({ from: "/chat" });

  const { inputMessage, submitMessage, setInputMessage } = useKeyboardInput({
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

  // Focus the input field when creating a new chat
  useEffect(() => {
    if (isNewChat) {
      // This will focus the input field in ChatContainer
      // You might need to expose a method from ChatContainer to focus the input
      const inputElement = document.querySelector(".chat-input");
      if (inputElement instanceof HTMLTextAreaElement) {
        inputElement.focus();
      }
    }
  }, [isNewChat]);

  const handleQuestionClick = (question: string) => {
    setInputMessage(question);
    submitMessage();
  };

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
      {messages.length === 0 && (
        <div className="flex flex-col items-center justify-center py-12 px-4">
          <h2 className="text-xl font-medium text-white mb-6">
            Start a conversation
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 w-full max-w-3xl">
            {RECOMMENDED_QUESTIONS.map((question, index) => (
              <button
                key={index}
                disabled={!question.enabled}
                onClick={() => handleQuestionClick(question.question)}
                className="bg-purple-900/30 hover:bg-purple-800/40 text-left p-4 rounded-lg border border-purple-500/30 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <p className="text-white">{question.question}</p>
              </button>
            ))}
          </div>
        </div>
      )}
      {messages.map((message) => (
        <MessageRenderer key={message.id} message={message} />
      ))}
      {isLoading && <LoadingIndicator />}
      <div ref={messagesEndRef} />
    </ChatContainer>
  );
}
