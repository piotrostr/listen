import { z } from "zod";

// this is the same pretty much as PortfolioItem - nice!
export const LifiTokenSchema = z.object({
  address: z.string(),
  name: z.string(),
  symbol: z.string(),
  decimals: z.number(),
  logoURI: z.string().nullable().optional(),
  chainId: z.number(),
  priceUSD: z.string().nullable(),
});

export type LifiToken = z.infer<typeof LifiTokenSchema>;

export const TokenMetadataSchema = z.object({
  address: z.string(),
  name: z.string(),
  symbol: z.string(),
  decimals: z.number(),
  logoURI: z.string().optional().nullable(),
  volume24h: z.number().optional(),
  chainId: z.number().optional(),
});
export type TokenMetadata = z.infer<typeof TokenMetadataSchema>;

export const PriceResponseSchema = z.object({
  data: z.record(
    z.object({
      id: z.string(),
      type: z.string(),
      price: z.string(),
    })
  ),
});
export type PriceResponse = z.infer<typeof PriceResponseSchema>;

export const HoldingSchema = z.object({
  mint: z.string(),
  ata: z.string(),
  amount: z.bigint(),
});
export type Holding = z.infer<typeof HoldingSchema>;

export const PortfolioItemSchema = z.object({
  address: z.string(),
  name: z.string(),
  symbol: z.string(),
  decimals: z.number(),
  logoURI: z.string().nullable().optional(),
  price: z.number(),
  amount: z.number(),
  chain: z.string(),
});
export type PortfolioItem = z.infer<typeof PortfolioItemSchema>;

export const PortfolioDataSchema = z.array(PortfolioItemSchema);
export type PortfolioData = z.infer<typeof PortfolioDataSchema>;

export const CandlestickSchema = z.object({
  timestamp: z.number(),
  open: z.number(),
  high: z.number(),
  low: z.number(),
  close: z.number(),
  volume: z.number(),
});

export type Candlestick = z.infer<typeof CandlestickSchema>;

export const CandlestickDataSchema = z.array(CandlestickSchema);

export type CandlestickData = z.infer<typeof CandlestickDataSchema>;
