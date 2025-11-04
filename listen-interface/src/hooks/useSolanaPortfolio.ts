import { useQuery } from "@tanstack/react-query";
import { fetchPortfolio as fetchSolanaPortfolio } from "../lib/solanaPortfolio";
import { PortfolioItem } from "../lib/types";

async function fetchSolanaPortfolioData(
  address: string | null
): Promise<PortfolioItem[] | null> {
  if (!address) return null;

  return await fetchSolanaPortfolio(address);
}

export function useSolanaPortfolio(address: string | null) {
  return useQuery({
    queryKey: ["solana-portfolio", address],
    queryFn: () => fetchSolanaPortfolioData(address),
    enabled: !!address,
    staleTime: 5 * 60_000, // 5 minutes - data considered fresh
    gcTime: 30 * 60_000, // 30 minutes - keep in cache
    refetchInterval: 60_000, // Refetch every minute
    refetchOnWindowFocus: false, // Disable refetch on window focus
    refetchOnMount: false, // Disable refetch on component mount
    refetchOnReconnect: false, // Disable refetch on reconnect
  });
}