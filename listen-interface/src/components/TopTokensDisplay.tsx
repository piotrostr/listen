import { z } from "zod";
import { useModal } from "../contexts/ModalContext";
import { useToken } from "../hooks/useToken";

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
  const { data: metadata, isLoading } = useToken(token.pubkey, "solana");
  const { openChart } = useModal();

  if (!metadata?.logoURI) {
    console.log(metadata, token);
  }

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
        {!isLoading && metadata?.logoURI ? (
          <img
            src={metadata.logoURI}
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
              {!isLoading ? metadata?.symbol || token.name : token.name}
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
  const { data: tokenData, isLoading } = useToken(
    token.pubkey,
    token.chain_id || undefined
  );
  const { openChart } = useModal();

  if (!tokenData?.logoURI && tokenData) {
    tokenData.logoURI = `https://dd.dexscreener.com/ds-data/tokens/${token.chain_id}/${token.pubkey}.png`;
  }

  if (!tokenData?.logoURI) {
    console.log(tokenData, token);
  }

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
  if (token.pubkey.startsWith("0x")) {
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
