import React from "react";
import { useSettingsStore } from "../store/settingsStore";
import {
  ToolCallSchema,
  ToolResult,
  ToolResultSchema,
  type Message,
} from "../types/message";
import { ChatMessage } from "./ChatMessage";
import { EditableMessage } from "./EditableMessage";
import { FundWallet } from "./FundWallet";
import { PipelineDisplay } from "./Pipeline";
import { SolanaWalletCreation } from "./SolanaWalletCreation";
import { ThoughtsDisplay } from "./ThoughtsDisplay";
import { ToolMessage } from "./ToolMessage";

// Type definitions for tag handlers
type TagHandler = {
  processTag: (content: string, index: number, msg: Message) => JSX.Element;
  wrapResults?: (results: JSX.Element[]) => JSX.Element;
};

// Registry of tag handlers
const tagHandlers: Record<string, TagHandler> = {
  pipeline: {
    processTag: (content: string, index: number, msg: Message) => {
      try {
        const pipelineContent = content
          .trim()
          .replace(/\/\*[\s\S]*?\*\/|\/\/.*/g, ""); // Remove comments

        const pipeline = JSON.parse(pipelineContent);
        if (pipeline && pipeline.steps) {
          return (
            <div key={`pipeline-${index}`} className="my-4 pb-4">
              <PipelineDisplay pipeline={pipeline} />
            </div>
          );
        }
      } catch (e) {
        console.error(`Failed to parse pipeline JSON #${index + 1}:`, e);
        // If we can't parse the JSON, just render the raw content
        return (
          <ChatMessage
            key={`pipeline-error-${index}`}
            message={`<pipeline>${content}</pipeline>`}
            direction={msg.direction}
          />
        );
      }
      return <></>;
    },
    wrapResults: (results: JSX.Element[]) => (
      <div className="mb-6">{results}</div>
    ),
  },
  setup_solana_wallet: {
    processTag: (_content: string, index: number) => {
      return (
        <div key={`setup-solana-wallet-${index}`}>
          <SolanaWalletCreation error={null} />
        </div>
      );
    },
  },
  fund_solana_wallet: {
    processTag: (_content: string, index: number) => {
      return (
        <div key={`fund-solana-wallet-${index}`}>
          <FundWallet />
        </div>
      );
    },
  },
};

export function MessageRendererBase({
  message: msg,
  messages,
}: {
  message: Message;
  messages: Message[];
}) {
  const { debugMode } = useSettingsStore();
  if (!msg.message) return null;

  // Check if this is the last user message
  const isLastUserMessage = (() => {
    if (msg.direction !== "outgoing") return false;
    const lastUserMessageIndex = [...messages]
      .reverse()
      .findIndex((m) => m.direction === "outgoing");
    if (lastUserMessageIndex === -1) return false;
    return messages[messages.length - 1 - lastUserMessageIndex].id === msg.id;
  })();

  // this is to support previous version of message schema
  if (msg.isToolCall !== undefined && msg.isToolCall) {
    // tool call was tool result in v1, v2 there is a distinction, tool call is
    // passing params, tool result is the "tool output"
    const toolResult = handleLegacyMessage(msg);
    return (
      <ToolMessage
        toolOutput={toolResult}
        messages={messages}
        currentMessage={msg}
      />
    );
  }

  if (msg.type === "ToolCall") {
    try {
      const toolCall = ToolCallSchema.parse(JSON.parse(msg.message));
      if (toolCall.name === "think") {
        const thoughts = JSON.parse(toolCall.params);
        const thought = thoughts["thought"];
        return <ThoughtsDisplay thought={thought} />;
      }
    } catch (e) {
      console.error("Failed to parse thoughts", e);
    }
    if (debugMode) {
      return <ChatMessage message={msg.message} direction={msg.direction} />;
    }
    return null;
  }

  if (msg.type === "ToolResult") {
    const toolOutput = ToolResultSchema.parse(JSON.parse(msg.message));
    return (
      <ToolMessage
        toolOutput={toolOutput}
        messages={messages}
        currentMessage={msg}
      />
    );
  }

  // Check if this is a user message that can be edited
  if (msg.direction === "outgoing" && msg.type === "Message") {
    return (
      <EditableMessage message={msg} isLastUserMessage={isLastUserMessage} />
    );
  }

  // Check if the message contains any of our special tags
  const hasSpecialTags = Object.keys(tagHandlers).some((tagName) => {
    const tagRegex = new RegExp(`<${tagName}>.*?<\\/${tagName}>`, "s");
    return tagRegex.test(msg.message);
  });

  if (hasSpecialTags) {
    // Process the message with all supported tags
    return processMessageWithAllTags(msg.message, msg);
  }

  // Default case: render as a regular message
  return <ChatMessage message={msg.message} direction={msg.direction} />;
}

// New function to process a message with all supported tags
function processMessageWithAllTags(message: string, msg: Message): JSX.Element {
  // Create a structure to track all tag positions
  type TagPosition = {
    tagName: string;
    startIndex: number;
    endIndex: number;
    content: string;
  };

  const tagPositions: TagPosition[] = [];

  // Find all tag positions for all supported tag types
  Object.keys(tagHandlers).forEach((tagName) => {
    const tagRegex = new RegExp(`<${tagName}>(.*?)<\\/${tagName}>`, "gs");
    let match;

    while ((match = tagRegex.exec(message)) !== null) {
      tagPositions.push({
        tagName,
        startIndex: match.index,
        endIndex: match.index + match[0].length,
        content: match[1],
      });
    }
  });

  // Sort tag positions by their start index to maintain order
  tagPositions.sort((a, b) => a.startIndex - b.startIndex);

  // If no tags were found, return the original message
  if (tagPositions.length === 0) {
    return <ChatMessage message={message} direction={msg.direction} />;
  }

  // Split the message into parts
  const result: JSX.Element[] = [];
  let lastIndex = 0;

  tagPositions.forEach((pos, index) => {
    // Add text before the tag if there is any
    if (pos.startIndex > lastIndex) {
      const textBefore = message.substring(lastIndex, pos.startIndex);
      if (textBefore.trim()) {
        result.push(
          <ChatMessage
            key={`text-${index}`}
            message={textBefore}
            direction={msg.direction}
          />
        );
      }
    }

    // Process the tag content
    const handler = tagHandlers[pos.tagName];
    if (handler) {
      const processedTag = handler.processTag(pos.content, index, msg);
      result.push(processedTag);
    }

    lastIndex = pos.endIndex;
  });

  // Add any remaining text after the last tag
  if (lastIndex < message.length) {
    const textAfter = message.substring(lastIndex);
    if (textAfter.trim()) {
      result.push(
        <ChatMessage
          key={`text-final`}
          message={textAfter}
          direction={msg.direction}
        />
      );
    }
  }

  return <div>{result}</div>;
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

// Export a memoized version of MessageRenderer
export const MessageRenderer = React.memo(MessageRendererBase);
