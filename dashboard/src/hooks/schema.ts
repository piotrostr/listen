import { z } from "zod";

export const BalanceResponseSchema = z.object({
  balance: z.number(),
  pubkey: z.string(),
});

export const HoldingSchema = z.object({
  mint: z.string(),
  ata: z.string(),
  amount: z.number(),
});

export const HoldingsResponseSchema = z.object({
  holdings: z.array(HoldingSchema),
});

export const PriceResponseSchema = z.object({
  mint: z.string(),
  price: z.number(),
});

export const PumpBuyParamsSchema = z.object({
  mint: z.string(),
  sol_amount: z.number(),
  slippage: z.number(),
});

export const PumpSellParamsSchema = z.object({
  mint: z.string(),
  token_amount: z.number(),
  slippage: z.number(),
});

export const SwapParamsSchema = z.object({
  input_mint: z.string(),
  output_mint: z.string(),
  amount: z.number(),
  slippage: z.number(),
});

export const TokenBalanceParamsSchema = z.object({
  pubkey: z.string(),
  mint: z.string(),
});

export const TokenBalanceResponseSchema = z.object({
  balance: z.number(),
  mint: z.string(),
  pubkey: z.string(),
});

export const PubkeyResponseSchema = z.object({
  pubkey: z.string(),
});

export const SwapResponseSchema = z.object({
  result: z.string(),
  status: z.string(),
});

export type BalanceResponse = z.infer<typeof BalanceResponseSchema>;
export type Holding = z.infer<typeof HoldingSchema>;
export type HoldingsResponse = z.infer<typeof HoldingsResponseSchema>;
export type PriceResponse = z.infer<typeof PriceResponseSchema>;
export type PumpBuyParams = z.infer<typeof PumpBuyParamsSchema>;
export type PumpSellParams = z.infer<typeof PumpSellParamsSchema>;
export type SwapParams = z.infer<typeof SwapParamsSchema>;
export type TokenBalanceParams = z.infer<typeof TokenBalanceParamsSchema>;
export type TokenBalanceResponse = z.infer<typeof TokenBalanceResponseSchema>;
export type PubkeyResponse = z.infer<typeof PubkeyResponseSchema>;
export type SwapResponse = z.infer<typeof SwapResponseSchema>;
