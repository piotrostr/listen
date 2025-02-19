"use client";

import { useEffect, useState } from "react";
import type { TokenData } from "../types/metadata";
import type { PriceUpdate } from "../types/price";
import { TokenTile } from "./TokenTile";

export function PriceUpdates() {
  const [latestUpdate, setLatestUpdate] = useState<PriceUpdate | null>(null);
  const [tokenMap, setTokenMap] = useState<Map<string, TokenData>>(new Map());
  const [marketCapFilter, setMarketCapFilter] = useState<string>("all");
  const [volumeFilter, setVolumeFilter] = useState<"bought" | "sold" | "all">(
    "all"
  );

  useEffect(() => {
    const ws = new WebSocket("wss://api.listen-rs.com/v1/adapter/ws");

    ws.onmessage = (event) => {
      try {
        const data: PriceUpdate = JSON.parse(event.data);
        if (!data.is_pump) return;
        setLatestUpdate(data);

        setTokenMap((prevMap) => {
          const newMap = new Map(prevMap);
          const existing = newMap.get(data.pubkey);

          newMap.set(data.pubkey, {
            name: data.name,
            buyVolume:
              (existing?.buyVolume || 0) + (data.is_buy ? data.swap_amount : 0),
            sellVolume:
              (existing?.sellVolume || 0) +
              (!data.is_buy ? data.swap_amount : 0),
            lastPrice: data.price,
            lastUpdate: new Date(data.timestamp),
            marketCap: data.market_cap,
            uniqueAddresses: new Set([
              ...(existing?.uniqueAddresses || []),
              data.owner,
            ]),
            pubkey: data.pubkey,
          });

          return newMap;
        });
      } catch (error) {
        alert("Error parsing message: " + JSON.stringify(error));
      }
    };

    ws.onopen = () => {
      ws.send(
        JSON.stringify({
          action: "subscribe",
          mints: ["*"],
        })
      );
      console.log("WebSocket connection opened");
    };

    ws.onerror = (error) => {
      console.error("WebSocket failed:", error);
    };

    ws.onclose = () => {
      console.log("WebSocket connection closed");
    };

    return () => {
      ws.close();
    };
  }, []);

  const filterTokensByMarketCap = (tokens: TokenData[]) => {
    switch (marketCapFilter) {
      case "under1m":
        return tokens.filter((token) => token.marketCap < 1_000_000);
      case "1mTo10m":
        return tokens.filter(
          (token) =>
            token.marketCap >= 1_000_000 && token.marketCap < 10_000_000
        );
      case "10mTo100m":
        return tokens.filter(
          (token) =>
            token.marketCap >= 10_000_000 && token.marketCap < 100_000_000
        );
      case "over100m":
        return tokens.filter((token) => token.marketCap >= 100_000_000);
      default:
        return tokens;
    }
  };

  const filterAndSortTokens = (tokens: TokenData[]) => {
    const marketCapFiltered = filterTokensByMarketCap(tokens);

    switch (volumeFilter) {
      case "bought":
        return marketCapFiltered.sort((a, b) => {
          const netVolumeA = a.buyVolume - a.sellVolume;
          const netVolumeB = b.buyVolume - b.sellVolume;
          return netVolumeB - netVolumeA;
        });
      case "sold":
        return marketCapFiltered.sort((a, b) => {
          const netVolumeA = a.sellVolume - a.buyVolume;
          const netVolumeB = b.sellVolume - b.buyVolume;
          return netVolumeB - netVolumeA;
        });
      default:
        return marketCapFiltered.sort(
          (a, b) => b.buyVolume + b.sellVolume - (a.buyVolume + a.sellVolume)
        );
    }
  };

  const topTokens = filterAndSortTokens(Array.from(tokenMap.values())).slice(
    0,
    20
  );

  return (
    <div className="h-full flex flex-col gap-4 p-4">
      {/* Latest Update Section */}
      <div className="bg-black/40 backdrop-blur-sm border border-purple-500/20 rounded-xl p-4">
        <div className="flex items-center justify-between">
          {latestUpdate ? (
            <>
              <span className="text-purple-50 font-medium w-48 truncate">
                {latestUpdate.name}
              </span>
              <span className="text-blue-200 w-32 text-right">
                ${latestUpdate.price.toFixed(5)}
              </span>
              {latestUpdate.is_buy ? (
                <span className="text-green-500 w-32 text-right">
                  ${latestUpdate.swap_amount.toFixed(2)}
                </span>
              ) : (
                <span className="text-red-500 w-32 text-right">
                  ${latestUpdate.swap_amount.toFixed(2)}
                </span>
              )}
              <span className="text-sm text-purple-300/70 w-24 text-right">
                {new Date(latestUpdate.timestamp * 1000).toLocaleTimeString()}
              </span>
            </>
          ) : (
            <span className="text-purple-300/70">Waiting for updates...</span>
          )}
        </div>
      </div>

      {/* Top Tokens Section */}
      <div className="flex-1 bg-black/40 backdrop-blur-sm border border-purple-500/20 rounded-xl shadow-lg flex flex-col min-h-0">
        <div className="p-4 border-b border-purple-500/20 flex justify-between items-center">
          <div className="flex justify-between space-x-4 w-full">
            {/* Volume Filter */}
            <div className="grid grid-cols-2 gap-2">
              <button
                onClick={() =>
                  setVolumeFilter(volumeFilter === "bought" ? "all" : "bought")
                }
                className={`px-3 py-1 rounded-lg text-sm ${
                  volumeFilter === "bought"
                    ? "bg-purple-500/20 border-2 border-purple-500"
                    : "bg-black/40 border-2 border-purple-500/30"
                } hover:bg-purple-500/10 transition-all`}
              >
                🟢
              </button>
              <button
                onClick={() =>
                  setVolumeFilter(volumeFilter === "sold" ? "all" : "sold")
                }
                className={`px-3 py-1 rounded-lg text-sm ${
                  volumeFilter === "sold"
                    ? "bg-purple-500/20 border-2 border-purple-500"
                    : "bg-black/40 border-2 border-purple-500/30"
                } hover:bg-purple-500/10 transition-all`}
              >
                🔴
              </button>
            </div>

            {/* Market Cap Filter */}
            <div className="flex items-center space-x-2">
              <span className="text-purple-100">MC:</span>
              <select
                value={marketCapFilter}
                onChange={(e) => setMarketCapFilter(e.target.value)}
                className="bg-black/40 text-purple-100 border border-purple-500/20 rounded-lg px-3 py-1 text-sm focus:outline-none focus:border-purple-500"
              >
                <option value="all">All Market Caps</option>
                <option value="under1m">Under $1M</option>
                <option value="1mTo10m">$1M - $10M</option>
                <option value="10mTo100m">$10M - $100M</option>
                <option value="over100m">Over $100M</option>
              </select>
            </div>
          </div>
        </div>
        <div className="divide-y divide-purple-500/20 overflow-y-auto">
          {topTokens.map((token, index) => (
            <TokenTile key={token.pubkey} token={token} index={index} />
          ))}
        </div>
      </div>
    </div>
  );
}
