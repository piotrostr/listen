import {
  AccountInfo,
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
} from "@solana/web3.js";
import { tokenMetadataCache } from "./localStorage";
import { fetchTokenPrices } from "./price";
import { Holding, PortfolioItem, TokenMetadata } from "./types";
import { decodeTokenAccount } from "./util";
import { jupiterRateLimiter } from "./rateLimiter";

const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);

const TOKEN_2022_PROGRAM_ID = new PublicKey(
  "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
);

const connection = new Connection(
  import.meta.env?.VITE_RPC_URL ?? "https://api.mainnet-beta.solana.com"
);

export async function getHoldings(
  connection: Connection,
  owner: PublicKey
): Promise<Holding[]> {
  const [atas, atas2022] = await Promise.all([
    connection.getTokenAccountsByOwner(
      owner,
      {
        programId: TOKEN_PROGRAM_ID,
      },
      "processed"
    ),
    connection.getTokenAccountsByOwner(
      owner,
      {
        programId: TOKEN_2022_PROGRAM_ID,
      },
      "processed"
    ),
  ]);

  const holdings = [...atas.value, ...atas2022.value]
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

export async function fetchTokenMetadataFromJupiter(
  mint: string
): Promise<TokenMetadata> {
  try {
    // First check IndexedDB cache
    const cachedMetadata = await tokenMetadataCache.get(mint);
    if (cachedMetadata) {
      return cachedMetadata;
    }

    // If not in cache, fetch from API with rate limiting
    const data = await jupiterRateLimiter.execute(async () => {
      const response = await fetch(`https://lite-api.jup.ag/tokens/v2/search?query=${mint}`);
      if (!response.ok) {
        const error = new Error(`Failed to fetch metadata for ${mint}`);
        (error as any).status = response.status;
        throw error;
      }
      return await response.json();
    });

    // Extract the first result (should match the mint exactly)
    const tokenData = Array.isArray(data) ? data[0] : data;
    
    if (!tokenData) {
      throw new Error(`No metadata found for ${mint}`);
    }

    const metadata: TokenMetadata = {
      address: tokenData.id || mint,
      decimals: tokenData.decimals || 9,
      name: tokenData.name || "unknown",
      symbol: tokenData.symbol || "unknown",
      logoURI: tokenData.icon || "",
      volume24h: tokenData.stats24h?.buyVolume || 0,
      chainId: 1151111081099710,
    };

    // Store in IndexedDB
    await tokenMetadataCache.set(mint, metadata);

    return metadata;
  } catch (error) {
    console.error(`Error fetching metadata for ${mint}:`, error);
    return {
      address: mint,
      decimals: 9,
      name: "unknown",
      symbol: "unknown",
      logoURI: "",
      volume24h: 0,
      chainId: 1151111081099710,
    };
  }
}

async function fetchTokenMetadataBatch(
  mints: string[]
): Promise<TokenMetadata[]> {
  const results: TokenMetadata[] = [];
  const uncachedMints: { mint: string; index: number }[] = [];

  // Check cache first
  for (let i = 0; i < mints.length; i++) {
    const cachedMetadata = await tokenMetadataCache.get(mints[i]);
    if (cachedMetadata) {
      results[i] = cachedMetadata;
    } else {
      uncachedMints.push({ mint: mints[i], index: i });
    }
  }

  // Batch fetch uncached tokens using the new V2 API
  if (uncachedMints.length > 0) {
    const BATCH_SIZE = 20; // V2 API supports up to 100, but we'll be conservative
    
    for (let i = 0; i < uncachedMints.length; i += BATCH_SIZE) {
      const batch = uncachedMints.slice(i, i + BATCH_SIZE);
      const mints = batch.map(b => b.mint).join(',');
      
      try {
        const data = await jupiterRateLimiter.execute(async () => {
          const response = await fetch(`https://lite-api.jup.ag/tokens/v2/search?query=${mints}`);
          if (!response.ok) {
            const error = new Error(`Failed to fetch batch metadata`);
            (error as any).status = response.status;
            throw error;
          }
          return await response.json();
        });

        // Map results back to correct positions
        const tokenDataMap = new Map((Array.isArray(data) ? data : []).map(t => [t.id, t]));
        
        for (const { mint, index } of batch) {
          const tokenData = tokenDataMap.get(mint);
          
          if (tokenData) {
            const metadata: TokenMetadata = {
              address: tokenData.id || mint,
              decimals: tokenData.decimals || 9,
              name: tokenData.name || "unknown",
              symbol: tokenData.symbol || "unknown",
              logoURI: tokenData.icon || "",
              volume24h: tokenData.stats24h?.buyVolume || 0,
              chainId: 1151111081099710,
            };
            
            results[index] = metadata;
            // Cache the result
            await tokenMetadataCache.set(mint, metadata);
          } else {
            // Fallback for not found tokens
            results[index] = {
              address: mint,
              decimals: 9,
              name: "unknown",
              symbol: "unknown",
              logoURI: "",
              volume24h: 0,
              chainId: 1151111081099710,
            };
          }
        }
      } catch (error) {
        console.error('Batch fetch error:', error);
        // Fallback to individual fetches
        for (const { mint, index } of batch) {
          results[index] = await fetchTokenMetadataFromJupiter(mint);
        }
      }
    }
  }

  return results;
}

export const fetchPortfolio = async (
  address: string
): Promise<PortfolioItem[]> => {
  const pubkey = new PublicKey(address);
  const WSOL_MINT = "So11111111111111111111111111111111111111112";

  // Get SOL balance and token holdings in parallel
  const [solBalance, holdings] = await Promise.all([
    connection.getBalance(pubkey),
    getHoldings(connection, pubkey),
  ]);

  const mints = [WSOL_MINT, ...holdings.map((h) => h.mint)];

  // Get metadata and prices in parallel with batching for metadata
  const [tokenMetadata, prices] = await Promise.all([
    fetchTokenMetadataBatch(mints),
    fetchTokenPrices(mints.map((mint) => ({ address: mint, chain: "solana" }))),
  ]);

  const solMetadata = tokenMetadata[0];
  const solPrice = prices.get(WSOL_MINT);
  const solPortfolioItem: PortfolioItem = {
    address: WSOL_MINT,
    name: "Solana",
    symbol: "SOL",
    decimals: solMetadata.decimals,
    logoURI: solMetadata.logoURI || "",
    price: solPrice?.price || 0,
    priceChange24h: solPrice?.priceChange24h || 0,
    amount: solBalance / LAMPORTS_PER_SOL,
    chain: "solana",
  };

  // Combine SOL with other tokens
  const tokenPortfolioItems = holdings
    .map((holding, index) => {
      const metadata = tokenMetadata[index + 1]; // offset by 1 since SOL metadata is first
      const priceData = prices.get(holding.mint);
      const amount = Number(holding.amount) / Math.pow(10, metadata.decimals);

      if (!priceData || (priceData.price * amount).toFixed(2) === "0.00")
        return null;

      const portfolioItem: PortfolioItem = {
        address: metadata.address,
        name: metadata.name,
        symbol: metadata.symbol,
        decimals: metadata.decimals,
        logoURI: metadata.logoURI || "",
        price: priceData.price,
        priceChange24h: priceData.priceChange24h,
        amount,
        chain: "solana",
      };
      return portfolioItem;
    })
    .filter((item): item is PortfolioItem => item !== null);

  return [solPortfolioItem, ...tokenPortfolioItems];
};
