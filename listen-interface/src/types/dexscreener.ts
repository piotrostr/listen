import { z } from "zod";

export const DexScreenerTokenSchema = z.object({
  address: z.string(),
  name: z.string(),
  symbol: z.string(),
});

export const DexScreenerLiquiditySchema = z.object({
  usd: z.number(),
  base: z.number(),
  quote: z.number(),
});

export const DexScreenerVolumeSchema = z.object({
  h24: z.number(),
  h6: z.number(),
  h1: z.number(),
  m5: z.number(),
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
  priceUsd: z.string(),
  liquidity: DexScreenerLiquiditySchema,
  volume: DexScreenerVolumeSchema,
});

export const DexScreenerResponseSchema = z.object({
  schemaVersion: z.string(),
  pairs: z.array(DexScreenerPairSchema),
});

export type DexScreenerToken = z.infer<typeof DexScreenerTokenSchema>;
export type DexScreenerPair = z.infer<typeof DexScreenerPairSchema>;
export type DexScreenerResponse = z.infer<typeof DexScreenerResponseSchema>;
