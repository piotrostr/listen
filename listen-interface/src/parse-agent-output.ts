import { StreamResponse, StreamResponseSchema } from "./types/message";

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

  // Use regex to extract all JSON objects enclosed in <content> tags
  const contentRegex = /<content>(.*?)<\/content>/gs;
  let match;

  while ((match = contentRegex.exec(output)) !== null) {
    try {
      const jsonStr = match[1];
      console.log("jsonStr", jsonStr);
      const parsed = JSON.parse(jsonStr);
      const validated = StreamResponseSchema.parse(parsed);
      results.push(validated);
    } catch (error) {
      console.error("Error parsing JSON:", error);
    }
  }

  return results;
}
