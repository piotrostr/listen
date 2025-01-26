import { useState, useEffect } from "react";
import { type Message, useChat } from "../hooks/useChat";
import { ChatContainer } from "./ChatContainer";
import { ChatMessage, ToolMessage } from "./Messages";

const LoadingIndicator = () => (
  <div className="bg-purple-900/20 text-purple-300 rounded px-4 py-2">...</div>
);

const renderMessage = (msg: Message) => {
  if (!msg.message) return null;

  const isToolOutput = msg.message.startsWith("Tool ");
  if (isToolOutput) {
    const toolOutput = {
      name: msg.message.split(": ")[0].replace("Tool ", ""),
      result: msg.message.split(": ").slice(1).join(": "),
    };
    return <ToolMessage key={msg.id} toolOutput={toolOutput} />;
  }

  return (
    <ChatMessage key={msg.id} message={msg.message} direction={msg.direction} />
  );
};

export function Chat() {
  const { messages, isLoading, sendMessage, setMessages } = useChat();
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
        if (inputMessage.trim() === "clear") {
          setMessages([]);
          setInputMessage("");
          return;
        }
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
  }, [inputMessage, sendMessage, setMessages]);

  return (
    <ChatContainer inputMessage={inputMessage}>
      {messages.map(renderMessage)}
      {isLoading && <LoadingIndicator />}
    </ChatContainer>
  );
}
