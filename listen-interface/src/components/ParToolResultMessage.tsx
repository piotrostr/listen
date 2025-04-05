import React, { useMemo } from "react";
import {
  Message,
  ParToolCallSchema,
  ParToolResult,
  RigToolCall,
  SimpleToolResult,
} from "../types/message";
import { ToolMessage } from "./ToolMessage";

interface ParToolResultMessageProps {
  parToolResult: ParToolResult;
  messages: Message[];
  currentMessage: Message;
}

export const ParToolResultMessage: React.FC<ParToolResultMessageProps> = ({
  parToolResult,
  messages,
  currentMessage,
}) => {
  // Find the corresponding ParToolCall message that preceded this result message
  const matchingParToolCallData = useMemo(() => {
    const currentIndex = messages.findIndex((m) => m.id === currentMessage.id);
    if (currentIndex === -1) return null;

    for (let i = currentIndex - 1; i >= 0; i--) {
      const message = messages[i];
      if (message.type === "ParToolCall") {
        try {
          const parsedParToolCall = ParToolCallSchema.parse(
            JSON.parse(message.message)
          );
          // Basic check: does the number of calls match the number of results?
          // A more robust check might involve specific IDs if available and necessary.
          if (
            Object.keys(parsedParToolCall.tool_calls).length ===
            parToolResult.tool_results.length
          ) {
            return parsedParToolCall.tool_calls;
          }
        } catch (e) {
          console.error("Failed to parse ParToolCall while searching:", e);
        }
      }
      // Stop searching if we hit an outgoing message or another tool result type,
      // assuming calls and results appear sequentially.
      if (
        message.direction === "outgoing" ||
        message.type === "ToolResult" ||
        message.type === "ParToolResult"
      ) {
        break;
      }
    }
    console.warn(
      "Could not find matching ParToolCall for ParToolResult:",
      currentMessage.id
    );
    return null;
  }, [messages, currentMessage.id, parToolResult.tool_results.length]);

  if (!matchingParToolCallData) {
    // Decide how to render if the matching call isn't found.
    // Maybe render results without call context or show an error/warning.
    return (
      <div className="text-orange-400 p-2 text-sm">
        Warning: Could not link parallel results to their calls. Displaying raw
        results:
        {parToolResult.tool_results.map((result) => (
          <pre key={result.id || result.index}>
            {JSON.stringify(result, null, 2)}
          </pre>
        ))}
      </div>
    );
  }

  // Match results to their calls using the ID
  const matchedResults = useMemo(() => {
    console.log(JSON.stringify(parToolResult, null, 2));
    console.log(JSON.stringify(matchingParToolCallData, null, 2));
    return parToolResult.tool_results
      .map((result) => {
        const matchingCall = matchingParToolCallData[result.index];
        if (matchingCall) {
          return { result, call: matchingCall };
        }
        console.warn(
          `Could not find matching call for result ID: ${result.id}`
        );
        return null; // Handle cases where a result might not have a matching call found
      })
      .filter(Boolean) as { result: SimpleToolResult; call: RigToolCall }[]; // Filter out nulls and assert type
  }, [parToolResult.tool_results, matchingParToolCallData]);

  // Render each matched result using ToolMessage
  return (
    <div className="flex flex-col gap-2">
      {matchedResults.map(({ result, call }) => (
        <ToolMessage
          key={result.id || result.index} // Use index as fallback key if id isn't guaranteed
          // Adapt SimpleToolResult to the ToolResult shape expected by ToolMessage
          toolOutput={{
            id: result.id,
            name: result.name,
            result: result.result,
          }}
          // Pass the corresponding RigToolCall data as a new prop
          toolCallData={call}
          messages={messages} // Keep passing messages for now, might remove later
          currentMessage={currentMessage} // Keep passing currentMessage for now
        />
      ))}
    </div>
  );
};
