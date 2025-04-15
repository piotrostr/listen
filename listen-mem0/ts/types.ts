import { z } from "zod";

export const MessageSchema = z.object({
  role: z.string(),
  content: z.string(),
});

export type Message = z.infer<typeof MessageSchema>;

export const SearchFiltersSchema = z.object({
  user_id: z.string(),
  agent_id: z.string().optional(),
  run_id: z.string().optional(),
});

export type SearchFilters = z.infer<typeof SearchFiltersSchema>;

const EntitySchema = z.object({
  user_id: z.string(),
  agent_id: z.string().optional(),
  run_id: z.string().optional(),
});

export type Entity = z.infer<typeof EntitySchema>;

export const AddMemoryOptionsSchema = EntitySchema.extend({
  user_id: z.string(),
});

export type AddMemoryOptions = z.infer<typeof AddMemoryOptionsSchema>;

export const SearchMemoryOptionsSchema = EntitySchema.extend({
  limit: z.number().optional(),
  filters: SearchFiltersSchema.optional(),
});

export type SearchMemoryOptions = z.infer<typeof SearchMemoryOptionsSchema>;

export const GetAllMemoryOptionsSchema = EntitySchema.extend({
  limit: z.number().optional(),
});

export type GetAllMemoryOptions = z.infer<typeof GetAllMemoryOptionsSchema>;

export const DeleteAllMemoryOptionsSchema = EntitySchema;

export type DeleteAllMemoryOptions = z.infer<
  typeof DeleteAllMemoryOptionsSchema
>;

export const AddMemorySchema = z.object({
  messages: z.array(MessageSchema),
  config: AddMemoryOptionsSchema,
});

export const MemoryItemSchema = z.object({
  id: z.string(),
  memory: z.string(),
  hash: z.string().optional(),
  createdAt: z.string().optional(),
  updatedAt: z.string().optional(),
  score: z.number().optional(),
  metadata: z.record(z.any()).optional(),
});

export const GraphMemoryResultSchema = z.object({
  deleted_entities: z.array(z.any()),
  added_entities: z.array(z.any()),
  relations: z.array(z.any()).optional(),
});

export const AddMemoryResultSchema = z.object({
  results: z.array(MemoryItemSchema),
  graph: GraphMemoryResultSchema.optional(),
});
