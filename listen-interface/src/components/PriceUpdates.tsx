"use client";

import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { FaCircle } from "react-icons/fa";
import { FaPause } from "react-icons/fa6";
import { setupWebSocket } from "../services/websocketService";
import { useTokenStore } from "../store/tokenStore";
import { TokenTile } from "./TokenTile";

export function PriceUpdates() {
  const { tokenMap, filterAndSortTokens } = useTokenStore();
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

  const { t } = useTranslation();

  const filterHeader = (
    <div className="flex items-center justify-between gap-2 h-full">
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

      <div className="flex items-center gap-2 flex-1 justify-end">
        {isListFrozen && (
          <div className="flex items-center gap-1 bg-black/60 border border-teal-400/30 rounded px-2 py-0.5 text-xs text-teal-300">
            <FaPause className="text-teal-300 text-[10px]" />{" "}
            {t("price_updates.paused")}
          </div>
        )}
        <span className="text-purple-100 text-sm hidden sm:inline">
          {t("price_updates.market_cap")}:
        </span>
        <select
          value={marketCapFilter}
          onChange={(e) => setMarketCapFilter(e.target.value)}
          className="bg-black/40 text-purple-100 border border-purple-500/20 rounded-lg px-2 py-1 text-sm focus:outline-none focus:border-purple-500 w-[120px]"
        >
          <option value="all">{t("price_updates.all")}</option>
          <option value="under1m">&lt;$1M</option>
          <option value="1mTo10m">$1M-$10M</option>
          <option value="10mTo100m">$10M-$100M</option>
          <option value="over100m">&gt;$100M</option>
        </select>
      </div>
    </div>
  );

  return (
    <div className="h-full font-mono overflow-y-auto scrollbar-thin scrollbar-thumb-purple-500/30 scrollbar-track-transparent">
      <div className="h-[64px] p-3 border-b border-purple-500/20">
        {filterHeader}
      </div>
      <div onMouseEnter={handleMouseEnter} onMouseLeave={handleMouseLeave}>
        {topTokens.map((token) => (
          <TokenTile key={token.pubkey} token={token} />
        ))}
      </div>
    </div>
  );
}
