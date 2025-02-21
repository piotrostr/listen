import { useQuery } from "@tanstack/react-query";
import { LifiToken, LifiTokenSchema, TokenMetadata } from "./types";
import { fetchTokenMetadata } from "./useSolanaPortfolio";

export async function getAnyToken(
  token: string,
  chainId: string
): Promise<LifiToken | null> {
  try {
    const res = await fetch(
      `https://li.quest/v1/token?mint=${token}&chainId=${chainId}`,
      {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json",
        },
      }
    );
    return LifiTokenSchema.parse(await res.json());
  } catch (error) {
    return null;
  }
}

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

export const useEvmToken = (address: string, chainId: string) => {
  const { data, isLoading, error } = useQuery({
    queryKey: ["evm-token", address],
    queryFn: async () => {
      const token = await getAnyToken(address, chainId);
      return token;
    },
  });

  return { data, isLoading, error };
};

export const useToken = (address: string, chainId: string) => {
  const { data, isLoading, error } = useQuery({
    queryKey: ["token", address, chainId],
    queryFn: async () => {
      const token = await getAnyToken(address, chainId);
      if (token) {
        return token;
      }
      return null;
    },
  });

  return { data, isLoading, error };
};
