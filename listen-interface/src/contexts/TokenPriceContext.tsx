"use client";

import React, { createContext, useContext, useEffect, useState } from "react";
import type { TokenMarketData } from "../types/metadata";
import type { PriceUpdate } from "../types/price";

interface TokenPriceContextType {
  latestUpdate: PriceUpdate | null;
  tokenMap: Map<string, TokenMarketData>;
  filterTokensByMarketCap: (
    tokens: TokenMarketData[],
    filter: string
  ) => TokenMarketData[];
  filterAndSortTokens: (
    tokens: TokenMarketData[],
    marketCapFilter: string,
    volumeFilter: "bought" | "sold" | "all"
  ) => TokenMarketData[];
}

const TokenPriceContext = createContext<TokenPriceContextType | undefined>(
  undefined
);

export function TokenPriceProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const [latestUpdate, setLatestUpdate] = useState<PriceUpdate | null>(null);
  const [tokenMap, setTokenMap] = useState<Map<string, TokenMarketData>>(
    new Map()
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
        console.error("Error parsing message:", error);
      }
    };

    ws.onopen = () => {
      ws.send(
        JSON.stringify({
          action: "subscribe",
          mints: ["*"],
        })
      );
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

  const filterTokensByMarketCap = (
    tokens: TokenMarketData[],
    filter: string
  ) => {
    switch (filter) {
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

  const filterAndSortTokens = (
    tokens: TokenMarketData[],
    marketCapFilter: string,
    volumeFilter: "bought" | "sold" | "all"
  ) => {
    const marketCapFiltered = filterTokensByMarketCap(tokens, marketCapFilter);

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

  return (
    <TokenPriceContext.Provider
      value={{
        latestUpdate,
        tokenMap,
        filterTokensByMarketCap,
        filterAndSortTokens,
      }}
    >
      {children}
    </TokenPriceContext.Provider>
  );
}

export function useTokenPrice() {
  const context = useContext(TokenPriceContext);
  if (context === undefined) {
    throw new Error("useTokenPrice must be used within a TokenPriceProvider");
  }
  return context;
}
