import { TokenData, TokenMetadata } from "@/app/types";
import Image from "next/image";
import { useEffect, useState } from "react";
import { FaGlobe, FaTelegram, FaXTwitter } from "react-icons/fa6";
import { Chart } from "./Chart";

interface TokenTileProps {
  token: TokenData;
  index: number;
}

function Socials({ tokenMetadata }: { tokenMetadata: TokenMetadata | null }) {
  return (
    <div className="flex flex-row gap-2">
      {tokenMetadata?.mpl.ipfs_metadata?.twitter && (
        <a
          href={`https://twitter.com/${tokenMetadata?.mpl.ipfs_metadata?.twitter}`}
          target="_blank"
          rel="noopener noreferrer"
        >
          <FaXTwitter />
        </a>
      )}
      {tokenMetadata?.mpl.ipfs_metadata?.telegram && (
        <a
          href={`https://t.me/${tokenMetadata?.mpl.ipfs_metadata?.telegram}`}
          target="_blank"
          rel="noopener noreferrer"
        >
          <FaTelegram />
        </a>
      )}
      {tokenMetadata?.mpl.ipfs_metadata?.website && (
        <a
          href={`https://${tokenMetadata?.mpl.ipfs_metadata?.website}`}
          target="_blank"
          rel="noopener noreferrer"
        >
          <FaGlobe />
        </a>
      )}
    </div>
  );
}

function ChartModal({
  isOpen,
  onClose,
  mint,
}: {
  isOpen: boolean;
  onClose: () => void;
  mint: string;
}) {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />
      <div className="relative bg-black/40 border border-purple-500/20 w-[90vw] h-[80vh] rounded-xl p-6 backdrop-blur-sm">
        <button
          onClick={onClose}
          className="absolute top-4 right-4 text-purple-300/70 hover:text-purple-100 transition-colors"
        >
          ✕
        </button>
        <div className="w-full h-full">
          <Chart mint={mint} />
        </div>
      </div>
    </div>
  );
}

export function TokenTile({ token, index }: TokenTileProps) {
  const [metadata, setMetadata] = useState<TokenMetadata | null>(null);
  const [showChart, setShowChart] = useState(false);

  useEffect(() => {
    fetch(`https://api.listen-rs.com/metadata?mint=${token.pubkey}`)
      .then(async (res) => {
        if (!res.ok) {
          const text = await res.text();
          console.log(text);
          throw new Error(text || res.statusText);
        }
        return res.json();
      })
      .then((data) => setMetadata(data))
      .catch((err) => {
        console.error("Failed to fetch metadata:", err);
      });
  }, [token.pubkey]);

  return (
    <div>
      <div
        className="p-4 flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer"
        onClick={() => setShowChart(true)}
      >
        <div className="flex items-center space-x-4">
          <span className="text-gray-500 w-6">{index + 1}.</span>
          <div className="flex items-center space-x-3">
            {metadata?.mpl.ipfs_metadata?.image &&
              metadata.mpl.ipfs_metadata.image.startsWith("https://") && (
                <div className="w-8 h-8 relative rounded-full overflow-hidden">
                  <Image
                    src={metadata.mpl.ipfs_metadata.image}
                    alt={token.name}
                    fill
                    className="object-cover"
                  />
                </div>
              )}
            <div>
              <div className="font-medium">
                <a
                  href={`https://solscan.io/address/${token.pubkey}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-blue-500"
                >
                  {token.name}
                </a>
                {metadata?.mpl.symbol && (
                  <span className="ml-2 text-sm text-gray-500">
                    {metadata.mpl.symbol}
                  </span>
                )}
              </div>
              <Socials tokenMetadata={metadata} />
              <div className="text-sm text-gray-500">
                Price: ${token.lastPrice.toFixed(5)}
              </div>
            </div>
          </div>
        </div>
        <div className="text-right">
          <div className="flex flex-col">
            <span className="text-green-500 font-medium">
              +${token.buyVolume.toLocaleString()}
            </span>
            <span className="text-red-500 font-medium">
              -${token.sellVolume.toLocaleString()}
            </span>
          </div>
          <div className="text-sm text-gray-500">
            MC: ${(token.marketCap / 1e6).toFixed(1)}M
          </div>
          <div className="text-xs text-gray-400">
            {token.uniqueAddresses.size} traders
          </div>
        </div>
      </div>
      <ChartModal
        isOpen={showChart}
        onClose={() => setShowChart(false)}
        mint={token.pubkey}
      />
    </div>
  );
}
