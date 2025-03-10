import { FaExternalLinkAlt } from "react-icons/fa";
import { DexScreenerPair } from "../types/dexscreener";

interface DexscreenerDisplayProps {
  pairs: DexScreenerPair[];
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

export const DexscreenerDisplay = ({ pairs }: DexscreenerDisplayProps) => {
  return (
    <div className="rounded-lg px-2 py-1 lg:px-4 lg:py-3 my-2 backdrop-blur-sm border border-[#2D2D2D]">
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="text-xs text-white">
              <th className="text-left p-2 font-normal">Token</th>
              <th className="text-right p-2 font-normal w-[100px]">
                Liquidity
              </th>
              <th className="text-right p-2 font-normal w-[100px]">24h Vol</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-[#2D2D2D]">
            {pairs.map((pair) => (
              <tr key={pair.pairAddress}>
                <td className="p-2">
                  <div className="flex items-center gap-2">
                    <div className="flex flex-row gap-1 items-start mt-1">
                      <img
                        src={
                          "https://dd.dexscreener.com/ds-data/chains/" +
                          pair.chainId.toLowerCase() +
                          ".png"
                        }
                        alt={pair.chainId}
                        className="w-4 h-4 sm:w-5 sm:h-5 rounded-full"
                      />
                      <img
                        src={
                          "https://dd.dexscreener.com/ds-data/dexes/" +
                          pair.dexId.toLowerCase() +
                          ".png"
                        }
                        alt={pair.dexId}
                        className="w-4 h-4 sm:w-5 sm:h-5 rounded-full"
                      />
                    </div>
                    <div className="flex flex-col min-w-0">
                      <div className="flex items-center gap-1">
                        <span className="font-medium truncate">
                          {pair.baseToken.symbol}
                        </span>
                        <a
                          href={pair.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-blue-400 hover:text-blue-300 flex-shrink-0"
                        >
                          <FaExternalLinkAlt size={10} />
                        </a>
                      </div>
                      <div className="text-gray-400 text-[10px] sm:text-xs truncate">
                        {pair.dexId.toUpperCase().slice(0, 11)} •{" "}
                        {pair.quoteToken.symbol}
                      </div>
                    </div>
                  </div>
                </td>
                {pair.liquidity && (
                  <td className="p-2 text-right">
                    <div className="font-medium text-sm">
                      {formatNumber(pair.liquidity.usd)}
                    </div>
                  </td>
                )}
                {pair.volume && (
                  <td className="p-2 text-right">
                    <div className="font-medium text-sm">
                      {formatNumber(pair.volume.h24)}
                    </div>
                  </td>
                )}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
