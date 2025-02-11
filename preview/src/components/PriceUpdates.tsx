"use client";

import type { PriceUpdate, TokenData } from "@/app/types";
import { useEffect, useState } from "react";
import { TokenTile } from "./TokenTile";

export default function PriceUpdates() {
  const [latestUpdate, setLatestUpdate] = useState<PriceUpdate | null>(null);
  const [tokenMap, setTokenMap] = useState<Map<string, TokenData>>(new Map());

  useEffect(() => {
    const eventSource = new EventSource("/api/price-updates");

    eventSource.onmessage = (event) => {
      const data: PriceUpdate = JSON.parse(event.data);
      setLatestUpdate(data);

      setTokenMap((prevMap) => {
        const newMap = new Map(prevMap);
        const existing = newMap.get(data.pubkey);

        newMap.set(data.pubkey, {
          name: data.name,
          buyVolume:
            (existing?.buyVolume || 0) + (data.is_buy ? data.swap_amount : 0),
          sellVolume:
            (existing?.sellVolume || 0) + (!data.is_buy ? data.swap_amount : 0),
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
    };

    eventSource.onerror = (error) => {
      console.error("EventSource failed:", error);
      eventSource.close();
    };

    return () => {
      eventSource.close();
    };
  }, []);

  const topTokens = Array.from(tokenMap.values())
    .sort((a, b) => b.buyVolume - a.buyVolume)
    .slice(0, 10);

  return (
    <div className="space-y-6">
      {/* Latest Update Section */}
      <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
        <h2 className="text-lg font-semibold mb-2">Latest Update</h2>
        {latestUpdate ? (
          <div className="space-y-1">
            <div className="font-medium">{latestUpdate.name}</div>
            <div>Price: ${latestUpdate.price.toFixed(5)}</div>
            <div>Amount: ${latestUpdate.swap_amount.toFixed(2)}</div>
            <div className="text-sm text-gray-500">
              {new Date(latestUpdate.timestamp).toLocaleString()}
            </div>
          </div>
        ) : (
          <div>Waiting for updates...</div>
        )}
      </div>

      {/* Top Tokens Section */}
      <div className="bg-white dark:bg-gray-900 rounded-lg shadow">
        <h2 className="text-lg font-semibold p-4 border-b dark:border-gray-800">
          Top 10 Tokens by Volume
        </h2>
        <div className="divide-y dark:divide-gray-800">
          {topTokens.map((token, index) => (
            <TokenTile key={token.pubkey} token={token} index={index} />
          ))}
        </div>
      </div>
    </div>
  );
}
