import { type Message } from "../hooks/types";
import { ChatMessage, ToolMessage } from "./Messages";
import { PipelineDisplay } from "./Pipeline";

export function MessageRenderer({ message: msg }: { message: Message }) {
  if (!msg.message) return null;

  if (msg.isToolCall) {
    const toolOutput = {
      name: msg.message.split(": ")[0].replace("Tool ", ""),
      result: msg.message.split(": ").slice(1).join(": "),
    };
    return <ToolMessage toolOutput={toolOutput} />;
  }

  // Check if this is a pipeline message
  const pipelineRegex = /<pipeline>(.*?)<\/pipeline>/s;
  const pipelineMatch = msg.message.match(pipelineRegex);

  if (pipelineMatch) {
    try {
      const pipelineContent = pipelineMatch[1]
        .trim()
        .replace(/\/\*[\s\S]*?\*\/|\/\/.*/g, ""); // Remove comments

      const pipeline = JSON.parse(pipelineContent);
      if (pipeline && pipeline.steps) {
        return (
          <div className="mb-6 border-b border-purple-500/30 pb-6">
            <ChatMessage
              message={msg.message.replace(pipelineRegex, "").trim()}
              direction={msg.direction}
            />
            <div className="mt-4">
              <PipelineDisplay pipeline={pipeline} />
            </div>
          </div>
        );
      }
    } catch (e) {
      console.error("Failed to parse pipeline JSON:", e);
      // If we can't parse the JSON, just render as regular message
    }
  }

  return <ChatMessage message={msg.message} direction={msg.direction} />;
}
