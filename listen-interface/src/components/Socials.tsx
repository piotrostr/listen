import { useEffect, useState } from "react";
import {
  FaCheck,
  FaDiscord,
  FaGlobe,
  FaTelegram,
  FaXTwitter,
} from "react-icons/fa6";
import { IoBarChart } from "react-icons/io5";
import { ChartAsset } from "../contexts/ModalContext";
import { TokenMetadataRaw } from "../types/metadata";
import { CopyIcon } from "./CopyIcon";

type SocialLinks = {
  twitter?: string | null;
  telegram?: string | null;
  website?: string | null;
  discord?: string | null;
};

function isTokenMetadataRaw(metadata: any): metadata is TokenMetadataRaw {
  return metadata && "mpl" in metadata && "spl" in metadata;
}

export function Socials({
  tokenMetadata,
  pubkey,
  openChart,
  chainId,
}: {
  tokenMetadata: TokenMetadataRaw | SocialLinks | null;
  pubkey: string;
  openChart?: (asset: ChartAsset) => void;
  chainId?: number;
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

  // Extract social links based on metadata type
  const socialLinks: SocialLinks = isTokenMetadataRaw(tokenMetadata)
    ? {
        twitter: tokenMetadata.mpl.ipfs_metadata?.twitter,
        telegram: tokenMetadata.mpl.ipfs_metadata?.telegram,
        website: tokenMetadata.mpl.ipfs_metadata?.website,
      }
    : (tokenMetadata as SocialLinks) || {};

  return (
    <div className="flex flex-row gap-1 sm:gap-2">
      {socialLinks.twitter && (
        <a
          href={
            socialLinks.twitter.startsWith("http")
              ? socialLinks.twitter
              : `https://twitter.com/${socialLinks.twitter}`
          }
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-blue-500"
        >
          <FaXTwitter size={12} className="sm:text-base" />
        </a>
      )}
      {socialLinks.telegram && (
        <a
          href={
            socialLinks.telegram.startsWith("http")
              ? socialLinks.telegram
              : `https://t.me/${socialLinks.telegram}`
          }
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-blue-500"
        >
          <FaTelegram size={12} className="sm:text-base" />
        </a>
      )}
      {socialLinks.discord && (
        <a
          href={socialLinks.discord}
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-blue-500"
        >
          <FaDiscord size={12} className="sm:text-base" />
        </a>
      )}
      {socialLinks.website && (
        <a
          href={socialLinks.website}
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
            onClick={() =>
              openChart({
                mint: pubkey,
                chainId: chainId?.toString(),
              })
            }
            className="hover:text-blue-500"
          >
            <IoBarChart size={14} className="sm:text-base" />
          </button>
        )}
      </div>
    </div>
  );
}
