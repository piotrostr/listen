import { getAddress } from "viem";
import { z } from "zod";
import { tokenMetadataCache } from "./localStorage";
import { fetchTokenPrices } from "./price";
import { PortfolioItem, TokenMetadata } from "./types";

const SUPPORTED_NETWORKS = [
  { chainId: "1", networkId: "eth-mainnet", chain: "ethereum" },
  { chainId: "42161", networkId: "arb-mainnet", chain: "arbitrum" },
  { chainId: "56", networkId: "bnb-mainnet", chain: "bsc" },
  { chainId: "8453", networkId: "base-mainnet", chain: "base" },
  { chainId: "480", networkId: "worldchain-mainnet", chain: "worldchain" },
] as const;

const TokenPriceSchema = z.object({
  currency: z.string(),
  value: z.string(),
  lastUpdatedAt: z.string(),
});

const TokenMetadataSchema = z.object({
  symbol: z.string().nullable().optional(),
  decimals: z.number().nullable().optional(),
  name: z.string().nullable().optional(),
  logo: z.string().nullable().optional(),
});

const TokenSchema = z.object({
  network: z.string(),
  tokenAddress: z.string().nullable(),
  tokenBalance: z.string(),
  tokenMetadata: TokenMetadataSchema.optional(),
  tokenPrices: z.array(TokenPriceSchema).optional(),
});

const AlchemyResponseSchema = z.object({
  data: z.object({
    tokens: z.array(TokenSchema),
  }),
});

import { LifiToken, LifiTokenSchema } from "../lib/types";
import { caip2Map, caip2ToLifiChainId } from "../lib/util";
import { tokenDenylistCache } from "./denylistCache";

export async function getAnyToken(
  token: string,
  chainIdOrCaip2: string
): Promise<LifiToken | null> {
  console.info("getAnyToken", token, chainIdOrCaip2);
  let chainId: number | string | null = null;
  if (chainIdOrCaip2.includes(":")) {
    chainId = caip2ToLifiChainId(chainIdOrCaip2);
  } else {
    if (Object.keys(caip2Map).includes(chainIdOrCaip2)) {
      const caip2 = caip2Map[chainIdOrCaip2 as keyof typeof caip2Map];
      chainId = caip2ToLifiChainId(caip2);
    }
  }
  if (chainId === null) {
    chainId = chainIdOrCaip2;
  }

  // Check denylist cache first
  if (await tokenDenylistCache.isDenylisted(token, chainId)) {
    console.log(`Token ${chainId}-${token} is in denylist, skipping API call`);
    return null;
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
      const errorText = await res.text();
      console.error("LiFi API error:", res.status, errorText);
      
      // Check if this is a denylist error and cache it
      if (await tokenDenylistCache.handlePotentialDenylistError(errorText, token, chainId)) {
        console.log(`Token ${chainId}-${token} added to denylist due to error: ${errorText}`);
      }
      
      return null;
    }
    const data = await res.json();
    const parsed = LifiTokenSchema.parse(data);
    return parsed;
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.warn("getAnyToken error:", errorMessage);
    
    // Check if this is a denylist error and cache it
    if (await tokenDenylistCache.handlePotentialDenylistError(errorMessage, token, chainId)) {
      console.log(`Token ${chainId}-${token} added to denylist due to error: ${errorMessage}`);
    }
    
    return null;
  }
}

async function enrichTokenMetadata(
  token: z.infer<typeof TokenSchema>,
  network: (typeof SUPPORTED_NETWORKS)[number]
): Promise<TokenMetadata | null> {
  try {
    const address =
      token.tokenAddress || "0x0000000000000000000000000000000000000000";
    const cacheKey = `${address}-${network.chainId}`;

    // Check cache first
    const cachedMetadata = await tokenMetadataCache.get(cacheKey);
    if (cachedMetadata) {
      return cachedMetadata;
    }

    // If we have complete metadata from Alchemy, use and cache it
    if (
      token.tokenMetadata &&
      token.tokenMetadata.name &&
      token.tokenMetadata.symbol &&
      token.tokenMetadata.decimals &&
      token.tokenMetadata.logo
    ) {
      const metadata: TokenMetadata = {
        address: address,
        name: token.tokenMetadata.name,
        symbol: token.tokenMetadata.symbol,
        decimals: token.tokenMetadata.decimals,
        logoURI: token.tokenMetadata.logo || "",
        chainId: parseInt(network.chainId),
      };

      await tokenMetadataCache.set(cacheKey, metadata);
      return metadata;
    }

    // If metadata is incomplete, fetch from traditional source
    const metadata = await getAnyToken(getAddress(address), network.chainId);

    if (!metadata || !metadata.decimals) {
      console.error(
        `No metadata found for ${address} on chain ${network.chainId}`
      );
      return null;
    }

    const tokenMetadata: TokenMetadata = {
      address,
      name: metadata.name || "",
      symbol: metadata.symbol || "",
      decimals: metadata.decimals,
      logoURI: metadata.logoURI || "",
      chainId: parseInt(network.chainId),
    };

    // Cache the fetched metadata
    await tokenMetadataCache.set(cacheKey, tokenMetadata);
    return tokenMetadata;
  } catch (error) {
    console.error(
      `Error enriching metadata for token ${token.tokenAddress}:`,
      error
    );
    return null;
  }
}

