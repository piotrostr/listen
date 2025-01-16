import { useQuery } from "@tanstack/react-query";
import { HoldingsResponse } from "./schema";

export type PortfolioItem = {
  address: string;
  name: string;
  symbol: string;
  decimals: number;
  logoURI: string;
  price: number;
  amount: number;
  daily_volume: number;
};

export type PortfolioData = PortfolioItem[];

interface TokenMetadata {
  address: string;
  name: string;
  symbol: string;
  decimals: number;
  logoURI: string;
  volume24h?: number;
}

interface PriceResponse {
  data: {
    [key: string]: {
      id: string;
      type: string;
      price: string;
    };
  };
}

export const usePortfolio = () => {
  const API_BASE = "http://localhost:6969";

  const fetchTokenMetadata = async (mint: string): Promise<TokenMetadata> => {
    const response = await fetch(`https://tokens.jup.ag/token/${mint}`);
    return response.json();
  };

  const fetchPrices = async (mints: string[]): Promise<PriceResponse> => {
    const response = await fetch(
      `https://api.jup.ag/price/v2?ids=${mints.join(",")}`,
    );
    return response.json();
  };

  const fetchHoldings = async (): Promise<HoldingsResponse> => {
    const response = await fetch(`${API_BASE}/holdings`);
    return response.json();
  };

  // Main query function
  const fetchPortfolioData = async (): Promise<PortfolioData> => {
    // 1. Get holdings
    const holdingsResponse = await fetchHoldings();
    const holdings = holdingsResponse.holdings;

    // 2. Fetch token metadata for all mints
    const metadataPromises = holdings.map((holding) =>
      fetchTokenMetadata(holding.mint),
    );
    const tokenMetadata = await Promise.all(metadataPromises);

    // 3. Fetch prices for all mints
    const mints = holdings.map((holding) => holding.mint);
    const pricesResponse = await fetchPrices(mints);

    // 4. Combine all data
    return holdings.map((holding, index) => {
      const metadata = tokenMetadata[index];
      const price = Number(pricesResponse.data[holding.mint].price);
      const amount = holding.amount / Math.pow(10, metadata.decimals);

      return {
        address: metadata.address,
        name: metadata.name,
        symbol: metadata.symbol,
        decimals: metadata.decimals,
        logoURI: metadata.logoURI,
        price,
        amount,
        daily_volume: metadata.volume24h || 0,
      };
    });
  };

  // Use React Query to handle the data fetching
  return useQuery<PortfolioData, Error>({
    queryKey: ["portfolio"],
    queryFn: fetchPortfolioData,
    refetchInterval: 30000, // Refetch every 60 seconds
    staleTime: 30000, // Consider data stale after 60 seconds
  });
};
