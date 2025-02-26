import { useEffect, useState } from "react";
import { FaCheck } from "react-icons/fa6";
import { z } from "zod";
import { Socials } from "./TokenTile";

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
    return `$${(num / 1_000_000_000).toFixed(2)}B`;
  } else if (num >= 1_000_000) {
    return `$${(num / 1_000_000).toFixed(2)}M`;
  } else if (num >= 1_000) {
    return `$${(num / 1_000).toFixed(2)}K`;
  }
  return `$${num.toFixed(2)}`;
};

const TokenTile = ({ token }: { token: TopToken }) => {
  const [metadata, setMetadata] = useState<any>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    fetch(`https://api.listen-rs.com/v1/adapter/metadata?mint=${token.pubkey}`)
      .then(async (res) => {
        if (!res.ok) throw new Error(res.statusText);
        return res.json();
      })
      .then(setMetadata)
      .catch(console.error);
  }, [token.pubkey]);

  const handleCopy = () => {
    navigator.clipboard.writeText(token.pubkey);
    setCopied(true);
    setTimeout(() => setCopied(false), 1000);
  };

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
              href={`https://solscan.io/token/${token.pubkey}`}
              target="_blank"
              rel="noopener noreferrer"
              className="font-medium hover:text-blue-400 truncate"
            >
              {metadata?.mpl?.symbol || token.name}
            </a>
            <Socials tokenMetadata={metadata} pubkey={token.pubkey} />
            <button
              onClick={handleCopy}
              className="hover:text-blue-400 hidden lg:block"
            >
              {copied ? (
                <FaCheck size={12} />
              ) : (
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  fill="none"
                  viewBox="0 0 24 24"
                  strokeWidth={1.5}
                  stroke="currentColor"
                  className="w-4 h-4"
                >
                  <path d="M15.75 17.25v3.375c0 .621-.504 1.125-1.125 1.125h-9.75a1.125 1.125 0 01-1.125-1.125V7.875c0-.621.504-1.125 1.125-1.125H6.75a9.06 9.06 0 011.5.124m7.5 10.376h3.375c.621 0 1.125-.504 1.125-1.125V11.25c0-4.46-3.243-8.161-7.5-8.876a9.06 9.06 0 00-1.5-.124H9.375c-.621 0-1.125.504-1.125 1.125v3.5m7.5 10.375H9.375a1.125 1.125 0 01-1.125-1.125v-9.25m12 6.625v-1.875a3.375 3.375 0 00-3.375-3.375h-1.5a1.125 1.125 0 01-1.125-1.125v-1.5a3.375 3.375 0 00-3.375-3.375H9.75" />
                </svg>
              )}
            </button>
          </div>
          <div className="text-sm text-gray-500">
            ${token.price.toFixed(token.price < 0.01 ? 4 : 2)}
          </div>
        </div>
      </div>
      <div className="grid grid-cols-2 gap-2 text-sm">
        <div>
          <div className="text-gray-500">Market Cap</div>
          <div className="font-medium">{formatNumber(token.market_cap)}</div>
        </div>
        <div>
          <div className="text-gray-500">24h Change</div>
          <div
            className={`font-medium ${token.price_change_24h >= 0 ? "text-green-500" : "text-red-500"}`}
          >
            {token.price_change_24h >= 0 ? "+" : ""}
            {token.price_change_24h.toFixed(2)}%
          </div>
        </div>
      </div>
    </div>
  );
};

export const TopTokensDisplay = ({ tokens }: TopTokensDisplayProps) => {
  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
      {tokens.map((token) => (
        <TokenTile key={token.pubkey} token={token} />
      ))}
    </div>
  );
};
