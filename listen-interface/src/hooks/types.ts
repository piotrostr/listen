import { z } from "zod";

// this is the same pretty much as PortfolioItem - nice!
export const LifiTokenSchema = z.object({
  address: z.string(),
  name: z.string(),
  symbol: z.string(),
  decimals: z.number(),
  logo_uri: z.string().nullable(),
  chain_id: z.number(),
  price_usd: z.string().nullable(),
});

export type LifiToken = z.infer<typeof LifiTokenSchema>;

export type PortfolioItem = {
  address: string;
  name: string;
  symbol: string;
  decimals: number;
  logoURI: string;
  price: number;
  amount: number;
  chain: string;
};

export type PortfolioData = PortfolioItem[];

export interface TokenMetadata {
  address: string;
  name: string;
  symbol: string;
  decimals: number;
  logoURI: string;
  volume24h?: number;
  chainId?: number;
}

export interface PriceResponse {
  data: {
    [key: string]: {
      id: string;
      type: string;
      price: string;
    };
  };
}

export interface Holding {
  mint: string;
  ata: string;
  amount: bigint;
}
