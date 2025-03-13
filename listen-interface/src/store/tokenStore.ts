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

        // Instead of skipping updates, we'll prevent recursive/nested updates
        // by tracking when we're in the middle of a set operation
        const isNested = (window as any).__currentlyUpdatingTokenData;
        if (isNested) {
          // Queue this update for the next tick to avoid nested updates
          setTimeout(() => {
            get().updateTokenData(data);
          }, 0);
          return;
        }

        try {
          (window as any).__currentlyUpdatingTokenData = true;

          set((state) => {
            const newMap = new Map(state.tokenMap);
            const existing = newMap.get(data.pubkey);

            newMap.set(data.pubkey, {
              name: data.name,
              buyVolume:
                (existing?.buyVolume || 0) +
                (data.is_buy ? data.swap_amount : 0),
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
        } finally {
          (window as any).__currentlyUpdatingTokenData = false;
        }
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
        const hiddenTokens = get().hiddenTokens;
        // Ensure hiddenTokens is a Set
        return hiddenTokens instanceof Set
          ? hiddenTokens.has(pubkey)
          : Array.isArray(hiddenTokens)
            ? (hiddenTokens as string[]).includes(pubkey)
            : false;
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
        const hiddenTokens = get().hiddenTokens;
        const showHiddenOnly = get().showHiddenOnly;
        const showWatchlistOnly = get().showWatchlistOnly;
        const watchlist = get().watchlist;

        // First apply watchlist/hidden filters
        let filteredTokens = tokens;

        if (showHiddenOnly) {
          // Show only hidden tokens
          filteredTokens = tokens.filter((token) => {
            if (hiddenTokens instanceof Set) {
              return hiddenTokens.has(token.pubkey);
            } else if (Array.isArray(hiddenTokens)) {
              return (hiddenTokens as string[]).includes(token.pubkey);
            }
            return false;
          });
        } else if (showWatchlistOnly) {
          // Show only watchlisted tokens
          filteredTokens = tokens.filter((token) => {
            if (watchlist instanceof Set) {
              return watchlist.has(token.pubkey);
            } else if (Array.isArray(watchlist)) {
              return (watchlist as string[]).includes(token.pubkey);
            }
            return false;
          });
        } else {
          // Normal mode: show non-hidden tokens
          filteredTokens = tokens.filter((token) => {
            if (hiddenTokens instanceof Set) {
              return !hiddenTokens.has(token.pubkey);
            } else if (Array.isArray(hiddenTokens)) {
              return !(hiddenTokens as string[]).includes(token.pubkey);
            }
            return true;
          });
        }

        const marketCapFiltered = get()
          .filterTokensByMarketCap(filteredTokens, marketCapFilter)
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
          // Ensure these are proper Set objects
          state.watchlist = new Set(
            Array.isArray(state.watchlist) ? state.watchlist : []
          );
          state.hiddenTokens = new Set(
            Array.isArray(state.hiddenTokens) ? state.hiddenTokens : []
          );

          console.log(
            "Store rehydrated, watchlist size:",
            state.watchlist.size,
            "hiddenTokens size:",
            state.hiddenTokens.size
          );
        }
      },
    }
  )
);
