import {
  StreamResponse,
  StreamResponseSchema,
  ToolCallSchema,
  ToolResultSchema,
} from "./types/message";

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
      results.push(validated);
    } catch (error) {
      console.error("Error parsing content:", error, "Raw string:", match[1]);
    }
  }

  return results;
}

export function renderAgentOutputString(
  streamResponses: StreamResponse[]
): string {
  let output = "";
  for (const streamResponse of streamResponses) {
    console.log("streamResponse", streamResponse);
    switch (streamResponse.type) {
      case "Message":
        output += `<p>${streamResponse.content}</p>`;
        break;
      case "ToolCall":
        let call = ToolCallSchema.parse(streamResponse.content);
        output += `<p>${call.name}: ${call.params}</p>`;
        break;
      case "ToolResult":
        let result = ToolResultSchema.parse(streamResponse.content);
        output += `<p>${result.name}: ${result.result}</p>`;
        break;
    }
    console.log("output", output);
  }
  return output;
}
