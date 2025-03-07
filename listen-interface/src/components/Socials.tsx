import { useEffect, useState } from "react";
import { FaCheck, FaGlobe, FaTelegram, FaXTwitter } from "react-icons/fa6";
import { IoBarChart } from "react-icons/io5";
import { TokenMetadataRaw } from "../types/metadata";
import { CopyIcon } from "./CopyIcon";

export function Socials({
  tokenMetadata,
  pubkey,
  openChart,
}: {
  tokenMetadata: TokenMetadataRaw | null;
  pubkey: string;
  openChart?: (pubkey: string) => void;
}) {
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (copied) {
      setTimeout(() => setCopied(false), 1000);
    }
  }, [copied]);

  const handleCopy = () => {
    navigator.clipboard.writeText(pubkey);
    setCopied(true);
  };

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
      <div className="flex gap-1 sm:gap-2">
        <button onClick={handleCopy} className="hover:text-blue-500">
          {copied ? (
            <FaCheck size={12} className="sm:text-base" />
          ) : (
            <CopyIcon />
          )}
        </button>
        {openChart && (
          <button
            onClick={() => openChart(pubkey)}
            className="hover:text-blue-500"
          >
            <IoBarChart size={14} className="sm:text-base" />
          </button>
        )}
      </div>
    </div>
  );
}
