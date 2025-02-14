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

const IpfsMetadataSchema = z.object({
  createdOn: z.string().nullable().optional(),
  description: z.string().nullable().optional(),
  image: z.string().nullable().optional(),
  name: z.string(),
  symbol: z.string(),
  showName: z.boolean().nullable().optional(),
  twitter: z.string().nullable().optional(),
  website: z.string().nullable().optional(),
});

const MplTokenMetadataSchema = z.object({
  name: z.string(),
  symbol: z.string(),
  uri: z.string(),
  ipfs_metadata: IpfsMetadataSchema.nullable().optional(),
});

const SplTokenMetadataSchema = z.object({
  mint_authority: z.string().nullable().optional(),
  supply: z.number(),
  decimals: z.number(),
  is_initialized: z.boolean(),
  freeze_authority: z.string().nullable().optional(),
});

export const TokenMetadataSchema = z.object({
  mint: z.string(),
  mpl: MplTokenMetadataSchema,
  spl: SplTokenMetadataSchema,
});

export type TokenMetadata = z.infer<typeof TokenMetadataSchema>;

export interface TokenData {
  name: string;
  buyVolume: number;
  sellVolume: number;
  lastPrice: number;
  lastUpdate: Date;
  marketCap: number;
  uniqueAddresses: Set<string>;
  pubkey: string;
}
