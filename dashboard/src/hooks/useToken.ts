import { useQuery } from "@tanstack/react-query";
import { z } from "zod";

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

const imageMap = {
  EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v:
    "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/logo.png",
};

export type TokenMetadata = z.infer<typeof TokenMetadataSchema>;

async function getSolanaTokenMetadata(mint: string): Promise<TokenMetadata> {
  const response = await fetch(
    `https://api.listen-rs.com/metadata?mint=${mint}`
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

// dirty, wont cache this good but its cached server-side
export const useSolanaTokens = (mints: string[]) => {
  const { data, isLoading, error } = useQuery({
    queryKey: ["solana-tokens", mints],
    queryFn: async () => {
      const tokens = await Promise.all(mints.map(getSolanaTokenMetadata));
      return tokens.reduce((acc, token) => {
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
      }, {} as Record<string, TokenMetadata>);
    },
  });

  return { data, isLoading, error };
};
