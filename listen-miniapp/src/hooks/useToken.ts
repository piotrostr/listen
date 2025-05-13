import { useQuery } from "@tanstack/react-query";
import { worldchainEnabled } from "../config/env";
import { fetchTokenMetadata } from "../lib/solanaPortfolio";
import { LifiToken, LifiTokenSchema, TokenMetadata } from "../lib/types";
import { caip2Map, caip2ToLifiChainId } from "../lib/util";

export async function getAnyToken(
  token: string,
  chainIdOrCaip2: string
): Promise<LifiToken | null> {
  console.info("getAnyToken", token, chainIdOrCaip2);
  let chainId: number | null = null;
  if (chainIdOrCaip2.includes(":")) {
    chainId = caip2ToLifiChainId(chainIdOrCaip2);
  } else {
    if (Object.keys(caip2Map).includes(chainIdOrCaip2)) {
      const caip2 = caip2Map[chainIdOrCaip2 as keyof typeof caip2Map];
      chainId = caip2ToLifiChainId(caip2);
    } else {
      chainId = parseInt(chainIdOrCaip2);
    }
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
        const token = await getAnyToken(address, chainId);
        if (!token) {
          return null;
        }
        if (
          worldchainEnabled &&
          (chainId.includes("480") || chainId.includes("world"))
        ) {
          token.logoURI = `https://dd.dexscreener.com/ds-data/tokens/worldchain/${token.address.toLowerCase()}.png`;
        }
        return token;
      }
    },
  });

  return { data, isLoading, error };
};

async function getWorldchainTokenData(address: string): Promise<TokenMetadata> {
  const response = await fetch(
    `https://api.geckoterminal.com/api/v2/networks/world-chain/tokens/${address}`,
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
    chainId: 480, // World Chain ID
  };
}

export const useWorldchainToken = (address: string) => {
  const { data, isLoading, error } = useQuery({
    queryKey: ["worldchain-token", address],
    queryFn: () => getWorldchainTokenData(address),
  });

  return { data, isLoading, error };
};
