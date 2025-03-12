import { StreamResponse } from "../types/message";

export class JsonChunkReader {
  private buffer = "";

  append(chunk: string): StreamResponse[] {
    this.buffer += chunk;
    const messages: StreamResponse[] = [];
    const lines = this.buffer.split("\n");

    this.buffer = lines[lines.length - 1];

    for (let i = 0; i < lines.length - 1; i++) {
      const line = lines[i];
      if (line.startsWith("data: ")) {
        try {
          const jsonStr = line.slice(6);
          const data = JSON.parse(jsonStr);
          messages.push(data);
        } catch (e) {
          console.warn("Failed to parse JSON from line:", line, e);
        }
      }
    }

    return messages;
  }
}
