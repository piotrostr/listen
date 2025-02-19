import { FaExternalLinkAlt } from "react-icons/fa";
import { DexScreenerPair } from "../types/dexscreener";

interface DexscreenerDisplayProps {
  pairs: DexScreenerPair[];
}

export const DexscreenerDisplay = ({ pairs }: DexscreenerDisplayProps) => {
  return (
    <div className="w-full space-y-2">
      {pairs.map((pair) => (
        <div
          key={pair.pairAddress}
          className="bg-gray-800/50 rounded-lg p-4 backdrop-blur-sm border border-opacity-20 border-blue-500"
        >
          <div className="flex justify-between items-start">
            <div>
              <div className="flex items-center gap-2">
                <span className="text-lg font-medium">
                  {pair.baseToken.name} ({pair.baseToken.symbol})
                </span>
                <a
                  href={pair.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-400 hover:text-blue-300"
                >
                  <FaExternalLinkAlt size={12} />
                </a>
              </div>
              <div className="text-gray-400 text-sm">
                {pair.dexId.toUpperCase()} • {pair.quoteToken.symbol} Pair
              </div>
            </div>
            <div className="text-right">
              <div className="text-lg font-medium">
                ${Number(pair.priceUsd).toFixed(6)}
              </div>
              <div className="text-gray-400 text-sm">
                {Number(pair.priceNative).toFixed(6)} {pair.quoteToken.symbol}
              </div>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4 mt-4">
            <div>
              <div className="text-gray-400 text-sm">Liquidity</div>
              <div className="font-medium">
                ${pair.liquidity.usd.toLocaleString()}
              </div>
            </div>
            <div>
              <div className="text-gray-400 text-sm">24h Volume</div>
              <div className="font-medium">
                ${pair.volume.h24.toLocaleString()}
              </div>
            </div>
          </div>

          {pair.labels && pair.labels.length > 0 && (
            <div className="mt-2 flex gap-2">
              {pair.labels.map((label) => (
                <span
                  key={label}
                  className="text-xs px-2 py-1 rounded-full bg-blue-500/20 text-blue-300"
                >
                  {label}
                </span>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  );
};
