import { useQuery } from "@tanstack/react-query";
import { TokenMetadata } from "./types";
import { fetchTokenMetadata } from "./useSolanaPortfolio";

async function getSolanaTokenMetadata(mint: string): Promise<TokenMetadata> {
  return fetchTokenMetadata(mint);
}

export const useSolanaToken = (mint: string) => {
  const { data, isLoading, error } = useQuery({
    queryKey: ["solana-token", mint],
    queryFn: async () => {
      return getSolanaTokenMetadata(mint);
    },
  });

  return { data, isLoading, error };
};
