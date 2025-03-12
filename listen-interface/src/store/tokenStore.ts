import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { TokenMarketData } from "../types/metadata";
import type { PriceUpdate } from "../types/price";

interface TokenState {
  latestUpdate: PriceUpdate | null;
  tokenMap: Map<string, TokenMarketData>;
  watchlist: Set<string>;
  hiddenTokens: Set<string>;

  // UI state
  isListFrozen: boolean;
  showWatchlistOnly: boolean;
  showHiddenOnly: boolean;
  marketCapFilter: string;
  volumeFilter: "bought" | "sold" | "all";

  // Actions
  setLatestUpdate: (update: PriceUpdate) => void;
  updateTokenData: (data: PriceUpdate) => void;
  toggleWatchlist: (pubkey: string) => void;
  toggleHidden: (pubkey: string) => void;
  isWatchlisted: (pubkey: string) => boolean;
  isHidden: (pubkey: string) => boolean;

  // UI actions
  setIsListFrozen: (frozen: boolean) => void;
  setShowWatchlistOnly: (show: boolean) => void;
  setShowHiddenOnly: (show: boolean) => void;
  setMarketCapFilter: (filter: string) => void;
  setVolumeFilter: (filter: "bought" | "sold" | "all") => void;

  // Selectors
  filterTokensByMarketCap: (
    tokens: TokenMarketData[],
    filter: string
  ) => TokenMarketData[];
  filterAndSortTokens: (
    tokens: TokenMarketData[],
    marketCapFilter: string,
    volumeFilter: "bought" | "sold" | "all",
    limit?: number
  ) => TokenMarketData[];
}

export const useTokenStore = create<TokenState>()(
  persist(
    (set, get) => ({
      latestUpdate: null,
      tokenMap: new Map<string, TokenMarketData>(),
      watchlist: new Set<string>(),
      hiddenTokens: new Set<string>(),

      // UI state
      isListFrozen: false,
      showWatchlistOnly: false,
      showHiddenOnly: false,
      marketCapFilter: "all",
      volumeFilter: "all",

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

          return {
            tokenMap: newMap,
            latestUpdate: data,
          };
        });
      },

      toggleWatchlist: (pubkey) => {
        set((state) => {
          const newWatchlist = new Set(state.watchlist);
          if (newWatchlist.has(pubkey)) {
            newWatchlist.delete(pubkey);
          } else {
            newWatchlist.add(pubkey);
          }
          return { watchlist: newWatchlist };
        });
      },

      toggleHidden: (pubkey) => {
        set((state) => {
          const newHiddenTokens = new Set(state.hiddenTokens);
          if (newHiddenTokens.has(pubkey)) {
            newHiddenTokens.delete(pubkey);
          } else {
            newHiddenTokens.add(pubkey);
          }
          return { hiddenTokens: newHiddenTokens };
        });
      },

      isWatchlisted: (pubkey) => {
        return get().watchlist.has(pubkey);
      },

      isHidden: (pubkey) => {
        return get().hiddenTokens.has(pubkey);
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

      filterAndSortTokens: (
        tokens: TokenMarketData[],
        marketCapFilter: string,
        volumeFilter: "bought" | "sold" | "all",
        limit?: number
      ) => {
        // Filter out hidden tokens
        const visibleTokens = tokens.filter(
          (token) => !get().hiddenTokens.has(token.pubkey)
        );

        const marketCapFiltered = get()
          .filterTokensByMarketCap(visibleTokens, marketCapFilter)
          .filter((token) => (token.marketCap / 1e6).toFixed(1) !== "0.0");

        let result;
        switch (volumeFilter) {
          case "bought":
            result = marketCapFiltered.sort((a, b) => {
              const netVolumeA = a.buyVolume - a.sellVolume;
              const netVolumeB = b.buyVolume - b.sellVolume;
              return netVolumeB - netVolumeA;
            });
            break;
          case "sold":
            result = marketCapFiltered.sort((a, b) => {
              const netVolumeA = a.sellVolume - a.buyVolume;
              const netVolumeB = b.sellVolume - b.buyVolume;
              return netVolumeB - netVolumeA;
            });
            break;
          default:
            result = marketCapFiltered.sort(
              (a, b) =>
                b.buyVolume + b.sellVolume - (a.buyVolume + a.sellVolume)
            );
        }

        // Return only what's needed
        return limit ? result.slice(0, limit) : result;
      },

      // UI actions
      setIsListFrozen: (frozen) => set({ isListFrozen: frozen }),
      setShowWatchlistOnly: (show) => set({ showWatchlistOnly: show }),
      setShowHiddenOnly: (show) => set({ showHiddenOnly: show }),
      setMarketCapFilter: (filter) => set({ marketCapFilter: filter }),
      setVolumeFilter: (filter) => set({ volumeFilter: filter }),
    }),
    {
      name: "token-storage",
      partialize: (state) => ({
        watchlist: Array.from(state.watchlist),
        hiddenTokens: Array.from(state.hiddenTokens),
        // Optional: persist UI state too
        showWatchlistOnly: state.showWatchlistOnly,
        showHiddenOnly: state.showHiddenOnly,
        marketCapFilter: state.marketCapFilter,
        volumeFilter: state.volumeFilter,
      }),
      onRehydrateStorage: () => (state) => {
        if (state) {
          state.watchlist = new Set(state.watchlist || []);
          state.hiddenTokens = new Set(state.hiddenTokens || []);
        }
      },
    }
  )
);
