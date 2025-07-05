import {
  AccountInfo,
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
} from "@solana/web3.js";
import { fetchListenMetadata } from "./listen";
import { tokenMetadataCache } from "./localStorage";
import { fetchTokenPrices } from "./price";
import { Holding, PortfolioItem, TokenMetadata } from "./types";
import { decodeTokenAccount, imageMap } from "./util";

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
  const atas = await connection.getTokenAccountsByOwner(
    owner,
    {
      programId: TOKEN_PROGRAM_ID,
    },
    "processed"
  );

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

export async function fetchTokenMetadataLegacy(
  mint: string
): Promise<TokenMetadata> {
  try {
    // listen metadata is cached on server, could cache on client too here
    const metadataRaw = await fetchListenMetadata(mint);
    const USDC_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    const usdcImage = imageMap[USDC_MINT];
    const logoUri =
      mint === USDC_MINT ? usdcImage : metadataRaw.mpl.ipfs_metadata?.image;

    return {
      address: metadataRaw.mint,
      name: metadataRaw.mpl.name,
      symbol: metadataRaw.mpl.symbol,
      decimals: metadataRaw.spl.decimals,
      logoURI: logoUri ?? "",
      volume24h: 0,
      chainId: 1151111081099710,
    };
  } catch (error) {
    return await fetchTokenMetadataFromJupiter(mint);
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

  // Get metadata and prices in parallel
  const [tokenMetadata, prices] = await Promise.all([
    Promise.all(mints.map(fetchTokenMetadataFromJupiter)),
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
