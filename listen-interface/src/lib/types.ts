import { z } from "zod";
import {
  GtTokenMetadataSchema,
  TokenMetadataRawSchema,
} from "../types/metadata";

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
  chainId: z.union([z.number(), z.string()]).optional(),
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

export const PriceActionAnalysisResponseSchema = z.object({
  analysis: z.string(),
  current_price: z.number(),
  current_time: z.string(),
  total_volume: z.number(),
  price_change: z.number(),
  high: z.number(),
  low: z.number(),
});

export type PriceActionAnalysisResponse = z.infer<
  typeof PriceActionAnalysisResponseSchema
>;

export const SimplePriceTickSchema = z.object({
  price: z.number(),
});

export type SimplePriceTick = z.infer<typeof SimplePriceTickSchema>;

export const PriceInfoSchema = z.object({
  latest_price: z.number(),
  ema_price_ticks: z.array(SimplePriceTickSchema),
  price_ticks_timeframe: z.string(),
  total_volume: z.number(),
  pct_change: z.number(),
  period: z.string(),
});

export type PriceInfo = z.infer<typeof PriceInfoSchema>;

export const TokenSchema = z.object({
  metadata: z
    .union([TokenMetadataRawSchema, GtTokenMetadataSchema])
    .nullable()
    .optional(),
  price_info: PriceInfoSchema.nullable().optional(),
});

export type Token = z.infer<typeof TokenSchema>;
