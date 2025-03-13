import { create } from "zustand";
import { persist } from "zustand/middleware";
import { PortfolioItem } from "../hooks/types";
import { getTokenHoldings as fetchEvmPortfolio } from "../hooks/useEvmPortfolioAlchemy";
import { fetchPortfolio as fetchSolanaPortfolio } from "../hooks/useSolanaPortfolio";
import { useSettingsStore } from "./settingsStore";
import { useTokenStore } from "./tokenStore";
import { useWalletStore } from "./walletStore";

export function getPortfolioTotalValue(assets: PortfolioItem[]): number {
  return assets.reduce((total, asset) => total + asset.price * asset.amount, 0);
}

// Stale time in milliseconds (data considered fresh for 30 seconds)
const STALE_TIME = 30 * 1000;

interface PortfolioState {
  // Data
  solanaAssetsMap: Map<string, PortfolioItem>; // mint => item
  evmAssetsMap: Map<string, PortfolioItem>; // address => item

  // Computed/derived values (can be accessed via selectors)
  getSolanaAssets: () => PortfolioItem[];
  getEvmAssets: () => PortfolioItem[];
  getCombinedPortfolio: () => PortfolioItem[];
  getPortfolioValue: () => number;

  // Status
  isLoading: boolean;
  error: Error | null;
  lastUpdated: number | null; // timestamp in ms

  // Actions
  fetchSolanaPortfolio: (address: string) => Promise<void>;
  fetchEvmPortfolio: (address: string) => Promise<void>;
  fetchAllPortfolios: () => Promise<void>;
  refreshPortfolio: () => Promise<void>;
  isFresh: () => boolean;
  initializePortfolioManager: () => void;
}

