import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { processMessageWithAllTags, tagHandlers } from "../process-tags";
import { useSettingsStore } from "../store/settingsStore";
import {
  ParToolResultSchema,
  ToolCallSchema,
  ToolResult,
  ToolResultSchema,
  type Message,
} from "../types/message";
import { ChatMessage } from "./ChatMessage";
import { EditableMessage } from "./EditableMessage";
import { ParToolResultMessage } from "./ParToolResultMessage";
import { ThoughtsDisplay } from "./ThoughtsDisplay";
import { ToolMessage } from "./ToolMessage";

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

export function MessageRendererBase({
  message: msg,
  messages,
  lastUserMessageRef,
}: {
  message: Message;
  messages: Message[];
  lastUserMessageRef: React.RefObject<HTMLDivElement>;
}) {
  // console.log("MessageRenderer received message:", msg);

  const { t } = useTranslation();
  const { debugMode } = useSettingsStore();

  if (debugMode && msg.direction === "incoming") {
    return (
      <div>
        <div>{msg.message}</div>
        <div>{msg.direction}</div>
      </div>
    );
  }

  // Move the isLastUserMessage calculation into a useMemo
  const isLastUserMessage = useMemo(() => {
    if (msg.direction !== "outgoing") return false;
    const lastUserMessageIndex = [...messages]
      .reverse()
      .findIndex((m) => m.direction === "outgoing");
    if (lastUserMessageIndex === -1) return false;
    return messages[messages.length - 1 - lastUserMessageIndex].id === msg.id;
  }, [messages, msg.direction, msg.id]);

  if (!msg.message) return null;

  // Handle legacy tool call messages
  if (msg.isToolCall !== undefined && msg.isToolCall) {
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

  if (msg.type === "ParToolCall") {
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

  if (msg.type === "ParToolResult") {
    try {
      const parToolResult = ParToolResultSchema.parse(JSON.parse(msg.message));
      return (
        <ParToolResultMessage
          parToolResult={parToolResult}
          messages={messages}
          currentMessage={msg}
        />
      );
    } catch (e) {
      console.error("Failed to parse ParToolResult:", e);
      return (
        <ChatMessage
          message={`Error parsing ParToolResult: ${e}`}
          direction="incoming"
        />
      );
    }
  }

  // Check if this is a user message that can be edited
  if (msg.direction === "outgoing" && msg.type === "Message") {
    return (
      <div ref={isLastUserMessage ? lastUserMessageRef : undefined}>
        <EditableMessage message={msg} isLastUserMessage={isLastUserMessage} />
      </div>
    );
  }

  // Check if the message contains any of our special tags
  const hasSpecialTags =
    Object.keys(tagHandlers).some((tagName) => {
      const tagRegex = new RegExp(`<${tagName}>.*?<\\/${tagName}>`, "s");
      // Match markdown blocks with or without newlines
      const markdownTagRegex = new RegExp(
        "```" + tagName + "\\s*[\\s\\S]*?\\s*```",
        "s"
      );
      return tagRegex.test(msg.message) || markdownTagRegex.test(msg.message);
    }) || msg.message.includes("```json"); // Check for JSON blocks separately

  if (hasSpecialTags) {
    return processMessageWithAllTags(msg.message, msg);
  }

  if (msg.message.includes("String should match pattern '^[a-zA-Z0-9_-]+$'")) {
    return (
      <ChatMessage
        message={t("tool_messages.switch_model_error")}
        direction={msg.direction}
      />
    );
  }

  // Default case: render as a regular message
  return <ChatMessage message={msg.message} direction={msg.direction} />;
}

// Update the memo to use proper comparison
export const MessageRenderer = React.memo(
  MessageRendererBase,
  (prevProps, nextProps) => {
    return (
      prevProps.message === nextProps.message &&
      prevProps.messages === nextProps.messages &&
      prevProps.lastUserMessageRef === nextProps.lastUserMessageRef
    );
  }
);
