import { useEffect, useState } from "react";
import { z } from "zod";
import { useModal } from "../contexts/ModalContext";
import { useToken, useWorldchainToken } from "../hooks/useToken";
import { TokenMetadata } from "../lib/types";

// Zod schema for token data
export const TopTokenSchema = z.object({
  name: z.string(),
  pubkey: z.string(),
  price: z.number(),
  market_cap: z.number(),
  volume_24h: z.number(),
  price_change_24h: z.number(),
  chain_id: z
    .union([z.string(), z.number()])
    .transform((val) => val?.toString())
    .optional()
    .nullable(),
});

export const TopTokensResponseSchema = z.array(TopTokenSchema);

type TopToken = z.infer<typeof TopTokenSchema>;

interface TopTokensDisplayProps {
  tokens: TopToken[];
}

const formatNumber = (num: number) => {
  if (num >= 1_000_000_000) {
    return `$${(num / 1_000_000_000).toFixed(1)}B`;
  } else if (num >= 1_000_000) {
    return `$${(num / 1_000_000).toFixed(1)}M`;
  } else if (num >= 1_000) {
    return `$${(num / 1_000).toFixed(1)}K`;
  }
  return `$${num.toFixed(1)}`;
};

const TokenTileSolana = ({ token }: { token: TopToken }) => {
  const [metadata, setMetadata] = useState<any>(null);
  const { openChart } = useModal();

  useEffect(() => {
    fetch(`https://api.listen-rs.com/v1/adapter/metadata?mint=${token.pubkey}`)
      .then(async (res) => {
        if (!res.ok) throw new Error(res.statusText);
        return res.json();
      })
      .then(setMetadata)
      .catch(console.error);
  }, [token.pubkey]);

  return (
    <div
      className="rounded-lg p-3 border border-[#2D2D2D] transition-colors bg-black/40 backdrop-blur-sm flex flex-col cursor-pointer"
      onClick={() => {
        openChart({
          mint: token.pubkey,
          chainId: token.chain_id?.toString() || "solana",
        });
      }}
    >
      <div className="flex items-center gap-2 mb-2">
        {metadata?.mpl?.ipfs_metadata?.image ? (
          <img
            src={metadata.mpl.ipfs_metadata.image.replace(
              "cf-ipfs.com",
              "ipfs.io"
            )}
            alt={token.name}
            className="w-8 h-8 rounded-full"
          />
        ) : (
          <div className="w-8 h-8 rounded-full bg-blue-500/20" />
        )}
        <div>
          <div className="flex items-center gap-2">
            <div
              onClick={(e) => {
                e.preventDefault();
                openChart({
                  mint: token.pubkey,
                  chainId: "solana",
                });
              }}
              className="font-medium hover:text-blue-400 truncate cursor-pointer"
            >
              {metadata?.mpl?.symbol || token.name}
            </div>
          </div>
          <div className="text-sm text-gray-500">
            ${token.price.toFixed(token.price < 0.01 ? 4 : 2)}
          </div>
        </div>
      </div>
      <div className="grid grid-cols-2 gap-2 text-sm">
        <div>
          <div className="font-medium">{formatNumber(token.market_cap)}</div>
        </div>
        <div>
          <div
            className={`font-medium ${token.price_change_24h >= 0 ? "text-green-500" : "text-red-500"} flex justify-end`}
          >
            {token.price_change_24h >= 0 ? "+" : ""}
            {token.price_change_24h.toFixed(1)}%
          </div>
        </div>
      </div>
    </div>
  );
};

const TokenTileEvm = ({ token }: { token: TopToken }) => {
  let tokenData: TokenMetadata | null = null;
  let isLoading = false;
  if (token.chain_id?.includes("world") || token.chain_id?.includes("480")) {
    const { data, isLoading: isLoadingWorldchain } = useWorldchainToken(
      token.pubkey
    );
    if (data) {
      tokenData = data;
      isLoading = isLoadingWorldchain;
    }
  } else {
    const { data, isLoading: isLoadingEvm } = useToken(
      token.pubkey,
      token.chain_id || undefined
    );
    if (data) {
      tokenData = data;
      isLoading = isLoadingEvm;
    }
  }
  const { openChart } = useModal();

  return (
    <div className="rounded-lg p-3 border border-[#2D2D2D] transition-colors bg-black/40 backdrop-blur-sm flex flex-col">
      <div className="flex items-center gap-2 mb-2">
        {!isLoading && tokenData?.logoURI ? (
          <img
            src={tokenData.logoURI}
            alt={token.name}
            className="w-8 h-8 rounded-full"
          />
        ) : (
          <div className="w-8 h-8 rounded-full bg-blue-500/20" />
        )}
        <div>
          <div className="flex items-center gap-2">
            <div
              onClick={(e) => {
                e.preventDefault();
                openChart({
                  mint: token.pubkey,
                  chainId: token.chain_id || undefined,
                });
              }}
              className="font-medium hover:text-blue-400 truncate cursor-pointer"
            >
              {tokenData?.symbol || token.name}
            </div>
          </div>
          <div className="text-sm text-gray-500">
            ${token.price.toFixed(token.price < 0.01 ? 4 : 2)}
          </div>
        </div>
      </div>
      <div className="grid grid-cols-2 gap-2 text-sm">
        <div>
          <div className="font-medium">
            {formatNumber(token.market_cap || 0)}
          </div>
        </div>
        <div>
          <div
            className={`font-medium ${token.price_change_24h >= 0 ? "text-green-500" : "text-red-500"} flex justify-end`}
          >
            {token.price_change_24h >= 0 ? "+" : ""}
            {token.price_change_24h.toFixed(1)}%
          </div>
        </div>
      </div>
    </div>
  );
};

const TokenTile = ({ token }: { token: TopToken }) => {
  // If chain_id exists and it's not Solana, use EVM tile
  if (token.chain_id) {
    return <TokenTileEvm token={token} />;
  }

  // Otherwise use Solana tile
  return <TokenTileSolana token={token} />;
};

export const TopTokensDisplay = ({ tokens }: TopTokensDisplayProps) => {
  return (
    <div className="container-query">
      <div
        className="grid grid-cols-2 gap-4 
        [@container(min-width:600px)]:grid-cols-4"
      >
        {tokens.map((token) => (
          <TokenTile key={token.pubkey} token={token} />
        ))}
      </div>
    </div>
  );
};
