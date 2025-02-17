import { useEffect, useState } from "react";
import { FaCheck, FaGlobe, FaTelegram, FaXTwitter } from "react-icons/fa6";
import { IoBarChart } from "react-icons/io5";
import { TokenData, TokenMetadata } from "../types/metadata";
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
          href={tokenMetadata?.mpl.ipfs_metadata?.twitter}
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-blue-500"
        >
          <FaXTwitter />
        </a>
      )}
      {tokenMetadata?.mpl.ipfs_metadata?.telegram && (
        <a
          href={tokenMetadata?.mpl.ipfs_metadata?.telegram}
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-blue-500"
        >
          <FaTelegram />
        </a>
      )}
      {tokenMetadata?.mpl.ipfs_metadata?.website && (
        <a
          href={tokenMetadata?.mpl.ipfs_metadata?.website}
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-blue-500"
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

const CopyIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    viewBox="0 0 24 24"
    strokeWidth={1.5}
    stroke="currentColor"
    className="w-4 h-4 cursor-pointer"
  >
    <path d="M15.75 17.25v3.375c0 .621-.504 1.125-1.125 1.125h-9.75a1.125 1.125 0 01-1.125-1.125V7.875c0-.621.504-1.125 1.125-1.125H6.75a9.06 9.06 0 011.5.124m7.5 10.376h3.375c.621 0 1.125-.504 1.125-1.125V11.25c0-4.46-3.243-8.161-7.5-8.876a9.06 9.06 0 00-1.5-.124H9.375c-.621 0-1.125.504-1.125 1.125v3.5m7.5 10.375H9.375a1.125 1.125 0 01-1.125-1.125v-9.25m12 6.625v-1.875a3.375 3.375 0 00-3.375-3.375h-1.5a1.125 1.125 0 01-1.125-1.125v-1.5a3.375 3.375 0 00-3.375-3.375H9.75" />
  </svg>
);

export function TokenTile({ token, index }: TokenTileProps) {
  const [metadata, setMetadata] = useState<TokenMetadata | null>(null);
  const [showChart, setShowChart] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (copied) {
      setTimeout(() => setCopied(false), 1000);
    }
  }, [copied]);

  const handleCopy = () => {
    navigator.clipboard.writeText(token.pubkey);
    setCopied(true);
  };

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
      <div className="p-4 flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-800">
        <div className="flex items-center space-x-4">
          <span className="text-gray-500 w-6">{index + 1}.</span>
          <div className="flex items-center space-x-3">
            {metadata?.mpl.ipfs_metadata?.image &&
              metadata.mpl.ipfs_metadata.image.startsWith("https://") && (
                <div className="w-8 h-8 relative rounded-full overflow-hidden">
                  <img
                    src={metadata.mpl.ipfs_metadata.image}
                    alt={token.name}
                    className="w-full h-full object-cover"
                  />
                </div>
              )}
            <div>
              <div className="font-medium">
                <span className="inline-flex items-center">
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
                  <button
                    onClick={handleCopy}
                    className="ml-2 hover:text-blue-500"
                  >
                    {copied ? <FaCheck /> : <CopyIcon />}
                  </button>
                  <button
                    onClick={() => setShowChart(true)}
                    className="ml-2 hover:text-blue-500"
                  >
                    <IoBarChart />
                  </button>
                </span>
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
