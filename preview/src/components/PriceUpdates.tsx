"use client";

import type { PriceUpdate, TokenData } from "@/app/types";
import { useEffect, useState } from "react";
import { TokenTile } from "./TokenTile";

export default function PriceUpdates() {
  const [latestUpdate, setLatestUpdate] = useState<PriceUpdate | null>(null);
  const [tokenMap, setTokenMap] = useState<Map<string, TokenData>>(new Map());

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

  const topTokens = Array.from(tokenMap.values())
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
        <h2 className="text-lg font-semibold p-4 border-b border-purple-500/20 text-purple-100 flex-shrink-0">
          Top Tokens by Volume
        </h2>
        <div className="divide-y divide-purple-500/20 overflow-y-auto">
          {topTokens.map((token, index) => (
            <TokenTile key={token.pubkey} token={token} index={index} />
          ))}
        </div>
      </div>
    </div>
  );
}
