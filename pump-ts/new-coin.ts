import { z } from "zod";

export const NewCoinCreatedSchema = z.object({
  mint: z.string(),
  name: z.string(),
  symbol: z.string(),
  description: z.string(),
  image_uri: z.string(),
  metadata_uri: z.string(),
  twitter: z.string().nullable(),
  telegram: z.string().nullable(),
  bonding_curve: z.string(),
  associated_bonding_curve: z.string(),
  creator: z.string(),
  created_timestamp: z.number(),
  raydium_pool: z.unknown().nullable(),
  complete: z.boolean(),
  virtual_sol_reserves: z.number(),
  virtual_token_reserves: z.number(),
  hidden: z.unknown().nullable(),
  total_supply: z.number(),
  website: z.string().nullable(),
  show_name: z.boolean(),
  last_trade_timestamp: z.number().nullable(),
  king_of_the_hill_timestamp: z.number().nullable(),
  market_cap: z.number(),
  nsfw: z.boolean(),
  market_id: z.string().nullable(),
  inverted: z.unknown().nullable(),
  real_sol_reserves: z.number(),
  real_token_reserves: z.number(),
  livestream_ban_expiry: z.number(),
  last_reply: z.number().nullable(),
  reply_count: z.number(),
  is_banned: z.boolean(),
  is_currently_live: z.boolean(),
  usd_market_cap: z.number(),
});

export type NewCoin = z.infer<typeof NewCoinCreatedSchema>;
