import { useQuery } from "@tanstack/react-query";
import { fetchTokenPrices, TokenPrice, WSOL_MINT } from "../lib/price";

async function fetchSolanaPrice(): Promise<TokenPrice> {
  const tokens = [{ address: WSOL_MINT, chain: "solana" }];
  const prices = await fetchTokenPrices(tokens);
  return prices.get(WSOL_MINT) as TokenPrice;
}

export function useSolanaPrice() {
  return useQuery({
    queryKey: ["solana-price"],
    queryFn: fetchSolanaPrice,
    staleTime: 30_000, // 30 seconds
    refetchInterval: 30_000, // Also refetch every 30 seconds
  });
}
