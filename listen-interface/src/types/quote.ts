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

export const JupiterQuoteResponseSchema = z.object({
  contextSlot: z.number().optional(),
  inAmount: z.string(),
  inputMint: z.string(),
  otherAmountThreshold: z.string(),
  outAmount: z.string(),
  outputMint: z.string(),
  platformFee: z
    .object({
      amount: z.string().optional(),
      feeBps: z.number().optional(),
    })
    .nullable()
    .optional(),
  priceImpactPct: z.string(),
  routePlan: z.array(
    z.object({
      percent: z.number(),
      swapInfo: z.object({
        ammKey: z.string(),
        feeAmount: z.string(),
        feeMint: z.string(),
        inAmount: z.string(),
        inputMint: z.string(),
        label: z.string().optional(),
        outAmount: z.string(),
        outputMint: z.string(),
      }),
    })
  ),
  slippageBps: z.number(),
  swapMode: z.string(),
  timeTaken: z.number().optional(),
});

export type QuoteResponse = z.infer<typeof QuoteResponseSchema>;
export type JupiterQuoteResponse = z.infer<typeof JupiterQuoteResponseSchema>;
