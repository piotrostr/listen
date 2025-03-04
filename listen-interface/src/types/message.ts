import { z } from "zod";

export const MessageDirectionSchema = z.enum(["incoming", "outgoing"]);
export type MessageDirection = z.infer<typeof MessageDirectionSchema>;

export const MessageTypeSchema = z.enum([
  "Message",
  "ToolCall",
  "ToolResult",
  "Error",
]);
export type MessageType = z.infer<typeof MessageTypeSchema>;

export const MessageSchema = z.object({
  id: z.string(),
  message: z.string(),
  direction: MessageDirectionSchema,
  timestamp: z.date(),
  type: MessageTypeSchema.optional(), // new approach, message type
  isToolCall: z.boolean().optional(), // legacy, supported but migrated
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

export const StreamResponseSchema = z.object({
  type: MessageTypeSchema,
  content: z.union([z.string(), ToolResultSchema, ToolCallSchema]),
});
export type StreamResponse = z.infer<typeof StreamResponseSchema>;

export const ChatSchema = z.object({
  id: z.string(),
  messages: z.array(MessageSchema),
  createdAt: z.date(),
  lastMessageAt: z.date(),
  title: z.string().optional(),
});
export type Chat = z.infer<typeof ChatSchema>;
