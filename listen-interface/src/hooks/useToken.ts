import { useQuery } from "@tanstack/react-query";
import { LifiToken, LifiTokenSchema, TokenMetadata } from "./types";
import { fetchTokenMetadata } from "./useSolanaPortfolio";
import { caip2ToLifiChainId } from "./util";

export async function getAnyToken(
  token: string,
  chainIdOrCaip2: string
): Promise<LifiToken | null> {
  console.info("getAnyToken", token, chainIdOrCaip2);
  let chainId: number | null = null;
  if (chainIdOrCaip2.includes(":")) {
    chainId = caip2ToLifiChainId(chainIdOrCaip2);
  } else {
    chainId = parseInt(chainIdOrCaip2);
  }
  try {
    const res = await fetch(
      `https://li.quest/v1/token?token=${token}&chain=${chainId}`,
      {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json",
        },
      }
    );
    if (!res.ok) {
      console.error(res);
      return null;
    }
    const data = await res.json();
    return LifiTokenSchema.parse(data);
  } catch (error) {
    console.error(error);
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

export const useToken = (address: string, chainId?: string) => {
  const { data, isLoading, error } = useQuery({
    queryKey: ["token", address, chainId],
    queryFn: async () => {
      if (!chainId || chainId === "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp") {
        return await getSolanaTokenMetadata(address);
      } else {
        return await getAnyToken(address, chainId);
      }
    },
  });

  return { data, isLoading, error };
};
