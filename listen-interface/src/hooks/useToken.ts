import { useQuery } from "@tanstack/react-query";
import { fetchTokenMetadata } from "../lib/solanaPortfolio";
import { TokenMetadata } from "../lib/types";
import { getNetworkId } from "../lib/util";

export async function getSolanaTokenMetadata(
  mint: string
): Promise<TokenMetadata> {
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
      return await getTokenFallback(address, chainId);
    },
  });

  return { data, isLoading, error };
};

export const useToken = (address: string, chainId?: string) => {
  const { data, isLoading, error } = useQuery({
    queryKey: ["token", address, chainId],
    queryFn: async () => {
      if (!chainId || chainId.includes("solana")) {
        const token = await getSolanaTokenMetadata(address);
        if (!token || !token.logoURI) {
          return await getTokenFallback(address, "696969");
        }
        return token;
      } else {
        return await getTokenFallback(address, chainId ?? "696969");
      }
    },
  });

  return { data, isLoading, error };
};

export async function getTokenFallback(
  address: string,
  chainId: string
): Promise<TokenMetadata> {
  const response = await fetch(
    `https://api.geckoterminal.com/api/v2/networks/${getNetworkId(
      chainId
    )}/tokens/${address}`,
    {
      headers: {
        accept: "application/json",
        "Content-Type": "application/json",
      },
    }
  );

  if (!response.ok) {
    throw new Error("Failed to fetch token data");
  }

  const data = await response.json();
  const tokenData = data.data.attributes;

  return {
    address: tokenData.address,
    name: tokenData.name,
    symbol: tokenData.symbol,
    decimals: tokenData.decimals,
    logoURI: tokenData.image_url,
    volume24h: parseFloat(tokenData.volume_usd?.h24 || "0"),
    chainId: chainId,
  };
}
