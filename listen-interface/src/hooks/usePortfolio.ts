import { useQuery } from "@tanstack/react-query";
import { PortfolioItem } from "./types";
import { useChatType } from "./useChatType";
import { useEvmPortfolio } from "./useEvmPortfolioAlchemy";
import { useSolanaPortfolio } from "./useSolanaPortfolio";

/**
 * Hook that combines Solana and EVM portfolios based on the current chat type
 */
export function usePortfolio() {
  const { chatType } = useChatType();
  const solanaQuery = useSolanaPortfolio();
  const evmQuery = useEvmPortfolio();

  // Calculate combined loading state
  const isLoading =
    chatType === "solana"
      ? solanaQuery.isLoading
      : solanaQuery.isLoading || evmQuery.isLoading;

  // Calculate combined error state
  const error =
    chatType === "solana"
      ? solanaQuery.error
      : solanaQuery.error || evmQuery.error;

  const result = useQuery({
    queryKey: ["combined-portfolio", chatType],
    queryFn: () => {
      // Combine assets based on the selected chat type
      if (chatType === "solana") {
        // Normalize Solana assets to ensure logoURI is always a string
        return (solanaQuery.data || []).map((asset) => ({
          ...asset,
          logoURI: asset.logoURI || "", // Convert null/undefined to empty string
        }));
      } else {
        // For "all" or any other type, combine both portfolios
        return [
          ...(evmQuery.data || []),
          // Normalize Solana assets
          ...(solanaQuery.data || []).map((asset) => ({
            ...asset,
            logoURI: asset.logoURI || "", // Convert null/undefined to empty string
          })),
        ] as PortfolioItem[];
      }
    },
    // Only run this query when the dependencies have data
    enabled:
      (chatType === "solana" && solanaQuery.data !== undefined) ||
      (chatType !== "solana" &&
        solanaQuery.data !== undefined &&
        evmQuery.data !== undefined),
    // This query depends on the underlying queries, so it should invalidate if they do
    staleTime: 10000,
  });

  // Calculate portfolio value inside the hook
  const portfolioValue = result.data ? getPortfolioTotalValue(result.data) : 0;

  // Return the query result along with our calculated loading state and portfolio value
  return {
    ...result,
    isLoading: result.isLoading || isLoading,
    error: result.error || error,
    portfolioValue,
  };
}

/**
 * Helper method to calculate total portfolio value
 */
export function getPortfolioTotalValue(assets: PortfolioItem[]): number {
  return assets.reduce((total, asset) => total + asset.price * asset.amount, 0);
}
