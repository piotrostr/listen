"use client";

import { useEffect, useState } from "react";
import type { TokenData } from "../types/metadata";
import type { PriceUpdate } from "../types/price";
import { TokenTile } from "./TokenTile";

export function PriceUpdates() {
  const [latestUpdate, setLatestUpdate] = useState<PriceUpdate | null>(null);
  const [tokenMap, setTokenMap] = useState<Map<string, TokenData>>(new Map());
  const [marketCapFilter, setMarketCapFilter] = useState<string>("all");

  useEffect(() => {
    const ws = new WebSocket("wss://api.listen-rs.com/ws");

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

  const topTokens = filterTokensByMarketCap(Array.from(tokenMap.values()))
    .sort((a, b) => b.buyVolume - a.buyVolume)
    .slice(0, 20);

  return (
    <div className="h-[calc(100vh-6rem)] flex flex-col space-y-2 overflow-hidden p-2 max-w-7xl mx-auto px-4">
      {/* Latest Update Section */}
      <div className="bg-black/40 backdrop-blur-sm border border-purple-500/20 rounded-xl flex-shrink-0">
        <div className="flex items-center">
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
      <div className="bg-black/40 backdrop-blur-sm border border-purple-500/20 rounded-xl shadow-lg flex-1 overflow-hidden flex flex-col min-h-0">
        <div className="p-4 border-b border-purple-500/20 flex justify-between items-center">
          <h2 className="text-lg font-semibold text-purple-100">
            Top Tokens by Volume
          </h2>
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
        <div className="divide-y divide-purple-500/20 overflow-y-auto">
          {topTokens.map((token, index) => (
            <TokenTile key={token.pubkey} token={token} index={index} />
          ))}
        </div>
      </div>
    </div>
  );
}
