import { z } from "zod";

export const ClearinghouseAssetPositionSchema = z.object({
  position: z.object({
    coin: z.string(),
    cumFunding: z.object({
      allTime: z.string(),
      sinceChange: z.string(),
      sinceOpen: z.string(),
    }),
    entryPx: z.string(),
    leverage: z.object({
      type: z.string(),
      value: z.number(),
    }),
    liquidationPx: z.string(),
    marginUsed: z.string(),
    maxLeverage: z.number(),
    positionValue: z.string(),
    returnOnEquity: z.string(),
    szi: z.string(),
    unrealizedPnl: z.string(),
  }),
  type: z.string(),
});

export type ClearinghouseAssetPosition = z.infer<
  typeof ClearinghouseAssetPositionSchema
>;

export const ClearinghouseStateSchema = z.object({
  marginSummary: z.object({
    accountValue: z.string(),
    totalNtlPos: z.string(),
    totalRawUsd: z.string(),
    totalMarginUsed: z.string(),
  }),
  crossMarginSummary: z.object({
    accountValue: z.string(),
    totalNtlPos: z.string(),
    totalRawUsd: z.string(),
    totalMarginUsed: z.string(),
  }),
  crossMaintenanceMarginUsed: z.optional(z.string()),
  withdrawable: z.string(),
  assetPositions: z.array(ClearinghouseAssetPositionSchema),
  time: z.optional(z.number()),
});

export type ClearinghouseState = z.infer<typeof ClearinghouseStateSchema>;

export const SpotClearinghouseStateSchema = z.object({
  balances: z.array(
    z.object({
      coin: z.string(),
      token: z.number(),
      hold: z.string(),
      total: z.string(),
      entryNtl: z.string(),
    })
  ),
});

export type SpotClearinghouseState = z.infer<
  typeof SpotClearinghouseStateSchema
>;

export const HyperliquidPortfolioOverviewSchema = z.object({
  spotBalances: SpotClearinghouseStateSchema,
  perpBalances: ClearinghouseStateSchema,
});

export type HyperliquidPortfolioOverview = z.infer<
  typeof HyperliquidPortfolioOverviewSchema
>;
