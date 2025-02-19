import { useEffect, useState } from "react";
import { FaCheck, FaGlobe, FaTelegram, FaXTwitter } from "react-icons/fa6";
import { IoBarChart } from "react-icons/io5";
import { useModal } from "../contexts/ModalContext";
import { TokenData, TokenMetadata } from "../types/metadata";

interface TokenTileProps {
  token: TokenData;
  index: number;
}

function Socials({ tokenMetadata }: { tokenMetadata: TokenMetadata | null }) {
  return (
    <div className="flex flex-row gap-1 sm:gap-2">
      {tokenMetadata?.mpl.ipfs_metadata?.twitter && (
        <a
          href={tokenMetadata?.mpl.ipfs_metadata?.twitter}
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-blue-500"
        >
          <FaXTwitter size={12} className="sm:text-base" />
        </a>
      )}
      {tokenMetadata?.mpl.ipfs_metadata?.telegram && (
        <a
          href={tokenMetadata?.mpl.ipfs_metadata?.telegram}
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-blue-500"
        >
          <FaTelegram size={12} className="sm:text-base" />
        </a>
      )}
      {tokenMetadata?.mpl.ipfs_metadata?.website && (
        <a
          href={tokenMetadata?.mpl.ipfs_metadata?.website}
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-blue-500"
        >
          <FaGlobe size={12} className="sm:text-base" />
        </a>
      )}
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
  const { openChart } = useModal();
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
    fetch(`https://api.listen-rs.com/v1/adapter/metadata?mint=${token.pubkey}`)
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
      <div className="p-3 sm:p-4 flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-800">
        <div className="flex items-center space-x-2 sm:space-x-4">
          <span className="text-gray-500 text-sm sm:text-base w-4 sm:w-6">
            {index + 1}.
          </span>
          <div className="flex items-center space-x-2 sm:space-x-3">
            {metadata?.mpl.ipfs_metadata?.image &&
              metadata.mpl.ipfs_metadata.image.startsWith("https://") && (
                <div className="w-6 h-6 sm:w-8 sm:h-8 relative rounded-full overflow-hidden">
                  <img
                    src={metadata.mpl.ipfs_metadata.image}
                    alt={token.name}
                    className="w-full h-full object-cover"
                  />
                </div>
              )}
            <div>
              <div className="font-medium">
                <span className="inline-flex items-center text-sm sm:text-base">
                  <a
                    href={`https://solscan.io/address/${token.pubkey}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="hover:text-blue-500 truncate max-w-[120px] sm:max-w-none"
                  >
                    {token.name}
                  </a>
                  {metadata?.mpl.symbol && (
                    <span className="ml-1 sm:ml-2 text-xs sm:text-sm text-gray-500">
                      {metadata.mpl.symbol}
                    </span>
                  )}
                  <button
                    onClick={handleCopy}
                    className="ml-1 sm:ml-2 hover:text-blue-500"
                  >
                    {copied ? (
                      <FaCheck size={12} className="sm:text-base" />
                    ) : (
                      <CopyIcon />
                    )}
                  </button>
                  <button
                    onClick={() => openChart(token.pubkey)}
                    className="ml-1 sm:ml-2 hover:text-blue-500"
                  >
                    <IoBarChart size={14} className="sm:text-base" />
                  </button>
                </span>
              </div>
              <div className="hidden sm:block">
                <Socials tokenMetadata={metadata} />
              </div>
              <div className="text-xs sm:text-sm text-gray-500">
                ${token.lastPrice.toFixed(5)}
              </div>
            </div>
          </div>
        </div>
        <div className="text-right">
          <div className="flex flex-col">
            <span className="text-green-500 font-medium text-xs sm:text-base">
              +${token.buyVolume.toLocaleString()}
            </span>
            <span className="text-red-500 font-medium text-xs sm:text-base">
              -${token.sellVolume.toLocaleString()}
            </span>
          </div>
          <div className="text-xs sm:text-sm text-gray-500">
            MC: ${(token.marketCap / 1e6).toFixed(1)}M
          </div>
          <div className="text-[10px] sm:text-xs text-gray-400">
            {token.uniqueAddresses.size} traders
          </div>
        </div>
      </div>
    </div>
  );
}
