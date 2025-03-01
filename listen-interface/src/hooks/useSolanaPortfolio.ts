import {
  AccountInfo,
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
} from "@solana/web3.js";
import { useQuery } from "@tanstack/react-query";
import { tokenMetadataCache } from "./localStorage";
import { Holding, PriceResponse, TokenMetadata } from "./types";
import { usePrivyWallets } from "./usePrivyWallet";
import { decodeTokenAccount } from "./util";

const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);

const connection = new Connection(
  import.meta.env?.VITE_RPC_URL ?? "https://api.mainnet-beta.solana.com"
);

export async function getHoldings(
  connection: Connection,
  owner: PublicKey
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

export async function fetchTokenMetadata(mint: string): Promise<TokenMetadata> {
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
      `https://api.jup.ag/price/v2?ids=${mints.join(",")}`
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

export const useSolanaPortfolio = () => {
  const { data: wallets } = usePrivyWallets();

  return useQuery({
    queryKey: ["portfolio", wallets?.solanaWallet.toString()],
    queryFn: async () => {
      const pubkey = new PublicKey(wallets!.solanaWallet);
      const WSOL_MINT = "So11111111111111111111111111111111111111112";
      const USDC_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

      // Get SOL balance and token holdings in parallel
      const [solBalance, holdings] = await Promise.all([
        connection.getBalance(pubkey),
        getHoldings(connection, pubkey),
      ]);

      // Ensure USDC is in the mints list even if not in holdings
      const existingMints = holdings.map((h) => h.mint);
      if (!existingMints.includes(USDC_MINT)) {
        holdings.push({
          mint: USDC_MINT,
          ata: "",
          amount: 0n,
        });
      }

      const mints = [WSOL_MINT, ...holdings.map((h) => h.mint)];

      // Get metadata and prices in parallel
      const [tokenMetadata, pricesResponse] = await Promise.all([
        Promise.all(mints.map(fetchTokenMetadata)),
        fetchPrices(mints),
      ]);

      // Create SOL portfolio item using Jupiter metadata
      const solMetadata = tokenMetadata[0];
      const solPortfolioItem = {
        address: WSOL_MINT,
        name: "Solana",
        symbol: "SOL",
        decimals: solMetadata.decimals,
        logoURI: solMetadata.logoURI,
        price: Number(pricesResponse.data[WSOL_MINT]?.price || 0),
        amount: solBalance / LAMPORTS_PER_SOL,
        daily_volume: solMetadata.volume24h || 0,
        chain: "solana",
      };

      // Combine SOL with other tokens
      const tokenPortfolioItems = holdings.map((holding, index) => {
        const metadata = tokenMetadata[index + 1]; // offset by 1 since SOL metadata is first
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
          chain: "solana",
        };
      });

      return [solPortfolioItem, ...tokenPortfolioItems];
    },
    enabled: !!wallets,
    staleTime: 10000, // 10 seconds
  });
};
