import { useEffect, useState } from "react";
import { type Message, useChat } from "../hooks/useChat";
import { Pipeline } from "../types/pipeline";
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
      // Try to parse any complete steps from the partial JSON
      const pipelineContent = msg.message
        .split("<pipeline>")[1]
        ?.split("</pipeline>")[0];
      if (pipelineContent) {
        // Handle partial JSON by wrapping incomplete content
        let jsonToParseStr = pipelineContent;
        if (!pipelineContent.trim().startsWith("{")) {
          jsonToParseStr = `{${pipelineContent}`;
        }
        if (!pipelineContent.trim().endsWith("}")) {
          jsonToParseStr = `${jsonToParseStr}}`;
        }

        try {
          const partialPipeline = JSON.parse(jsonToParseStr);
          const pipeline: Pipeline = {
            steps: Array.isArray(partialPipeline.steps)
              ? partialPipeline.steps
              : [],
          };

          return (
            <div
              key={msg.id}
              className="mb-6 border-b border-purple-500/30 pb-6"
            >
              <PipelineDisplay pipeline={pipeline} />
            </div>
          );
        } catch (e) {
          console.log(pipelineContent);
          // If we can't parse the JSON yet, try to extract any complete steps
          const stepsMatch = pipelineContent.match(/"steps"\s*:\s*\[(.*?)\]/s);
          if (stepsMatch) {
            const stepsContent = stepsMatch[1];
            const steps = [];
            let bracketCount = 0;
            let currentStep = "";

            // Parse steps one by one
            for (const char of stepsContent) {
              currentStep += char;
              if (char === "{") bracketCount++;
              if (char === "}") bracketCount--;

              if (bracketCount === 0 && currentStep.trim()) {
                try {
                  const step = JSON.parse(currentStep);
                  steps.push(step);
                  currentStep = "";
                } catch (e) {
                  // Skip incomplete steps
                }
              }
            }

            if (steps.length > 0) {
              const pipeline: Pipeline = { steps };
              return (
                <div
                  key={msg.id}
                  className="mb-6 border-b border-purple-500/30 pb-6"
                >
                  <PipelineDisplay pipeline={pipeline} />
                </div>
              );
            }
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

  return (
    <ChatContainer inputMessage={inputMessage}>
      {messages.map(renderMessage)}
      {isLoading && <LoadingIndicator />}
    </ChatContainer>
  );
}
