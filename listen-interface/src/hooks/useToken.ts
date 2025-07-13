import { useQuery } from "@tanstack/react-query";
import { fetchTokenMetadataFromJupiter } from "../lib/solanaPortfolio";
import { TokenMetadata } from "../lib/types";
import { getNetworkId, imageMap } from "../lib/util";

export const useSolanaToken = (mint: string) => {
  const { data, isLoading, error } = useQuery({
    queryKey: ["solana-token", mint],
    queryFn: async () => {
      return fetchTokenMetadataFromJupiter(mint);
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
        const token = await fetchTokenMetadataFromJupiter(address);
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

const nativeToken = (geckoTerminalId: string): TokenMetadata => {
  if (geckoTerminalId === "solana") {
    return {
      address: "So11111111111111111111111111111111111111112",
      name: "Solana",
      symbol: "SOL",
      decimals: 9,
      logoURI: imageMap["solana"],
    };
  }

  if (
    geckoTerminalId === "eth" ||
    geckoTerminalId === "base" ||
    geckoTerminalId === "arbitrum"
  ) {
    return {
      address: "0x0000000000000000000000000000000000000000",
      name: "Ethereum",
      symbol: "ETH",
      decimals: 18,
      logoURI: imageMap["eth"],
    };
  }

  if (geckoTerminalId === "bsc") {
    return {
      address: "0x0000000000000000000000000000000000000000",
      name: "BNB",
      symbol: "BNB",
      decimals: 18,
      logoURI: imageMap["bnb"],
    };
  }

  return {
    address: "0x0000000000000000000000000000000000000000",
    name: "unknown",
    symbol: "unknown",
    decimals: 18,
    logoURI: null,
  };
};

export async function getTokenFallback(
  address: string,
  chainId: string
): Promise<TokenMetadata> {
  const geckoTerminalId = getNetworkId(chainId);
  if (!geckoTerminalId) {
    throw new Error("Invalid chain ID");
  }

  if (address === "native") {
    return nativeToken(geckoTerminalId);
  }

  const response = await fetch(
    `https://api.geckoterminal.com/api/v2/networks/${geckoTerminalId}/tokens/${address}`,
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
