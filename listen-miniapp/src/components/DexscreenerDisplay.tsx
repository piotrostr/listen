import { useState } from "react";
import { useTranslation } from "react-i18next";
import { FaChevronDown, FaChevronUp, FaExternalLinkAlt } from "react-icons/fa";
import dexscreenerIcon from "../assets/icons/dexscreener.png";
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
  const { t } = useTranslation();
  const [isExpanded, setIsExpanded] = useState(false);

  const toggleExpand = () => {
    setIsExpanded(!isExpanded);
  };

  return (
    <div className="rounded-lg px-2 py-1 my-2 backdrop-blur-sm border border-[#2D2D2D] text-sm">
      <div
        className="flex items-center justify-between p-2 cursor-pointer"
        onClick={toggleExpand}
      >
        <div className="flex items-center gap-2">
          <img src={dexscreenerIcon} alt="Dexscreener" className="w-4 h-4" />
          <span className="font-medium">
            {t("tool_messages.dexscreener_search")}
          </span>
        </div>
        <span>
          {isExpanded ? <FaChevronUp size={14} /> : <FaChevronDown size={14} />}
        </span>
      </div>

      {isExpanded && (
        <div className="p-2">
          {pairs.length > 0 ? (
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="text-xs text-white">
                    <th className="text-left p-2 font-normal">
                      {t("dexscreener_display.token")}
                    </th>
                    <th className="text-right p-2 font-normal w-[100px]">
                      {t("dexscreener_display.liquidity")}
                    </th>
                    <th className="text-right p-2 font-normal w-[100px]">
                      {t("dexscreener_display.volume")}
                    </th>
                  </tr>
                </thead>
                <tbody>
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
          ) : (
            <div className="text-gray-400 text-sm py-2 text-center">
              {t("dexscreener_display.no_search_results")}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
