import { useQuery } from "@tanstack/react-query";
import { z } from "zod";
import { imageMap } from "./util";

const IpfsMetadataSchema = z.object({
  createdOn: z.string().nullable().optional(),
  description: z.string().nullable().optional(),
  image: z.string().nullable().optional(),
  name: z.string(),
  symbol: z.string(),
  showName: z.boolean().nullable().optional(),
  twitter: z.string().nullable().optional(),
  website: z.string().nullable().optional(),
  telegram: z.string().nullable().optional(),
});

const MplTokenMetadataSchema = z.object({
  name: z.string(),
  symbol: z.string(),
  uri: z.string(),
  ipfs_metadata: IpfsMetadataSchema.nullable().optional(),
});

const SplTokenMetadataSchema = z.object({
  mint_authority: z.string().nullable().optional(),
  supply: z.number(),
  decimals: z.number(),
  is_initialized: z.boolean(),
  freeze_authority: z.string().nullable().optional(),
});

export const TokenMetadataSchema = z.object({
  mint: z.string(),
  mpl: MplTokenMetadataSchema,
  spl: SplTokenMetadataSchema,
});

export type TokenMetadata = z.infer<typeof TokenMetadataSchema>;

async function getSolanaTokenMetadata(mint: string): Promise<TokenMetadata> {
  const response = await fetch(
    `https://api.listen-rs.com/v1/adapter/metadata?mint=${mint}`
  );
  const data = await response.json();
  return TokenMetadataSchema.parse(data);
}

export const useSolanaToken = (mint: string) => {
  const { data, isLoading, error } = useQuery({
    queryKey: ["solana-token", mint],
    queryFn: () => getSolanaTokenMetadata(mint),
  });

  return { data, isLoading, error };
};

// special case
const solanaToken = (): TokenMetadata => ({
  mint: "So11111111111111111111111111111111111111112",
  mpl: {
    symbol: "SOL",
    name: "Solana",
    uri: imageMap.solana,
  },
  spl: {
    mint_authority: null,
    supply: 0,
    decimals: 9,
    is_initialized: true,
    freeze_authority: null,
  },
});

// dirty, wont cache this good but its cached server-side
export const useSolanaTokens = (mints: string[]) => {
  const { data, isLoading, error } = useQuery({
    queryKey: ["solana-tokens", mints],
    queryFn: async () => {
      // Filter out SOL mint to handle separately
      const nonSolMints = mints.filter(
        (mint) => mint !== "So11111111111111111111111111111111111111112"
      );

      // Fetch non-SOL tokens
      const tokens = await Promise.all(nonSolMints.map(getSolanaTokenMetadata));

      const tokenMap = tokens.reduce(
        (acc, token) => {
          // Create a deep copy of the token to avoid mutating the original
          const tokenWithFallback = { ...token };

          // Initialize ipfs_metadata if it doesn't exist
          if (!tokenWithFallback.mpl.ipfs_metadata) {
            tokenWithFallback.mpl.ipfs_metadata = {
              name: token.mpl.name,
              symbol: token.mpl.symbol,
              image: null,
            };
          }

          // Apply fallback image if needed
          if (
            !tokenWithFallback.mpl.ipfs_metadata.image &&
            token.mint in imageMap
          ) {
            tokenWithFallback.mpl.ipfs_metadata.image =
              imageMap[token.mint as keyof typeof imageMap];
          }

          acc[token.mint] = tokenWithFallback;
          return acc;
        },
        {} as Record<string, TokenMetadata>
      );

      // Add SOL token if it's in the requested mints
      if (mints.includes("So11111111111111111111111111111111111111112")) {
        tokenMap["So11111111111111111111111111111111111111112"] = solanaToken();
      }

      return tokenMap;
    },
  });

  return { data, isLoading, error };
};
