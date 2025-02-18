import { useEffect, useState } from "react";
import { type Message, useChat } from "../hooks/useChat";
import { ChatContainer } from "./ChatContainer";
import { ChatMessage, ToolMessage } from "./Messages";
import { PipelineDisplay } from "./Pipeline";

const IS_DISABLED = true;

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

  // Check if this is a pipeline message
  const pipelineRegex = /<pipeline>(.*?)<\/pipeline>/s;
  const pipelineMatch = msg.message.match(pipelineRegex);

  if (pipelineMatch) {
    try {
      const pipelineContent = pipelineMatch[1]
        .trim()
        .replace(/\/\*[\s\S]*?\*\/|\/\/.*/g, ""); // Remove comments
      console.log(JSON.stringify(pipelineContent));
      const pipeline = JSON.parse(pipelineContent);
      if (pipeline && pipeline.steps) {
        return (
          <div key={msg.id} className="mb-6 border-b border-purple-500/30 pb-6">
            <PipelineDisplay pipeline={pipeline} />
          </div>
        );
      }
    } catch (e) {
      console.error("Failed to parse pipeline JSON:", e);
      // If we can't parse the JSON, just render as regular message
      return (
        <ChatMessage
          key={msg.id}
          message={msg.message}
          direction={msg.direction}
        />
      );
    }
  }

  return (
    <ChatMessage key={msg.id} message={msg.message} direction={msg.direction} />
  );
};

export function Chat() {
  const { messages, isLoading, sendMessage, setMessages } = useChat();
  const [inputMessage, setInputMessage] = useState("");

  useEffect(() => {
    if (IS_DISABLED) return;

    const handleKeyPress = async (e: KeyboardEvent) => {
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      // Handle paste with cmd/ctrl + v
      if ((e.metaKey || e.ctrlKey) && e.key === "v") {
        try {
          const text = await navigator.clipboard.readText();
          setInputMessage((prev) => prev + text);
          e.preventDefault();
          return;
        } catch (err) {
          console.error("Failed to read clipboard:", err);
        }
      }

      // Handle cmd/ctrl + backspace to clear entire text
      if ((e.metaKey || e.ctrlKey) && e.key === "Backspace") {
        setInputMessage("");
        e.preventDefault();
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

  if (IS_DISABLED) {
    return (
      <ChatContainer inputMessage="">
        <div className="text-purple-300 px-4 py-2">
          Chat is currently disabled
        </div>
      </ChatContainer>
    );
  }

  return (
    <ChatContainer inputMessage={inputMessage}>
      {messages.map(renderMessage)}
      {isLoading && <LoadingIndicator />}
    </ChatContainer>
  );
}
