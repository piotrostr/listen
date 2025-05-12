import { z } from "zod";

export enum PipelineActionType {
  SwapOrder = "SwapOrder",
  Notification = "Notification",
}

export enum PipelineConditionType {
  PriceAbove = "PriceAbove",
  PriceBelow = "PriceBelow",
  Now = "Now",
}

export const SwapOrderActionSchema = z.object({
  type: z.literal(PipelineActionType.SwapOrder),
  input_token: z.string(),
  output_token: z.string(),
  amount: z.string(),
  from_chain_caip2: z.string().optional(),
  to_chain_caip2: z.string().optional(),
});

export const NotificationActionSchema = z.object({
  type: z.literal(PipelineActionType.Notification),
  input_token: z.string().optional().nullable(),
  message: z.string(),
});

export const PipelineActionSchema = z.discriminatedUnion("type", [
  SwapOrderActionSchema,
  NotificationActionSchema,
]);

export const PipelineConditionSchema = z.object({
  type: z.nativeEnum(PipelineConditionType),
  asset: z.string(),
  value: z.number(),
});

export const PipelineStepSchema = z.object({
  action: PipelineActionSchema,
  conditions: z.array(PipelineConditionSchema).optional(),
});

export const PipelineSchema = z.object({
  steps: z.array(PipelineStepSchema),
});

export type SwapOrderAction = z.infer<typeof SwapOrderActionSchema>;
export type NotificationAction = z.infer<typeof NotificationActionSchema>;
export type PipelineAction = z.infer<typeof PipelineActionSchema>;
export type Pipeline = z.infer<typeof PipelineSchema>;
export type PipelineStep = z.infer<typeof PipelineStepSchema>;
export type PipelineCondition = z.infer<typeof PipelineConditionSchema>;
