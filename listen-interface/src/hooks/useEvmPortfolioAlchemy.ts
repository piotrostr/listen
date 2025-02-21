import { useQuery } from "@tanstack/react-query";
import { Alchemy, Network } from "alchemy-sdk";
import { tokenMetadataCache } from "./cache";
import { PortfolioItem, TokenMetadata } from "./types";
import { usePrivyWallets } from "./usePrivyWallet";

const SUPPORTED_NETWORKS = [
  { network: Network.ARB_MAINNET, chain: "arb" },
  { network: Network.BNB_MAINNET, chain: "bsc" },
  { network: Network.BASE_MAINNET, chain: "base" },
] as const;

const alchemyClients = SUPPORTED_NETWORKS.map(({ network, chain }) => ({
  client: new Alchemy({
    apiKey: import.meta.env.VITE_ALCHEMY_API_KEY,
    network,
  }),
  chain,
  network,
}));

export async function getTokensMetadata(
  addresses: string[],
  client: Alchemy
): Promise<Map<string, TokenMetadata>> {
  try {
    const metadataMap = new Map<string, TokenMetadata>();
    const addressesToFetch: string[] = [];

    await Promise.all(
      addresses.map(async (address) => {
        const cachedMetadata = await tokenMetadataCache.get(address);
        if (cachedMetadata) {
          metadataMap.set(address, cachedMetadata);
        } else {
          addressesToFetch.push(address);
        }
      })
    );

    if (addressesToFetch.length === 0) {
      return metadataMap;
    }

    const metadataResults = await Promise.all(
      addressesToFetch.map((address) => client.core.getTokenMetadata(address))
    );

    addressesToFetch.forEach(async (address, index) => {
      const metadata = metadataResults[index];
      if (!metadata.decimals) {
        console.error(`No decimals found for ${address}`);
        return;
      }

      const tokenMetadata: TokenMetadata = {
        address,
        name: metadata.name || "",
        symbol: metadata.symbol || "",
        decimals: metadata.decimals,
        logoURI: metadata.logo || "",
      };

      metadataMap.set(address, tokenMetadata);
      await tokenMetadataCache.set(address, tokenMetadata);
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
      alchemyClients.map(async (alchemy) => {
        const { network, chain, client } = alchemy;

        try {
          const { tokenBalances } = await client.core.getTokenBalances(address);

          const nonZeroBalances = tokenBalances.filter((token) => {
            if (!token.tokenBalance) return false;
            const balance = BigInt(token.tokenBalance);
            return balance !== BigInt(0);
          });

          const tokenAddresses = nonZeroBalances.map(
            (token) => token.contractAddress
          );
          const metadataMap = await getTokensMetadata(tokenAddresses, client);

          const priceData = await client.prices.getTokenPriceByAddress(
            tokenAddresses.map((address) => ({
              address,
              network,
            }))
          );

          const tokens = nonZeroBalances.map((token) => {
            const metadata = metadataMap.get(token.contractAddress);
            if (!metadata) return null;

            const price =
              priceData.data.find((p) => p.address === token.contractAddress)
                ?.prices[0]?.value || "0";

            const rawBalance = BigInt(token.tokenBalance!);
            const amount = Number(rawBalance) / Math.pow(10, metadata.decimals);

            const portfolioItem: PortfolioItem = {
              ...metadata,
              price: Number(price),
              amount,
              chain,
            };

            return portfolioItem;
          });

          allTokens.push(
            ...tokens.filter((token): token is PortfolioItem => token !== null)
          );
        } catch (error) {
          console.error(`Error fetching tokens for ${chain}:`, error);
        }
      })
    );

    return allTokens;
  } catch (error) {
    console.error("Error fetching token holdings:", error);
    throw error;
  }
}

export function useEvmPortfolio() {
  const { data: wallets } = usePrivyWallets();
  const address = wallets?.evmWallet;

  return useQuery({
    queryKey: ["portfolio", address],
    queryFn: async () => {
      if (!address) throw new Error("No address provided");
      return getTokenHoldings(address);
    },
    enabled: !!address,
    staleTime: 30000,
  });
}
