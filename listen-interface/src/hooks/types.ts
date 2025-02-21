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
