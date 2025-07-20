import { useQuery } from "@tanstack/react-query";
import { Hyperliquid } from "../lib/hype";
import { HyperliquidPortfolioOverview } from "../lib/hype-types";
import { PortfolioItem } from "../lib/types";

// Helper to convert Hyperliquid portfolio to PortfolioItem format
function convertHyperliquidToPortfolioItems(
  portfolio: HyperliquidPortfolioOverview
): PortfolioItem[] {
  const items: PortfolioItem[] = [];

  // Convert spot balances
  portfolio.spotBalances.balances.forEach((balance) => {
    if (parseFloat(balance.total) > 0) {
      items.push({
        address: balance.coin,
        name: balance.coin,
        symbol: balance.coin,
        decimals: 6, // Default decimals for Hyperliquid
        logoURI: balance.coin.toUpperCase() === "USDC" 
          ? "/usdc-logo.svg" 
          : `https://app.hyperliquid.xyz/coins/${balance.coin}_USDC.svg`,
        price: 1, // Would need to fetch actual price
        amount: parseFloat(balance.total),
        chain: "hyperliquid",
        priceChange24h: 0, // Would need to fetch from API
        volume24h: 0,
        type: "spot",
      });
    }
  });

  // Convert perp positions
  portfolio.perpBalances.assetPositions.forEach((position) => {
    const szi = parseFloat(position.position.szi);
    if (szi !== 0) {
      const positionValue = parseFloat(position.position.positionValue);
      const unrealizedPnl = parseFloat(position.position.unrealizedPnl);
      const entryPx = parseFloat(position.position.entryPx);
      
      items.push({
        address: position.position.coin,
        name: position.position.coin,
        symbol: position.position.coin,
        decimals: 6,
        logoURI: `https://app.hyperliquid.xyz/coins/${position.position.coin}.svg`,
        price: entryPx,
        amount: Math.abs(szi),
        chain: "hyperliquid",
        priceChange24h: positionValue > 0 ? (unrealizedPnl / positionValue) * 100 : 0,
        volume24h: 0,
        type: "perp",
      });
    }
  });

  return items;
}

async function fetchHyperliquidPortfolio(
  address: string | null
): Promise<PortfolioItem[] | null> {
  if (!address) return null;

  const hyperliquid = new Hyperliquid();
  const portfolio = await hyperliquid.portfolioOverview(address);
  
  if (!portfolio) return null;
  
  return convertHyperliquidToPortfolioItems(portfolio);
}

export function useHyperliquidPortfolio(address: string | null) {
  return useQuery({
    queryKey: ["hyperliquid-portfolio", address],
    queryFn: () => fetchHyperliquidPortfolio(address),
    enabled: !!address,
    staleTime: 5 * 60_000, // 5 minutes - data considered fresh
    gcTime: 30 * 60_000, // 30 minutes - keep in cache
    refetchInterval: 60_000, // Refetch every minute
    refetchOnWindowFocus: false, // Disable refetch on window focus
    refetchOnMount: false, // Disable refetch on component mount
    refetchOnReconnect: false, // Disable refetch on reconnect
  });
}
