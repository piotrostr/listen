import { z } from "zod";

export const DexScreenerTokenSchema = z.object({
  address: z.string(),
  name: z.string(),
  symbol: z.string(),
});

export const DexScreenerLiquiditySchema = z.object({
  usd: z.number(),
  base: z.number().nullable().optional(),
  quote: z.number().nullable().optional(),
});

export const DexScreenerVolumeSchema = z.object({
  h24: z.number(),
  h6: z.number().nullable().optional(),
  h1: z.number().nullable().optional(),
  m5: z.number().nullable().optional(),
});

export const DexScreenerPairSchema = z.object({
  chainId: z.string(),
  dexId: z.string(),
  url: z.string(),
  pairAddress: z.string(),
  labels: z.array(z.string()).nullable(),
  baseToken: DexScreenerTokenSchema,
  quoteToken: DexScreenerTokenSchema,
  priceNative: z.string(),
  priceUsd: z.string().nullable(),
  liquidity: DexScreenerLiquiditySchema.nullable(),
  volume: DexScreenerVolumeSchema,
});

export const DexScreenerResponseSchema = z.object({
  schemaVersion: z.string(),
  pairs: z.array(DexScreenerPairSchema),
});

export type DexScreenerToken = z.infer<typeof DexScreenerTokenSchema>;
export type DexScreenerPair = z.infer<typeof DexScreenerPairSchema>;
export type DexScreenerResponse = z.infer<typeof DexScreenerResponseSchema>;
