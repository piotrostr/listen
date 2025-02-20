import { useQuery } from "@tanstack/react-query";
import { TokenMetadata } from "./types";
import { getTokensMetadata } from "./useEvmPortfolio";
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

export const useEvmToken = (address: string) => {
  const { data, isLoading, error } = useQuery({
    queryKey: ["evm-token", address],
    queryFn: async () => {
      const tokens = await getTokensMetadata([address]);
      return tokens.get(address);
    },
  });

  return { data, isLoading, error };
};
