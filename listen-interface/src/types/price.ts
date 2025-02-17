import { z } from "zod";

export const PriceUpdateSchema = z.object({
  name: z.string(),
  pubkey: z.string(),
  price: z.number(),
  market_cap: z.number(),
  timestamp: z.number(),
  slot: z.number(),
  swap_amount: z.number(),
  owner: z.string(),
  signature: z.string(),
  multi_hop: z.boolean(),
  is_buy: z.boolean(),
  is_pump: z.boolean(),
});

export type PriceUpdate = z.infer<typeof PriceUpdateSchema>;
