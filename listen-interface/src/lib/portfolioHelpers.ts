import { PortfolioItem } from "./types";

// Map of symbols that should be aggregated
const SYMBOL_AGGREGATION_MAP: Record<string, string> = {
  WETH: "ETH",
  ETH: "ETH",
  USDC: "USDC",
  // Add more as needed
};

export interface AggregatedPortfolioItem extends Omit<PortfolioItem, "chain"> {
  chains: string[]; // Array of chains where this asset exists
  originalItems: PortfolioItem[]; // Keep track of original items for reference
  isAggregated: boolean; // Flag to indicate if this is an aggregated item
}

export function aggregatePortfolioItems(
  items: PortfolioItem[],
): AggregatedPortfolioItem[] {
  const aggregationMap = new Map<string, AggregatedPortfolioItem>();

  items.forEach((item) => {
    // Get the canonical symbol (e.g., WETH -> ETH)
    const canonicalSymbol =
      SYMBOL_AGGREGATION_MAP[item.symbol.toUpperCase()] || item.symbol;

    // Only aggregate if it's in our aggregation map
    if (SYMBOL_AGGREGATION_MAP[item.symbol.toUpperCase()]) {
      const key = canonicalSymbol;

      if (aggregationMap.has(key)) {
        // Aggregate with existing item
        const existing = aggregationMap.get(key)!;

        // Add amounts
        existing.amount += item.amount;

        // Add chain if not already present
        if (!existing.chains.includes(item.chain)) {
          existing.chains.push(item.chain);
        }

        // Keep track of original items
        existing.originalItems.push(item);

        // Recalculate weighted average price
        const totalValue = existing.originalItems.reduce(
          (sum, i) => sum + i.price * i.amount,
          0,
        );
        const totalAmount = existing.originalItems.reduce(
          (sum, i) => sum + i.amount,
          0,
        );
        existing.price = totalAmount > 0 ? totalValue / totalAmount : 0;

        // Recalculate weighted average price change
        existing.priceChange24h =
          totalAmount > 0
            ? existing.originalItems.reduce(
                (sum, i) => sum + i.priceChange24h * i.amount,
                0,
              ) / totalAmount
            : 0;
      } else {
        // Create new aggregated item
        aggregationMap.set(key, {
          ...item,
          symbol: canonicalSymbol,
          name: canonicalSymbol === "ETH" ? "Ethereum" : canonicalSymbol,
          chains: [item.chain],
          originalItems: [item],
          // Use appropriate logos for aggregated tokens
          logoURI:
            canonicalSymbol === "ETH"
              ? "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/ethereum/info/logo.png"
              : item.logoURI,
          isAggregated: true,
        });
      }
    } else {
      // Non-aggregated item, just convert to aggregated format
      aggregationMap.set(`${item.symbol}-${item.chain}`, {
        ...item,
        chains: [item.chain],
        originalItems: [item],
        isAggregated: false,
      });
    }
  });

  return Array.from(aggregationMap.values());
}

// Helper to get chain icon path
export function getChainIcon(chain: string): string {
  const chainIcons: Record<string, string> = {
    solana: "/solana-logo.svg",
    ethereum: "/eth-logo.svg",
    polygon: "/polygon-logo.svg",
    arbitrum: "/arbitrum-logo.svg",
    optimism: "/optimism-logo.svg",
    base: "/base-logo.svg",
    hyperliquid: "/hyperliquid-logo.svg",
    // Add more chains as needed
  };

  return chainIcons[chain.toLowerCase()] || "/default-chain-logo.svg";
}
