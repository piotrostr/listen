import { useQuery } from "@tanstack/react-query";
import { AccountInfo, Connection, PublicKey } from "@solana/web3.js";
import { usePrivyWallets } from "./usePrivyWallet";
import { Holding, TokenMetadata, PriceResponse } from "./types";
import { tokenMetadataCache } from "./cache";
import { decodeTokenAccount } from "./util";

const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
);

const connection = new Connection(
  import.meta.env?.VITE_RPC_URL ?? "https://api.mainnet-beta.solana.com",
);

async function getHoldings(
  connection: Connection,
  owner: PublicKey,
): Promise<Holding[]> {
  const atas = await connection.getTokenAccountsByOwner(owner, {
    programId: TOKEN_PROGRAM_ID,
  });

  const holdings = atas.value
    .map((ata) => parseHolding(ata))
    .filter((holding): holding is Holding => {
      return holding !== null && holding.amount > 0n;
    });

  return holdings;
}

function parseHolding(ata: {
  pubkey: PublicKey;
  account: AccountInfo<Buffer>;
}): Holding | null {
  try {
    const parsedData = decodeTokenAccount(ata.account.data);
    if (!parsedData) return null;
    return {
      mint: parsedData.mint.toString(),
      ata: ata.pubkey.toString(),
      amount: parsedData.amount,
    };
  } catch (error) {
    console.error("Failed to parse holding:", error);
    return null;
  }
}

async function fetchTokenMetadata(mint: string): Promise<TokenMetadata> {
  try {
    // First check IndexedDB cache
    const cachedMetadata = await tokenMetadataCache.get(mint);
    if (cachedMetadata) {
      return cachedMetadata;
    }

    // If not in cache, fetch from API
    const response = await fetch(`https://tokens.jup.ag/token/${mint}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch metadata for ${mint}`);
    }
    const metadata = await response.json();

    // Store in IndexedDB
    await tokenMetadataCache.set(mint, metadata);

    return metadata;
  } catch (error) {
    console.error(`Error fetching metadata for ${mint}:`, error);
    throw error;
  }
}

async function fetchPrices(mints: string[]): Promise<PriceResponse> {
  try {
    const response = await fetch(
      `https://api.jup.ag/price/v2?ids=${mints.join(",")}`,
    );
    if (!response.ok) {
      throw new Error("Failed to fetch prices");
    }
    return response.json();
  } catch (error) {
    console.error("Error fetching prices:", error);
    throw error;
  }
}

export const usePortfolio = () => {
  const { data: wallets } = usePrivyWallets();

  return useQuery({
    queryKey: ["portfolio", wallets?.solanaWallet.toString()],
    queryFn: async () => {
      // Get holdings
      const holdings = await getHoldings(
        connection,
        new PublicKey(wallets!.solanaWallet),
      );
      const mints = holdings.map((h) => h.mint);

      // Get metadata and prices in parallel
      const [tokenMetadata, pricesResponse] = await Promise.all([
        Promise.all(mints.map(fetchTokenMetadata)),
        fetchPrices(mints),
      ]);

      // Combine data
      return holdings.map((holding, index) => {
        const metadata = tokenMetadata[index];
        const price = Number(pricesResponse.data[holding.mint]?.price || 0);
        const amount = Number(holding.amount) / Math.pow(10, metadata.decimals);

        return {
          address: metadata.address,
          name: metadata.name,
          symbol: metadata.symbol,
          decimals: metadata.decimals,
          logoURI: metadata.logoURI,
          price,
          amount,
          daily_volume: metadata.volume24h || 0,
        };
      });
    },
    enabled: !!wallets,
    staleTime: 10000, // 10 seconds
  });
};
