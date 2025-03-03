import { type Message } from "../hooks/types";
import { ChatMessage, ToolMessage } from "./Messages";
import { PipelineDisplay } from "./Pipeline";

export function MessageRenderer({ message: msg }: { message: Message }) {
  if (!msg.message) return null;

  console.log("msg", msg);

  if (msg.isToolCall) {
    const toolOutput = {
      name: msg.message.split(": ")[0].replace("Tool ", ""),
      result: msg.message.split(": ").slice(1).join(": "),
    };
    return <ToolMessage toolOutput={toolOutput} />;
  }

  // Check if this is a pipeline message
  const pipelineRegex = /<pipeline>(.*?)<\/pipeline>/gs;
  const pipelineMatches = [...msg.message.matchAll(pipelineRegex)];

  if (pipelineMatches.length > 0) {
    try {
      // Split the message by pipeline tags to maintain order
      const messageParts = msg.message.split(/<pipeline>.*?<\/pipeline>/s);
      const result = [];

      // Process each part and pipeline in order
      for (let i = 0; i < messageParts.length; i++) {
        // Add text part if it's not empty
        if (messageParts[i].trim()) {
          result.push(
            <ChatMessage
              key={`text-${i}`}
              message={messageParts[i].trim()}
              direction={msg.direction}
            />
          );
        }

        // Add pipeline if available
        if (i < pipelineMatches.length) {
          const pipelineContent = pipelineMatches[i][1]
            .trim()
            .replace(/\/\*[\s\S]*?\*\/|\/\/.*/g, ""); // Remove comments

          try {
            const pipeline = JSON.parse(pipelineContent);
            if (pipeline && pipeline.steps) {
              result.push(
                <div
                  key={`pipeline-${i}`}
                  className="my-4 border-b border-purple-500/30 pb-4"
                >
                  <PipelineDisplay pipeline={pipeline} />
                </div>
              );
            }
          } catch (e) {
            console.error(`Failed to parse pipeline JSON #${i + 1}:`, e);
            // If we can't parse the JSON, just render the raw content
            result.push(
              <ChatMessage
                key={`pipeline-error-${i}`}
                message={`<pipeline>${pipelineMatches[i][1]}</pipeline>`}
                direction={msg.direction}
              />
            );
          }
        }
      }

      return <div className="mb-6">{result}</div>;
    } catch (e) {
      console.error("Failed to process pipelines:", e);
      // If we can't process the pipelines, just render as regular message
    }
  }

  return <ChatMessage message={msg.message} direction={msg.direction} />;
}
