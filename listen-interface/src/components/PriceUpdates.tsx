"use client";

import { useEffect, useMemo, useState } from "react";
import { FaCircle } from "react-icons/fa";
import { FaPause } from "react-icons/fa6";
import { setupWebSocket } from "../services/websocketService";
import { useTokenStore } from "../store/tokenStore";
import { TokenTile } from "./TokenTile";

export function PriceUpdates() {
  const { latestUpdate, tokenMap, filterAndSortTokens } = useTokenStore();
  const [marketCapFilter, setMarketCapFilter] = useState<string>("all");
  const [volumeFilter, setVolumeFilter] = useState<"bought" | "sold" | "all">(
    "all"
  );
  const [isListFrozen, setIsListFrozen] = useState(false);
  const [frozenTokens, setFrozenTokens] = useState<any[]>([]);

  // Setup WebSocket connection
  useEffect(() => {
    const ws = setupWebSocket();
    return () => {
      ws.close();
    };
  }, []);

  // Get the current tokens based on filters
  const currentTokens = useMemo(() => {
    return filterAndSortTokens(
      Array.from(tokenMap.values()),
      marketCapFilter,
      volumeFilter
    ).slice(0, 20);
  }, [tokenMap, marketCapFilter, volumeFilter, filterAndSortTokens]);

  // Keep frozen tokens updated with current tokens when not frozen
  useEffect(() => {
    if (!isListFrozen) {
      setFrozenTokens(currentTokens);
    }
  }, [currentTokens, isListFrozen]);

  // Use frozen tokens when list is frozen, otherwise use current tokens
  const topTokens = isListFrozen ? frozenTokens : currentTokens;

  // Handlers for mouse events
  const handleMouseEnter = () => {
    setIsListFrozen(true);
  };

  const handleMouseLeave = () => {
    setIsListFrozen(false);
  };

  return (
    <div className="flex flex-col gap-2 p-2 sm:p-4 overflow-hidden h-full">
      {/* Latest Update Section */}
      <div className="h-[52px] bg-black/40 backdrop-blur-sm border border-purple-500/20 rounded-xl p-3 flex items-center">
        {latestUpdate ? (
          <div className="flex flex-row w-full text-sm">
            <span className="text-purple-300/70 w-18 text-left mr-2">
              {latestUpdate.slot}
            </span>
            <span className="text-white w-28 text-left">
              {latestUpdate.name.slice(0, 10)}
            </span>
            <span className="text-blue-200 w-16 text-left mr-2">
              ${latestUpdate.price.toFixed(5)}
            </span>
            {latestUpdate.is_buy ? (
              <span className="text-green-500 w-16 text-right">
                +${latestUpdate.swap_amount.toFixed(2)}
              </span>
            ) : (
              <span className="text-red-500 w-16 text-right">
                -${latestUpdate.swap_amount.toFixed(2)}
              </span>
            )}
          </div>
        ) : (
          <span className="text-purple-300/70 text-sm w-full text-center">
            Waiting for updates...
          </span>
        )}
      </div>

      {/* Top Tokens Section */}
      <div className="flex-1 bg-black/40 backdrop-blur-sm border border-purple-500/20 rounded-xl shadow-lg flex flex-col min-h-0">
        <div className="h-[64px] shrink-0 p-3 border-b border-purple-500/20">
          <div className="flex items-center justify-between gap-2 h-full">
            {/* Volume Filter */}
            <div className="flex gap-2">
              <button
                onClick={() =>
                  setVolumeFilter(volumeFilter === "bought" ? "all" : "bought")
                }
                className={`w-8 h-8 rounded-lg text-sm flex items-center justify-center ${
                  volumeFilter === "bought"
                    ? "bg-purple-500/20 border-2 border-purple-500"
                    : "bg-black/40 border-2 border-purple-500/30"
                } hover:bg-purple-500/10 transition-all`}
              >
                <FaCircle className="text-green-500" />
              </button>
              <button
                onClick={() =>
                  setVolumeFilter(volumeFilter === "sold" ? "all" : "sold")
                }
                className={`w-8 h-8 rounded-lg text-sm flex items-center justify-center ${
                  volumeFilter === "sold"
                    ? "bg-purple-500/20 border-2 border-purple-500"
                    : "bg-black/40 border-2 border-purple-500/30"
                } hover:bg-purple-500/10 transition-all`}
              >
                <FaCircle className="text-red-500" />
              </button>
            </div>

            {/* Market Cap Filter */}
            <div className="flex items-center gap-2 flex-1 justify-end">
              {isListFrozen && (
                <div className="flex items-center gap-1 bg-black/60 border border-teal-400/30 rounded px-2 py-0.5 text-xs text-teal-300">
                  <FaPause className="text-teal-300 text-[10px]" /> PAUSED
                </div>
              )}
              <span className="text-purple-100 text-sm hidden sm:inline">
                Market Cap:
              </span>
              <select
                value={marketCapFilter}
                onChange={(e) => setMarketCapFilter(e.target.value)}
                className="bg-black/40 text-purple-100 border border-purple-500/20 rounded-lg px-2 py-1 text-sm focus:outline-none focus:border-purple-500 w-[120px]"
              >
                <option value="all">Any</option>
                <option value="under1m">&lt;$1M</option>
                <option value="1mTo10m">$1M-$10M</option>
                <option value="10mTo100m">$10M-$100M</option>
                <option value="over100m">&gt;$100M</option>
              </select>
            </div>
          </div>
        </div>
        <div
          className="divide-y divide-purple-500/20 overflow-y-auto flex-1"
          onMouseEnter={handleMouseEnter}
          onMouseLeave={handleMouseLeave}
        >
          {topTokens.map((token, index) => (
            <TokenTile key={token.pubkey} token={token} index={index} />
          ))}
        </div>
      </div>
    </div>
  );
}
