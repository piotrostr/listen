import { z } from "zod";

const TokenInfoSchema = z.object({
  address: z.string(),
  amount: z.string(),
  chain_id: z.number(),
  decimals: z.number(),
  token: z.string(),
});

const ToTokenInfoSchema = TokenInfoSchema.extend({
  amount_min: z.string(),
});

export const QuoteResponseSchema = z.object({
  costs: z.record(z.string(), z.string()),
  execution_time_seconds: z.number(),
  from: TokenInfoSchema,
  slippage_percent: z.number(),
  to: ToTokenInfoSchema,
});

export type QuoteResponse = z.infer<typeof QuoteResponseSchema>;
