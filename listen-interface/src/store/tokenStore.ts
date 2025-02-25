import { create } from "zustand";
import type { TokenMarketData } from "../types/metadata";
import type { PriceUpdate } from "../types/price";

interface TokenState {
  latestUpdate: PriceUpdate | null;
  tokenMap: Map<string, TokenMarketData>;

  // Actions
  setLatestUpdate: (update: PriceUpdate) => void;
  updateTokenData: (data: PriceUpdate) => void;

  // Selectors
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

export const useTokenStore = create<TokenState>((set, get) => ({
  latestUpdate: null,
  tokenMap: new Map<string, TokenMarketData>(),

  setLatestUpdate: (update) => set({ latestUpdate: update }),

  updateTokenData: (data) => {
    if (!data.is_pump) return;

    set((state) => {
      const newMap = new Map(state.tokenMap);
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

      return {
        tokenMap: newMap,
        latestUpdate: data,
      };
    });
  },

  filterTokensByMarketCap: (tokens, filter) => {
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
  },

  filterAndSortTokens: (tokens, marketCapFilter, volumeFilter) => {
    const marketCapFiltered = get().filterTokensByMarketCap(
      tokens,
      marketCapFilter
    );

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
  },
}));