export async function getTokensMetadata(
  addresses: string[],
  chainId: string
): Promise<Map<string, TokenMetadata>> {
  try {
    const metadataMap = new Map<string, TokenMetadata>();
    const addressesToFetch: string[] = [];

    await Promise.all(
      addresses.map(async (address) => {
        // Skip denylisted tokens
        if (await tokenDenylistCache.isDenylisted(address, chainId)) {
          console.log(`Skipping denylisted token: ${chainId}-${address}`);
          return;
        }

        const cacheKey = `${address}-${chainId}`;
        const cachedMetadata = await tokenMetadataCache.get(cacheKey);
        if (cachedMetadata) {
          metadataMap.set(cacheKey, cachedMetadata);
        } else {
          addressesToFetch.push(address);
        }
      })
    );

    if (addressesToFetch.length === 0) {
      return metadataMap;
    }

    const tokenPromises = addressesToFetch.map((address) => {
      return getAnyToken(getAddress(address), chainId);
    });

    const tokenResults = await Promise.all(tokenPromises);

    addressesToFetch.forEach(async (address, index) => {
      const metadata = tokenResults[index];
      if (!metadata) {
        console.error(`No metadata found for ${address}`);
        return;
      }

      if (!metadata.decimals) {
        console.error(`No decimals found for ${address}`);
        return;
      }

      const tokenMetadata: TokenMetadata = {
        address,
        name: metadata.name || "",
        symbol: metadata.symbol || "",
        decimals: metadata.decimals,
        logoURI: metadata.logoURI || "",
        chainId: parseInt(chainId),
      };

      const metadataKey = `${address}-${chainId}`;
      metadataMap.set(metadataKey, tokenMetadata);
      await tokenMetadataCache.set(metadataKey, tokenMetadata);
    });

    return metadataMap;
  } catch (error) {
    console.error("Error fetching tokens metadata:", error);
    throw error;
  }
}

export async function getTokenHoldings(
  address: string
): Promise<PortfolioItem[]> {
  try {
    const response = await fetch(
      `https://api.g.alchemy.com/data/v1/${import.meta.env.VITE_ALCHEMY_API_KEY}/assets/tokens/by-address`,
      {
        method: "POST",
        headers: {
          accept: "application/json",
          "content-type": "application/json",
        },
        body: JSON.stringify({
          addresses: [
            {
              address,
              networks: SUPPORTED_NETWORKS.map((n) => n.networkId),
            },
          ],
          withMetadata: true,
          withPrices: true,
          includeNativeTokens: true,
        }),
      }
    );

    const rawData = await response.json();
    const parsedData = AlchemyResponseSchema.safeParse(rawData);
    if (!parsedData.success) {
      console.error(
        "Error parsing Alchemy response:",
        parsedData.error,
        rawData
      );
      throw new Error("Error parsing Alchemy response");
    }

    const validatedData = parsedData.data;

    // First, filter out invalid tokens, then process them
    const preFilteredTokens = validatedData.data.tokens.filter((token) => {
      const balance = BigInt(token.tokenBalance);
      return balance > BigInt(0);
    });

    // Filter out denylisted tokens asynchronously
    const validTokenPromises = preFilteredTokens.map(async (token) => {
      // Don't filter out native tokens by denylist
      const isNativeToken =
        !token.tokenAddress ||
        token.tokenAddress === "0x0000000000000000000000000000000000000000";
      
      if (!isNativeToken) {
        // Filter out denylisted tokens
        const network = SUPPORTED_NETWORKS.find(n => n.networkId === token.network);
        if (network && token.tokenAddress && await tokenDenylistCache.isDenylisted(token.tokenAddress, network.chainId)) {
          console.log(`Filtering out denylisted token: ${network.chainId}-${token.tokenAddress}`);
          return null;
        }
      }

      return token;
    });

    const filteredTokens = (await Promise.all(validTokenPromises)).filter(token => token !== null);

    // Process metadata for valid tokens
    const validTokens = filteredTokens.map(async (token) => {
        const network = SUPPORTED_NETWORKS.find(
          (n) => n.networkId === token.network
        );
        if (!network) return null;

        const metadata = await enrichTokenMetadata(token, network);
        if (!metadata) return null;

        const rawBalance = BigInt(token.tokenBalance);
        const amount = Number(rawBalance) / Math.pow(10, metadata.decimals);

        return {
          metadata,
          amount,
          chain: network.chain,
          address:
            token.tokenAddress || "0x0000000000000000000000000000000000000000",
        };
      });

    const tokens = (await Promise.all(validTokens)).filter(
      (token): token is NonNullable<typeof token> => token !== null
    );

    // Fetch prices for all tokens
    const prices = await fetchTokenPrices(
      tokens.map((token) => ({
        address: token.address,
        chain: token.chain,
      }))
    );

    // Combine metadata with prices and filter out zero-value tokens
    return tokens
      .map((token) => {
        const priceData = prices.get(token.address);
        if (
          !priceData ||
          (priceData.price * token.amount).toFixed(2) === "0.00"
        ) {
          return null;
        }

        const portfolioItem: PortfolioItem = {
          ...token.metadata,
          amount: token.amount,
          chain: token.chain,
          price: priceData.price,
          priceChange24h: priceData.priceChange24h,
          logoURI: token.metadata.logoURI || "",
        };

        return portfolioItem;
      })
      .filter((item): item is PortfolioItem => item !== null);
  } catch (error) {
    console.error("Error fetching token holdings:", error);
    throw error;
  }
}
