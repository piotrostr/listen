import { useQuery } from "@tanstack/react-query";
import { Hyperliquid } from "../lib/hype";
import { HyperliquidPortfolioOverview } from "../lib/hype-types";

async function fetchHyperliquidPortfolio(
  address: string | null
): Promise<HyperliquidPortfolioOverview | null> {
  if (!address) return null;

  const hyperliquid = new Hyperliquid();
  return await hyperliquid.portfolioOverview(address);
}

export function useHyperliquidPortfolio(address: string | null) {
  return useQuery({
    queryKey: ["hyperliquid-portfolio", address],
    queryFn: () => fetchHyperliquidPortfolio(address),
    enabled: !!address,
    staleTime: 60_000, // 1 minute
    refetchInterval: 60_000, // Refetch every minute
  });
}
