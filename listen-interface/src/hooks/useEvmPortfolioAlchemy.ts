import { Alchemy, Network } from "alchemy-sdk";
import { getAddress } from "viem";
import { tokenMetadataCache } from "./localStorage";
import { PortfolioItem, TokenMetadata } from "./types";
import { getAnyToken } from "./useToken";

const SUPPORTED_NETWORKS = [
  { network: Network.ARB_MAINNET, chainId: "42161", chain: "arbitrum" },
  { network: Network.BNB_MAINNET, chainId: "56", chain: "bsc" },
  { network: Network.BASE_MAINNET, chainId: "8453", chain: "base" },
] as const;

const alchemyClients = SUPPORTED_NETWORKS.map(
  ({ network, chainId, chain }) => ({
    client: new Alchemy({
      apiKey: import.meta.env.VITE_ALCHEMY_API_KEY,
      network,
    }),
    chainId,
    chain,
    network,
  })
);

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
    const allTokens: PortfolioItem[] = [];

    await Promise.all(
      alchemyClients.map(async ({ chainId, chain, client, network }) => {
        const { tokenBalances } = await client.core.getTokenBalances(address);

        // Get native token balance
        const nativeBalance = await client.core.getBalance(address);

        // Filter non-zero token balances
        const nonZeroBalances = tokenBalances.filter((token) => {
          const balance = BigInt(token.tokenBalance || "0");
          return balance > BigInt(0);
        });

        // Add native token if balance > 0
        if (nativeBalance.toBigInt() > BigInt(0)) {
          nonZeroBalances.push({
            contractAddress: "0x0000000000000000000000000000000000000000",
            tokenBalance: nativeBalance.toString(),
            error: null,
          });
        }

        // Get all token addresses including native token
        const tokenAddresses = nonZeroBalances.map(
          (token) => token.contractAddress
        );

        // Get metadata for all tokens
        const metadataMap = await getTokensMetadata(tokenAddresses, chainId);

        // Get prices for all tokens
        const priceData = await client.prices.getTokenPriceByAddress(
          tokenAddresses.map((address) => ({
            address,
            network,
          }))
        );

        // Process all tokens including native
        const tokens = nonZeroBalances
          .map((token) => {
            // Create the same compound key format used in caching
            const metadataKey = `${token.contractAddress}-${chainId}`;
            const metadata = metadataMap.get(metadataKey);
            if (!metadata) return null;

            const price =
              priceData.data.find((p) => p.address === token.contractAddress)
                ?.prices[0]?.value || 0;

            const rawBalance = BigInt(token.tokenBalance!);
            const amount = Number(rawBalance) / Math.pow(10, metadata.decimals);

            const portfolioItem: PortfolioItem = {
              ...metadata,
              price: Number(price),
              amount,
              chain,
              logoURI: metadata.logoURI || "",
            };

            return portfolioItem;
          })
          .filter((token): token is PortfolioItem => token !== null);

        allTokens.push(...tokens);
      })
    );

    return allTokens;
  } catch (error) {
    console.error("Error fetching token holdings:", error);
    throw error;
  }
}
