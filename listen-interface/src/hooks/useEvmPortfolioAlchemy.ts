import { useQuery } from "@tanstack/react-query";
import { Alchemy, Network } from "alchemy-sdk";
import { tokenMetadataCache } from "./cache";
import { PortfolioItem, TokenMetadata } from "./types";
import { usePrivyWallets } from "./usePrivyWallet";

const alchemy = new Alchemy({
  apiKey: import.meta.env.VITE_ALCHEMY_API_KEY,
  network: Network.ARB_MAINNET,
});

export async function getTokensMetadata(
  addresses: string[]
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
      addressesToFetch.map((address) => alchemy.core.getTokenMetadata(address))
    );

    addressesToFetch.forEach(async (address, index) => {
      const metadata = metadataResults[index];

      const tokenMetadata: TokenMetadata = {
        address,
        name: metadata.name || "",
        symbol: metadata.symbol || "",
        decimals: metadata.decimals || 18,
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
    // Get token balances
    const { tokenBalances } = await alchemy.core.getTokenBalances(address);

    // Filter out zero balances
    const nonZeroBalances = tokenBalances.filter((token) => {
      if (!token.tokenBalance) return false;
      const balance = BigInt(token.tokenBalance);
      return balance !== BigInt(0);
    });

    // Batch fetch metadata for all tokens
    const tokenAddresses = nonZeroBalances.map(
      (token) => token.contractAddress
    );
    const metadataMap = await getTokensMetadata(tokenAddresses);

    // Batch fetch prices
    const priceData = await alchemy.prices.getTokenPriceByAddress(
      tokenAddresses.map((address) => ({
        address,
        network: Network.ARB_MAINNET,
      }))
    );

    const tokens = nonZeroBalances.map((token) => {
      const metadata = metadataMap.get(token.contractAddress);
      if (!metadata) return null;

      const price =
        priceData.data.find((p) => p.address === token.contractAddress)
          ?.prices[0]?.value || "0";

      // Convert hex balance to decimal
      const rawBalance = BigInt(token.tokenBalance!);
      const amount = Number(rawBalance) / Math.pow(10, metadata.decimals);

      const portfolioItem: PortfolioItem = {
        ...metadata,
        price: Number(price),
        amount,
        chain: "arb",
      };

      return portfolioItem;
    });

    // Filter out null values and return
    return tokens.filter((token): token is PortfolioItem => token !== null);
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
