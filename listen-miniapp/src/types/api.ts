// this is the schema of what api returns
import { z } from "zod";

// Update condition types to match Rust enum
const PriceAboveConditionSchema = z.object({
  PriceAbove: z.object({
    asset: z.string(),
    value: z.number(),
  }),
});

const PriceConditionSchema = z.object({
  PriceBelow: z.object({
    asset: z.string(),
    value: z.number(),
  }),
});

const NowConditionSchema = z.object({
  Now: z.object({
    asset: z.string(),
  }),
});

type ConditionType = z.infer<typeof ConditionTypeSchema>;

const AndConditionSchema: z.ZodType<{ And: ConditionType[] }> = z.object({
  And: z.array(z.lazy(() => ConditionTypeSchema)),
});

const OrConditionSchema: z.ZodType<{ Or: ConditionType[] }> = z.object({
  Or: z.array(z.lazy(() => ConditionTypeSchema)),
});

// Combined condition type schema using discriminated union
const ConditionTypeSchema = z.union([
  PriceAboveConditionSchema,
  PriceConditionSchema,
  NowConditionSchema,
  AndConditionSchema,
  OrConditionSchema,
]);

const ConditionSchema = z.object({
  condition_type: ConditionTypeSchema,
  triggered: z.boolean(),
  last_evaluated: z.string().datetime().nullable(),
});

// Update status enum
export const StatusSchema = z.enum([
  "Pending",
  "Completed",
  "Failed",
  "Cancelled",
]);

const OrderActionSchema = z.object({
  Order: z.object({
    amount: z.string(),
    from_chain_caip2: z.string(),
    input_token: z.string(),
    output_token: z.string(),
    to_chain_caip2: z.string(),
  }),
});

const NotificationActionSchema = z.object({
  Notification: z.object({
    input_token: z.string().optional().nullable(),
    message: z.string(),
  }),
});

// Combined action schema using discriminated union
const ExtendedActionSchema = z.union([
  OrderActionSchema,
  NotificationActionSchema,
]);

// Update pipeline step schema
const ExtendedPipelineStepSchema = z.object({
  action: ExtendedActionSchema,
  conditions: z.array(ConditionSchema),
  id: z.string().uuid(),
  next_steps: z.array(z.string().uuid()),
  status: StatusSchema,
  transaction_hash: z.string().nullable(),
  error: z.string().nullable().optional(),
});

// Update pipeline schema
export const ExtendedPipelineSchema = z.object({
  created_at: z.string().datetime(),
  current_steps: z.array(z.string().uuid()),
  id: z.string().uuid(),
  pubkey: z.string().optional().nullable(),
  status: StatusSchema,
  steps: z.record(z.string().uuid(), ExtendedPipelineStepSchema),
  user_id: z.string(),
  wallet_address: z.string().optional().nullable(),
});

// Updated response schema
export const ExtendedPipelineResponseSchema = z.object({
  status: z.string(),
  pipelines: z.array(ExtendedPipelineSchema),
});

// Export types
export type ExtendedPipelineResponse = z.infer<
  typeof ExtendedPipelineResponseSchema
>;
export type ExtendedPipeline = z.infer<typeof ExtendedPipelineSchema>;
export type ExtendedPipelineStep = z.infer<typeof ExtendedPipelineStepSchema>;
export type ExtendedPipelineCondition = z.infer<typeof ConditionSchema>;
export type Status = z.infer<typeof StatusSchema>;
