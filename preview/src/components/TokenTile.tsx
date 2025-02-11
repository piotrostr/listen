import { TokenData, TokenMetadata } from "@/app/types";
import Image from "next/image";
import { useEffect, useState } from "react";

interface TokenTileProps {
  token: TokenData;
  index: number;
}

export function TokenTile({ token, index }: TokenTileProps) {
  const [metadata, setMetadata] = useState<TokenMetadata | null>(null);

  useEffect(() => {
    fetch(`/api/token-metadata/${token.pubkey}`)
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
    <div className="p-4 flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-800">
      <div className="flex items-center space-x-4">
        <span className="text-gray-500 w-6">{index + 1}.</span>
        <div className="flex items-center space-x-3">
          {metadata?.mpl.ipfs_metadata?.image && (
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
  );
}
