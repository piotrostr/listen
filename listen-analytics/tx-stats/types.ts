import { z } from "zod";

export const WalletSchema = z.object({
  id: z.string(),
  address: z.string(),
  chain_type: z.union([z.literal("solana"), z.literal("ethereum")]),
  created_at: z.number(),
});

export type Wallet = z.infer<typeof WalletSchema>;

export const WalletResponseSchema = z.object({
  data: z.array(WalletSchema),
  next_cursor: z.string().nullable().optional(),
});
