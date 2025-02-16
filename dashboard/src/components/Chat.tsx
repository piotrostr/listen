import { useEffect, useState } from "react";
import { type Message, useChat } from "../hooks/useChat";
import { ChatContainer } from "./ChatContainer";
import { ChatMessage, ToolMessage } from "./Messages";
import { PipelineDisplay } from "./Pipeline";

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
  if (msg.message.includes("<pipeline>")) {
    try {
      const pipelineContent = msg.message
        .split("<pipeline>")[1]
        ?.split("</pipeline>")[0];

      if (!pipelineContent) {
        return (
          <ChatMessage
            key={msg.id}
            message={msg.message}
            direction={msg.direction}
          />
        );
      }

      // Try to parse the pipeline content
      try {
        // First try to parse it as a complete pipeline
        const pipeline = JSON.parse(pipelineContent);
        if (Array.isArray(pipeline.steps)) {
          return (
            <div
              key={msg.id}
              className="mb-6 border-b border-purple-500/30 pb-6"
            >
              <PipelineDisplay pipeline={pipeline} />
            </div>
          );
        }
      } catch (e) {
        // If parsing fails, try to extract any complete steps
        const stepsStart = pipelineContent.indexOf('"steps"');
        if (stepsStart !== -1) {
          const stepsArray = pipelineContent.slice(stepsStart);
          // Try to construct a valid JSON
          const partialPipeline = `{"${stepsArray}`;
          try {
            const pipeline = JSON.parse(
              partialPipeline.endsWith("}")
                ? partialPipeline
                : partialPipeline + "}"
            );
            if (Array.isArray(pipeline.steps)) {
              return (
                <div
                  key={msg.id}
                  className="mb-6 border-b border-purple-500/30 pb-6"
                >
                  <PipelineDisplay pipeline={pipeline} />
                </div>
              );
            }
          } catch (e) {
            // If we still can't parse it, just show the message
            console.log("Partial pipeline content:", pipelineContent);
          }
        }
      }
    } catch (error) {
      console.error("Failed to parse pipeline:", error);
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
