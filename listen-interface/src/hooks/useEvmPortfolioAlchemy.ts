import { getAddress } from "viem";
import { z } from "zod";
import { tokenMetadataCache } from "./localStorage";
import { PortfolioItem, TokenMetadata } from "./types";
import { getAnyToken } from "./useToken";

const SUPPORTED_NETWORKS = [
  { chainId: "1", networkId: "eth-mainnet", chain: "ethereum" },
  { chainId: "42161", networkId: "arb-mainnet", chain: "arbitrum" },
  { chainId: "56", networkId: "bnb-mainnet", chain: "bsc" },
  { chainId: "8453", networkId: "base-mainnet", chain: "base" },
] as const;

const TokenPriceSchema = z.object({
  currency: z.string(),
  value: z.string(),
  lastUpdatedAt: z.string(),
});

const TokenMetadataSchema = z.object({
  symbol: z.string().nullable(),
  decimals: z.number().nullable(),
  name: z.string().nullable(),
  logo: z.string().nullable(),
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

async function enrichTokenMetadata(
  token: z.infer<typeof TokenSchema>,
  network: (typeof SUPPORTED_NETWORKS)[number]
): Promise<TokenMetadata | null> {
  try {
    // If we have complete metadata from Alchemy, use it
    if (
      token.tokenMetadata &&
      token.tokenMetadata.name &&
      token.tokenMetadata.symbol &&
      token.tokenMetadata.decimals &&
      token.tokenMetadata.logo
    ) {
      return {
        address: token.tokenAddress || "",
        name: token.tokenMetadata.name,
        symbol: token.tokenMetadata.symbol,
        decimals: token.tokenMetadata.decimals,
        logoURI: token.tokenMetadata.logo || "",
        chainId: parseInt(network.chainId),
      };
    }

    // If token address is null (native token) or metadata is incomplete, fetch from traditional source
    const address =
      token.tokenAddress || "0x0000000000000000000000000000000000000000";
    const metadata = await getAnyToken(getAddress(address), network.chainId);

    if (!metadata || !metadata.decimals) {
      console.error(
        `No metadata found for ${address} on chain ${network.chainId}`
      );
      return null;
    }

    return {
      address,
      name: metadata.name || "",
      symbol: metadata.symbol || "",
      decimals: metadata.decimals,
      logoURI: metadata.logoURI || "",
      chainId: parseInt(network.chainId),
    };
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
    const validatedData = AlchemyResponseSchema.parse(rawData);

    const portfolioPromises = validatedData.data.tokens
      .filter((token) => {
        const balance = BigInt(token.tokenBalance);
        return balance > BigInt(0);
      })
      .map(async (token) => {
        const network = SUPPORTED_NETWORKS.find(
          (n) => n.networkId === token.network
        );
        if (!network) return null;

        const metadata = await enrichTokenMetadata(token, network);
        if (!metadata) return null;

        const rawBalance = BigInt(token.tokenBalance);
        const amount = Number(rawBalance) / Math.pow(10, metadata.decimals);
        const price = token.tokenPrices?.[0]?.value
          ? parseFloat(token.tokenPrices[0].value)
          : 0;

        if ((price * amount).toFixed(2) === "0.00") return null;

        const portfolioItem: PortfolioItem = {
          ...metadata,
          chain: network.chain,
          price,
          amount,
        };

        return portfolioItem;
      });

    const portfolioItems = await Promise.all(portfolioPromises);
    return portfolioItems.filter(
      (item): item is PortfolioItem => item !== null
    );
  } catch (error) {
    console.error("Error fetching token holdings:", error);
    throw error;
  }
}