export const usePortfolioStore = create<PortfolioState>()(
  persist(
    (set, get) => ({
      // Initial data state using Maps
      solanaAssetsMap: new Map<string, PortfolioItem>(),
      evmAssetsMap: new Map<string, PortfolioItem>(),

      // Selectors to get arrays from maps when needed
      getSolanaAssets: () => Array.from(get().solanaAssetsMap.values()),
      getEvmAssets: () => Array.from(get().evmAssetsMap.values()),
      getCombinedPortfolio: () => {
        const chatType = useSettingsStore.getState().chatType;
        const solanaAssets = Array.from(get().solanaAssetsMap.values());

        return chatType === "solana"
          ? solanaAssets
          : [...solanaAssets, ...Array.from(get().evmAssetsMap.values())];
      },
      getPortfolioValue: () =>
        getPortfolioTotalValue(get().getCombinedPortfolio()),

      // Initial status
      isLoading: false,
      error: null,
      lastUpdated: null,

      // Check if data is still fresh
      isFresh: () => {
        const lastUpdated = get().lastUpdated;
        if (!lastUpdated) return false;

        const now = Date.now();
        return now - lastUpdated < STALE_TIME;
      },

      // Actions
      fetchSolanaPortfolio: async (address: string) => {
        if (!address) return;

        set((state) => ({
          isLoading: state.solanaAssetsMap.size === 0,
          error: null,
        }));

        try {
          const solanaAssets = await fetchSolanaPortfolio(address);

          // Convert array to map
          const solanaAssetsMap = new Map();
          solanaAssets.forEach((asset) => {
            solanaAssetsMap.set(asset.address, {
              ...asset,
              logoURI: asset.logoURI || "",
            });
          });

          set(() => ({
            solanaAssetsMap,
            isLoading: false,
            lastUpdated: Date.now(),
          }));

          // Update combined portfolio
          get().getCombinedPortfolio();
        } catch (error) {
          set({
            error: error as Error,
            isLoading: false,
          });
          console.error("Error fetching Solana portfolio:", error);
        }
      },

      fetchEvmPortfolio: async (address: string) => {
        if (!address) return;

        // Access chatType directly from settings store
        const chatType = useSettingsStore.getState().chatType;

        // Skip fetching EVM assets if we're in Solana-only mode
        if (chatType === "solana") {
          console.log("Skipping EVM fetch in Solana-only mode");
          return;
        }

        // Don't set isLoading if we already have data
        set((state) => ({
          isLoading:
            state.evmAssetsMap.size === 0 && state.solanaAssetsMap.size === 0,
          error: null,
        }));

        try {
          const evmAssets = await fetchEvmPortfolio(address);

          // Convert array to map
          const evmAssetsMap = new Map();
          evmAssets.forEach((asset) => {
            evmAssetsMap.set(asset.address, asset);
          });

          set(() => ({
            evmAssetsMap,
            isLoading: false,
            lastUpdated: Date.now(),
          }));

          get().getCombinedPortfolio();
        } catch (error) {
          set({
            error: error as Error,
            isLoading: false,
          });
          console.error("Error fetching EVM portfolio:", error);
        }
      },

      // Fetch portfolios using current wallet addresses
      fetchAllPortfolios: async () => {
        // Get addresses directly from wallet store
        const { solanaAddress, evmAddress } = useWalletStore.getState();
        const solAddr = solanaAddress || "";
        const evmAddr = evmAddress || "";

        if (!solAddr && !evmAddr) return;

        // Access chatType directly from settings store
        const chatType = useSettingsStore.getState().chatType;

        // Set loading state at the beginning
        set({ isLoading: true, error: null });

        try {
          const fetchPromises = [];

          // Always fetch Solana portfolio if solanaAddress is provided
          if (solAddr) {
            fetchPromises.push(get().fetchSolanaPortfolio(solAddr));
          }

          // Only fetch EVM portfolio if chatType is "omni" and evmAddress is provided
          if (evmAddr && chatType === "omni") {
            fetchPromises.push(get().fetchEvmPortfolio(evmAddr));
          }

          // Wait for all portfolio fetches to complete
          await Promise.all(fetchPromises);

          set(() => ({
            lastUpdated: Date.now(),
            isLoading: false,
          }));
        } catch (error) {
          set({
            error: error as Error,
            isLoading: false,
          });
          console.error("Error fetching portfolios:", error);
        }
      },

      // Refresh portfolio data - always forces a refresh
      refreshPortfolio: async () => {
        // Access chatType directly from settings store
        const chatType = useSettingsStore.getState().chatType;
        console.log("Force refreshing portfolio, chatType:", chatType);

        // Reset data first to ensure UI shows loading state and indicate force refresh
        set({
          isLoading: true,
          error: null,
        });

        // Fetch portfolios with current wallet addresses
        await get().fetchAllPortfolios();
      },

      // Initialize visibility listener and other portfolio management
      initializePortfolioManager: () => {
        // Don't add multiple event listeners
        const visibilityListenerAlreadyAdded = Boolean(
          (window as any).__portfolioVisibilityListenerAdded
        );

        if (!visibilityListenerAlreadyAdded) {
          console.log("Adding portfolio visibility listener");

          // Function to handle visibility change
          const handleVisibilityChange = () => {
            if (document.visibilityState === "visible") {
              // On becoming visible, check if data is fresh
              if (!get().isFresh()) {
                console.log("Data is stale, refreshing");
                get().refreshPortfolio();
              } else {
                console.log("Data is fresh, no refresh needed");
              }
            }
          };

          // Add visibility listener
          document.addEventListener("visibilitychange", handleVisibilityChange);

          // Mark that we've added the listener
          (window as any).__portfolioVisibilityListenerAdded = true;
        }

        // Initial fetch if needed - run this only if we have no data
        if (get().solanaAssetsMap.size === 0 && !get().isLoading) {
          console.log("Initial portfolio load");
          get().refreshPortfolio();
        }

        // Prevent multiple subscriptions
        if ((window as any).__portfolioTokenStoreUnsubscribe) {
          (window as any).__portfolioTokenStoreUnsubscribe();
        }

        // Subscribe to tokenStore updates - handle directly in the callback
        const unsubscribe = useTokenStore.subscribe((state, prevState) => {
          // Only proceed if we have a new update and we're not loading
          if (
            state.latestUpdate !== prevState.latestUpdate &&
            state.latestUpdate &&
            !get().isLoading
          ) {
            const latestUpdate = state.latestUpdate;
            const mintToUpdate = latestUpdate.pubkey;

            // Skip token if not in portfolio (using our efficient HashMap lookup)
            const portfolioItem = get().solanaAssetsMap.get(mintToUpdate);
            if (!portfolioItem) return;

            // Skip if already processing an update (prevents recursive updates)
            if ((window as any).__updatingSolanaTokenPrices) return;

            try {
              (window as any).__updatingSolanaTokenPrices = true;

              // Get token data for the price
              const tokenData = state.tokenMap.get(mintToUpdate);
              if (!tokenData) return;

              set((currentState) => {
                // Create new map (immutable update for React)
                const updatedMap = new Map(currentState.solanaAssetsMap);

                // Update just this one token
                updatedMap.set(mintToUpdate, {
                  ...portfolioItem,
                  price: tokenData.lastPrice,
                });

                return { solanaAssetsMap: updatedMap };
              });
            } finally {
              // Clear the guard immediately
              (window as any).__updatingSolanaTokenPrices = false;
            }
          }
        });

        // Store the unsubscribe function to prevent memory leaks
        (window as any).__portfolioTokenStoreUnsubscribe = unsubscribe;
      },
    }),
    {
      name: "portfolio-storage",
      partialize: (state) => ({
        // Convert maps to arrays for storage
        solanaAssets: Array.from(state.solanaAssetsMap.values()),
        evmAssets: Array.from(state.evmAssetsMap.values()),
        lastUpdated: state.lastUpdated,
      }),
      onRehydrateStorage: (
        state: PortfolioState & {
          solanaAssets?: PortfolioItem[];
          evmAssets?: PortfolioItem[];
        }
      ) => {
        if (state) {
          // Convert arrays back to maps
          if (Array.isArray(state.solanaAssets)) {
            state.solanaAssetsMap = new Map(
              state.solanaAssets.map((asset) => [asset.address, asset])
            );
          }
          if (Array.isArray(state.evmAssets)) {
            state.evmAssetsMap = new Map(
              state.evmAssets.map((asset) => [asset.address, asset])
            );
          }
        }
      },
    }
  )
);
