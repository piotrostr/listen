import {
  NestedAgentOutputSchema,
  StreamResponse,
  StreamResponseSchema,
  ToolCallSchema,
  ToolResultSchema,
} from "./types/message";

export const renderAgentOutput = (
  output: string,
  messageOnly: boolean
): string => {
  const streamResponses = parseAgentOutput(output);
  return renderAgentOutputString(streamResponses, messageOnly);
};

/**
 * Parses a string of concatenated JSON objects into an array of StreamResponse objects
 * Each JSON object is enclosed in <content>{...}</content> tags
 *
 * @param output - String containing concatenated JSON objects from agent output
 * @returns Array of parsed and validated StreamResponse objects
 */
export function parseAgentOutput(output: string): StreamResponse[] {
  if (!output || output.trim() === "") {
    return [];
  }

  const results: StreamResponse[] = [];
  // Use a non-greedy match and capture everything between tags
  const contentRegex = /<content>([\s\S]*?)<\/content>/g;

  let match;
  while ((match = contentRegex.exec(output)) !== null) {
    try {
      // First decode base64 to string
      const decodedStr = Buffer.from(match[1], "base64").toString();

      const jsonStr = decodedStr
        .replace(/\n/g, "\\n") // Properly escape newlines
        .replace(/\r/g, "\\r"); // Properly escape carriage returns
      const parsed = JSON.parse(jsonStr);
      const validated = StreamResponseSchema.parse(parsed);
      if (validated.type === "NestedAgentOutput") {
        const safeParsed = NestedAgentOutputSchema.safeParse(validated.content);
        if (safeParsed.success) {
          let content = safeParsed.data.content;
          // Create a new regex instance for nested content
          const nestedContentRegex = /<content>([\s\S]*?)<\/content>/g;
          const subres = [];
          let nestedMatch;
          let lastIndex = 0;

          while ((nestedMatch = nestedContentRegex.exec(content)) !== null) {
            try {
              const decodedStr = Buffer.from(
                nestedMatch[1],
                "base64"
              ).toString();
              const jsonStr = decodedStr
                .replace(/\n/g, "\\n")
                .replace(/\r/g, "\\r");
              const parsed = StreamResponseSchema.parse(JSON.parse(jsonStr));
              subres.push(parsed);
              lastIndex = nestedContentRegex.lastIndex;
            } catch (error) {
              console.error("Error parsing nested content:", error);
            }
          }

          if (subres.length > 0) {
            results.push(...subres);
          }
          continue; // Skip adding the nested agent output itself
        }
      }
      // in the nested output, the tool result it not to be included.
      // it is already the body and will arrive as a separate, non-nested message
      if (validated.type === "ToolResult") {
        continue;
      }
      results.push(validated);
    } catch (error) {
      console.error("Error parsing content:", error, "Raw string:", match[1]);
    }
  }

  return results;
}

export function renderAgentOutputString(
  streamResponses: StreamResponse[],
  messageOnly: boolean
): string {
  let output = "";
  let accumulatedMessage = "";

  if (messageOnly) {
    for (const streamResponse of streamResponses) {
      if (streamResponse.type === "Message") {
        output += streamResponse.content;
      }
    }
    return output;
  }

  for (const streamResponse of streamResponses) {
    switch (streamResponse.type) {
      case "Message":
        // Accumulate message content
        accumulatedMessage += streamResponse.content;
        break;
      case "ToolCall":
        // First render any accumulated message
        if (accumulatedMessage) {
          output += `<p>${accumulatedMessage}</p>`;
          accumulatedMessage = "";
        }
        let call = ToolCallSchema.parse(streamResponse.content);
        output += `<p>${call.name} with ${Object.entries(
          JSON.parse(call.params)
        )
          .map(([key, value]) => `<li>${key}: ${value}</li>`)
          .join("")}</p>`;
        break;
      case "ToolResult":
        // First render any accumulated message
        if (accumulatedMessage) {
          output += `<p>${accumulatedMessage}</p>`;
          accumulatedMessage = "";
        }
        let result = ToolResultSchema.parse(streamResponse.content);
        output += `<p>${
          result.result.includes("{") || result.result.includes("ToolCallError")
            ? result.result
            : JSON.parse(result.result)
        }</p>`;
        break;
    }
  }

  // Don't forget to render any remaining accumulated message
  if (accumulatedMessage) {
    output += `<p>${accumulatedMessage}</p>`;
  }

  return output;
}
