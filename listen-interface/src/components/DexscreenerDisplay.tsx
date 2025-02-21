import { FaExternalLinkAlt } from "react-icons/fa";
import { DexScreenerPair } from "../types/dexscreener";

interface DexscreenerDisplayProps {
  pairs: DexScreenerPair[];
}

export const DexscreenerDisplay = ({ pairs }: DexscreenerDisplayProps) => {
  return (
    <table className="w-full">
      <thead>
        <tr className="text-xs text-gray-400">
          <th className="text-left p-2 font-normal">Token</th>
          <th className="text-right p-2 font-normal w-[140px]">Liquidity</th>
          <th className="text-right p-2 font-normal w-[140px]">24h Volume</th>
        </tr>
      </thead>
      <tbody className="divide-y divide-blue-500/20">
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
                    className="w-5 h-5 rounded-full"
                  />
                  <img
                    src={
                      "https://dd.dexscreener.com/ds-data/dexes/" +
                      pair.dexId.toLowerCase() +
                      ".png"
                    }
                    alt={pair.dexId}
                    className="w-5 h-5 rounded-full"
                  />
                </div>
                <div className="flex flex-col">
                  <span className="font-medium">
                    {pair.baseToken.name} ({pair.baseToken.symbol})
                  </span>
                  <div className="text-gray-400 text-xs">
                    {pair.dexId.toUpperCase()} • {pair.quoteToken.symbol}
                  </div>
                </div>
                <a
                  href={pair.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-400 hover:text-blue-300"
                >
                  <FaExternalLinkAlt size={12} />
                </a>
              </div>
            </td>
            <td className="p-2 text-right">
              <div className="font-medium">
                ${pair.liquidity.usd.toLocaleString()}
              </div>
            </td>
            <td className="p-2 text-right">
              <div className="font-medium">
                ${pair.volume.h24.toLocaleString()}
              </div>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
};
