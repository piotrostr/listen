import { z } from "zod";

export const MessageDirectionSchema = z.enum(["incoming", "outgoing"]);
export type MessageDirection = z.infer<typeof MessageDirectionSchema>;

export const MessageTypeSchema = z.enum([
  "Message",
  "ToolCall",
  "ToolResult",
  "Error",
  "NestedAgentOutput",
  "ParToolCall",
  "ParToolResult",
]);
export type MessageType = z.infer<typeof MessageTypeSchema>;

const dateOrString = z.union([z.date(), z.string()]).transform((val) => {
  if (typeof val === "string") {
    return new Date(val);
  }
  return val;
});

export const MessageSchema = z.object({
  id: z.string(),
  message: z.string(),
  direction: MessageDirectionSchema,
  timestamp: dateOrString,
  type: MessageTypeSchema.optional(), // new approach, message type
  isToolCall: z.boolean().optional(), // legacy, supported but migrated
  edited: z.boolean().optional(), // New flag to indicate edited messages
});
export type Message = z.infer<typeof MessageSchema>;

export const ToolResultSchema = z.object({
  id: z.string().optional(),
  name: z.string(),
  result: z.string(),
});
export type ToolResult = z.infer<typeof ToolResultSchema>;

export const ToolCallSchema = z.object({
  id: z.string(),
  name: z.string(),
  params: z.string(),
});
export type ToolCall = z.infer<typeof ToolCallSchema>;

export const SimpleToolResultSchema = z.object({
  index: z.number(),
  id: z.string(),
  name: z.string(),
  result: z.string(),
});
export type SimpleToolResult = z.infer<typeof SimpleToolResultSchema>;

export const RigToolCallSchema = z.object({
  id: z.string(),
  function: z.object({
    name: z.string(),
    arguments: z.record(z.string(), z.any()),
  }),
});
export type RigToolCall = z.infer<typeof RigToolCallSchema>;

export const ParToolCallSchema = z.object({
  tool_calls: z.record(z.string(), RigToolCallSchema),
});
export type ParToolCall = z.infer<typeof ParToolCallSchema>;

export const ParToolResultSchema = z.object({
  tool_results: z.array(SimpleToolResultSchema),
});
export type ParToolResult = z.infer<typeof ParToolResultSchema>;

export const NestedAgentOutputSchema = z.object({
  agent_type: z.string(),
  content: z.string(),
});

export const StreamResponseSchema = z.object({
  type: MessageTypeSchema,
  content: z.union([
    z.string(),
    ToolResultSchema,
    ToolCallSchema,
    ParToolCallSchema,
    ParToolResultSchema,
    NestedAgentOutputSchema,
  ]),
});
export type StreamResponse = z.infer<typeof StreamResponseSchema>;

export const ChatSchema = z.object({
  id: z.string(),
  messages: z.array(MessageSchema),
  createdAt: dateOrString,
  lastMessageAt: dateOrString,
  title: z.string().optional(),
});
export type Chat = z.infer<typeof ChatSchema>;
