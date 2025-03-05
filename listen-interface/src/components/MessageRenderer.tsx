import { ToolResult, ToolResultSchema, type Message } from "../types/message";
import { ChatMessage, ToolMessage } from "./Messages";
import { PipelineDisplay } from "./Pipeline";

export function MessageRenderer({ message: msg }: { message: Message }) {
  if (!msg.message) return null;

  // this is to support previous version of message schema
  if (msg.isToolCall !== undefined && msg.isToolCall) {
    // tool call was tool result in v1, v2 there is a distinction, tool call is
    // passing params, tool result is the "tool output"
    const toolResult = handleLegacyMessage(msg);
    return <ToolMessage toolOutput={toolResult} />;
  }

  if (msg.type === "ToolCall") {
    // no need to display tool calls, just the tool results
    // those are really important for the chat history consistency though!
    if (process.env.NODE_ENV === "production") {
      return;
    }

    const { id, name, params } = JSON.parse(msg.message);

    return (
      <div>
        id: {id}
        <br />
        name: {name}
        <br />
        params: <pre>{JSON.stringify(JSON.parse(params), null, 2)}</pre>
      </div>
    );
  }

  if (msg.type === "ToolResult") {
    const toolOutput = ToolResultSchema.parse(JSON.parse(msg.message));
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

const handleLegacyMessage = (msg: Message): ToolResult => {
  // Get everything after "Tool " prefix
  const afterToolPrefix = msg.message.substring(5); // "Tool ".length = 5

  // Find the position of the first colon which separates id+name/name from result
  const colonIndex = afterToolPrefix.indexOf(": ");

  if (colonIndex === -1) {
    return {
      id: "",
      name: afterToolPrefix,
      result: "",
    };
  }

  // Get the part before the colon (contains name and possibly id)
  const nameAndId = afterToolPrefix.substring(0, colonIndex);
  // Get the result part after the colon
  const result = afterToolPrefix.substring(colonIndex + 2); // Skip ": "

  // Find the last space which separates name from id (if present)
  const lastSpaceIndex = nameAndId.lastIndexOf(" ");

  // Check if this is likely a legacy format message (no ID)
  // In legacy format, nameAndId would be just the tool name without spaces
  // We can detect this by checking if the nameAndId looks like a single word/identifier
  if (lastSpaceIndex === -1 || /^[a-zA-Z0-9_]+$/.test(nameAndId)) {
    return {
      id: "",
      name: nameAndId, // this is just name, legacy format
      result,
    };
  }

  // Parse name and id for new format
  const name = nameAndId.substring(0, lastSpaceIndex);
  const id = nameAndId.substring(lastSpaceIndex + 1);

  return { name, id, result };
};
