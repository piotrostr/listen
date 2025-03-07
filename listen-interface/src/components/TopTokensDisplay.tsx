import { useEffect, useState } from "react";
import { z } from "zod";
import { useModal } from "../contexts/ModalContext";

// Zod schema for token data
export const TopTokenSchema = z.object({
  name: z.string(),
  pubkey: z.string(),
  price: z.number(),
  market_cap: z.number(),
  volume_24h: z.number(),
  price_change_24h: z.number(),
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

const TokenTile = ({ token }: { token: TopToken }) => {
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
    <div className="rounded-lg p-3 border border-blue-500/20 hover:border-blue-500/40 transition-colors">
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
        <div className="flex-1">
          <div className="flex items-center gap-2">
            <a
              onClick={(e) => {
                e.preventDefault();
                openChart(token.pubkey);
              }}
              href={`https://solscan.io/token/${token.pubkey}`}
              target="_blank"
              rel="noopener noreferrer"
              className="font-medium hover:text-blue-400 truncate cursor-pointer"
            >
              {metadata?.mpl?.symbol || token.name}
            </a>
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
            className={`font-medium ${token.price_change_24h >= 0 ? "text-green-500" : "text-red-500"}`}
          >
            {token.price_change_24h >= 0 ? "+" : ""}
            {token.price_change_24h.toFixed(1)}%
          </div>
        </div>
      </div>
    </div>
  );
};

export const TopTokensDisplay = ({ tokens }: TopTokensDisplayProps) => {
  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 gap-4">
      {tokens.map((token) => (
        <TokenTile key={token.pubkey} token={token} />
      ))}
    </div>
  );
};
