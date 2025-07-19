import { useQuery } from "@tanstack/react-query";
import { fetchTokenMetadataFromJupiter } from "../lib/solanaPortfolio";
import { TokenMetadata } from "../lib/types";
import { getTokenFallback } from "./useToken";
import { z } from "zod";

// Import TopTokenSchema to ensure type compatibility
import { TopTokenSchema } from "../components/TopTokensDisplay";

type TopToken = z.infer<typeof TopTokenSchema>;

interface EnrichedTokenData extends TokenMetadata {
  // Additional fields from TopToken that we want to preserve
  price?: number;
  market_cap?: number;
  volume_24h?: number;
  price_change_24h?: number;
}

export const useEnrichedToken = (token: TopToken) => {
  const isEvm = token.pubkey.startsWith("0x");
  
  const { data, isLoading, error } = useQuery({
    queryKey: ["enriched-token", token.pubkey, token.chain_id],
    queryFn: async (): Promise<EnrichedTokenData> => {
      // Check if we already have sufficient data
      if (token.img_url && token.name) {
        // We already have the essential display data
        // Just transform it to match TokenMetadata interface
        return {
          address: token.pubkey,
          name: token.name,
          symbol: token.name, // Use name as symbol if we don't have it
          decimals: 0, // We don't need decimals for display
          logoURI: token.img_url,
          // Preserve the additional data we already have
          price: token.price,
          market_cap: token.market_cap,
          volume_24h: token.volume_24h,
          price_change_24h: token.price_change_24h,
          chainId: token.chain_id?.toString(),
        };
      }
      
      // Otherwise, fetch the missing data
      let metadata: TokenMetadata;
      
      if (!isEvm) {
        // Solana token
        metadata = await fetchTokenMetadataFromJupiter(token.pubkey);
        if (!metadata || !metadata.logoURI) {
          metadata = await getTokenFallback(token.pubkey, "696969");
        }
      } else {
        // EVM token
        metadata = await getTokenFallback(token.pubkey, token.chain_id ?? "696969");
      }
      
      // Merge fetched metadata with existing token data
      return {
        ...metadata,
        // Override with any existing data from TopToken
        logoURI: token.img_url || metadata.logoURI,
        name: token.name || metadata.name,
        // Preserve the additional data
        price: token.price,
        market_cap: token.market_cap,
        volume_24h: token.volume_24h,
        price_change_24h: token.price_change_24h,
      };
    },
    // Cache for longer since this data doesn't change often
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 30 * 60 * 1000, // 30 minutes
    // Only fetch if we're missing essential data
    enabled: !token.img_url || !token.name,
  });

  // If we're not fetching (because we have the data), return the transformed token data
  if (!isLoading && !data && token.img_url && token.name) {
    return {
      data: {
        address: token.pubkey,
        name: token.name,
        symbol: token.name,
        decimals: 0,
        logoURI: token.img_url,
        price: token.price,
        market_cap: token.market_cap,
        volume_24h: token.volume_24h,
        price_change_24h: token.price_change_24h,
        chainId: token.chain_id?.toString(),
      } as EnrichedTokenData,
      isLoading: false,
      error: null,
    };
  }

  return { data, isLoading, error };
};