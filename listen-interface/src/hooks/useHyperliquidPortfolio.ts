import { useQuery } from "@tanstack/react-query";
import { Hyperliquid } from "../lib/hype";
import {
  HyperliquidPortfolioOverview,
  SpotClearinghouseBalance,
} from "../lib/hype-types";
import { PortfolioItem } from "../lib/types";
import { useHyperliquidMids } from "./useHyperliquidMids";

function fixUPump(balance: SpotClearinghouseBalance): SpotClearinghouseBalance {
  if (balance.coin === "UPUMP") {
    return {
      ...balance,
      coin: "PUMP",
    };
  }

  return balance;
}

// Helper to convert Hyperliquid portfolio to PortfolioItem format
function convertHyperliquidToPortfolioItems(
  portfolio: HyperliquidPortfolioOverview,
  midsMap?: Map<string, number>,
): PortfolioItem[] {
  const items: PortfolioItem[] = [];

  // Convert spot balances
  portfolio.spotBalances.balances.map(fixUPump).forEach((balance) => {
    if (parseFloat(balance.total) > 0) {
      // Get price from mids map, default to 1 for USDC or if not found
      const price =
        balance.coin.toUpperCase() === "USDC"
          ? 1
          : midsMap?.get(balance.coin) || 1;

      items.push({
        address: balance.coin,
        name: balance.coin,
        symbol: balance.coin,
        decimals: 6, // Default decimals for Hyperliquid
        logoURI:
          balance.coin.toUpperCase() === "USDC"
            ? "/usdc-logo.svg"
            : `https://app.hyperliquid.xyz/coins/${balance.coin}_USDC.svg`,
        price: price,
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
        priceChange24h:
          positionValue > 0 ? (unrealizedPnl / positionValue) * 100 : 0,
        volume24h: 0,
        type: "perp",
      });
    }
  });

  return items;
}

async function fetchHyperliquidPortfolio(
  address: string | null,
  midsMap?: Map<string, number>,
): Promise<{ items: PortfolioItem[]; raw: HyperliquidPortfolioOverview } | null> {
  if (!address) return null;

  const hyperliquid = new Hyperliquid();
  const portfolio = await hyperliquid.portfolioOverview(address);

  if (!portfolio) return null;

  return {
    items: convertHyperliquidToPortfolioItems(portfolio, midsMap),
    raw: portfolio,
  };
}

export function useHyperliquidPortfolio(
  address: string | null,
  enabled: boolean = true,
) {
  // Always call hooks, but control their execution with enabled flag
  const { data: midsMap } = useHyperliquidMids();

  return useQuery({
    queryKey: ["hyperliquid-portfolio", address, midsMap?.size || 0], // Include mids size to refetch when prices update
    queryFn: () => fetchHyperliquidPortfolio(address, midsMap),
    enabled: !!address && enabled, // Only fetch when we have an address AND it's enabled
    staleTime: 5 * 60_000, // 5 minutes - data considered fresh
    gcTime: 30 * 60_000, // 30 minutes - keep in cache
    refetchInterval: 60_000, // Refetch every minute
    refetchOnWindowFocus: false, // Disable refetch on window focus
    refetchOnMount: false, // Disable refetch on component mount
    refetchOnReconnect: false, // Disable refetch on reconnect
  });
}
