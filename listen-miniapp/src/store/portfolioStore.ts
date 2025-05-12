import { create } from "zustand";
import { persist } from "zustand/middleware";
import { getTokenHoldings as fetchEvmPortfolio } from "../lib/evmPortfolio";
import { fetchPortfolio as fetchSolanaPortfolio } from "../lib/solanaPortfolio";
import { PortfolioItem } from "../lib/types";
import { useTokenStore } from "./tokenStore";
import { ActiveWallet, useWalletStore } from "./walletStore";

export function getPortfolioTotalValue(assets: PortfolioItem[]): number {
  return assets.reduce((total, asset) => total + asset.price * asset.amount, 0);
}

// Stale time in milliseconds (data considered fresh for 30 seconds)
const STALE_TIME = 30 * 1000;

interface PortfolioState {
  // Separate maps for different wallet types
  listenSolanaAssetsMap: Map<string, PortfolioItem>;
  listenEvmAssetsMap: Map<string, PortfolioItem>;
  eoaSolanaAssetsMap: Map<string, PortfolioItem>;
  eoaEvmAssetsMap: Map<string, PortfolioItem>;

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
  fetchSolanaPortfolio: (
    address: string,
    walletType: ActiveWallet
  ) => Promise<void>;
  fetchEvmPortfolio: (
    address: string,
    walletType: ActiveWallet
  ) => Promise<void>;
  fetchAllPortfolios: (fetchAll?: boolean) => Promise<void>;
  refreshPortfolio: (fetchAll?: boolean) => Promise<void>;
  isFresh: () => boolean;
  initializePortfolioManager: () => void;

  clearPortfolio: () => void;

  // Add new action
  updateTokenBalance: (mint: string, amount: number) => void;

  // Add these to match the persisted state
  listenSolanaAssets?: PortfolioItem[];
  listenEvmAssets?: PortfolioItem[];
  eoaSolanaAssets?: PortfolioItem[];
  eoaEvmAssets?: PortfolioItem[];
}

// Define the persisted state type
interface PersistedPortfolioState {
  listenSolanaAssets: PortfolioItem[];
  listenEvmAssets: PortfolioItem[];
  eoaSolanaAssets: PortfolioItem[];
  eoaEvmAssets: PortfolioItem[];
  lastUpdated: number | null;
}

export const usePortfolioStore = create<PortfolioState>()(
  persist<PortfolioState, [], [], PersistedPortfolioState>(
    (set, get) => ({
      // Initial state with separate maps
      listenSolanaAssetsMap: new Map<string, PortfolioItem>(),
      listenEvmAssetsMap: new Map<string, PortfolioItem>(),
      eoaSolanaAssetsMap: new Map<string, PortfolioItem>(),
      eoaEvmAssetsMap: new Map<string, PortfolioItem>(),

      // Data
      solanaAssetsMap: new Map<string, PortfolioItem>(),
      evmAssetsMap: new Map<string, PortfolioItem>(),

      getSolanaAssets: () => Array.from(get().solanaAssetsMap.values()),
      getEvmAssets: () => Array.from(get().evmAssetsMap.values()),
      getCombinedPortfolio: () => {
        const { activeWallet } = useWalletStore.getState();

        switch (activeWallet) {
          case "listen":
            return [
              ...Array.from(get().listenSolanaAssetsMap.values()),
              ...Array.from(get().listenEvmAssetsMap.values()),
            ];
          case "eoaSolana":
            return Array.from(get().eoaSolanaAssetsMap.values());
          case "eoaEvm":
            return Array.from(get().eoaEvmAssetsMap.values());
          case "worldchain":
            return Array.from(get().evmAssetsMap.values());
          default:
            return [];
        }
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
      fetchSolanaPortfolio: async (
        address: string,
        walletType: ActiveWallet = "listen"
      ) => {
        if (!address) return;

        set((state) => ({
          isLoading: state.listenSolanaAssetsMap.size === 0,
          error: null,
        }));

        try {
          const solanaAssets = await fetchSolanaPortfolio(address);
          const solanaAssetsMap = new Map();
          solanaAssets.forEach((asset) => {
            solanaAssetsMap.set(asset.address, {
              ...asset,
              logoURI: asset.logoURI || "",
            });
          });

          // Store in appropriate map based on wallet type parameter
          set(() => ({
            [walletType === "listen"
              ? "listenSolanaAssetsMap"
              : "eoaSolanaAssetsMap"]: solanaAssetsMap,
            isLoading: false,
            lastUpdated: Date.now(),
          }));
        } catch (error) {
          set({
            error: error as Error,
            isLoading: false,
          });
          console.error("Error fetching Solana portfolio:", error);
        }
      },

      fetchEvmPortfolio: async (
        address: string,
        walletType: ActiveWallet = "listen"
      ) => {
        if (!address) return;

        set((state) => ({
          isLoading: state.listenEvmAssetsMap.size === 0,
          error: null,
        }));

        try {
          const evmAssets = await fetchEvmPortfolio(address);
          const evmAssetsMap = new Map();
          evmAssets.forEach((asset) => {
            evmAssetsMap.set(asset.address, asset);
          });

          // Store in appropriate map based on wallet type parameter
          set(() => ({
            [walletType === "listen"
              ? "listenEvmAssetsMap"
              : "eoaEvmAssetsMap"]: evmAssetsMap,
            isLoading: false,
            lastUpdated: Date.now(),
          }));
        } catch (error) {
          set({
            error: error as Error,
            isLoading: false,
          });
          console.error("Error fetching EVM portfolio:", error);
        }
      },

      // Update fetchAllPortfolios to accept optional parameter
      fetchAllPortfolios: async (fetchAll: boolean = false) => {
        const {
          solanaAddress,
          evmAddress,
          eoaSolanaAddress,
          eoaEvmAddress,
          activeWallet,
          worldchainAddress,
        } = useWalletStore.getState();

        console.log(
          "fetch all portfolios:",
          {
            solanaAddress,
            evmAddress,
            eoaSolanaAddress,
            eoaEvmAddress,
            activeWallet,
            worldchainAddress,
          },
          "all:",
          fetchAll
        );

        // If fetchAll is true, fetch everything regardless of active wallet
        if (fetchAll) {
          const fetchPromises: Promise<void>[] = [];

          // Listen wallet
          if (solanaAddress) {
            fetchPromises.push(
              get().fetchSolanaPortfolio(solanaAddress, "listen")
            );
          }
          if (evmAddress) {
            fetchPromises.push(get().fetchEvmPortfolio(evmAddress, "listen"));
          }

          // EOA Solana wallet
          if (eoaSolanaAddress) {
            fetchPromises.push(
              get().fetchSolanaPortfolio(eoaSolanaAddress, "eoaSolana")
            );
          }

          // EOA EVM wallet
          if (eoaEvmAddress) {
            fetchPromises.push(
              get().fetchEvmPortfolio(eoaEvmAddress, "eoaEvm")
            );
          }

          // Worldchain wallet
          if (worldchainAddress) {
            fetchPromises.push(
              get().fetchEvmPortfolio(worldchainAddress, "worldchain")
            );
          }

          if (fetchPromises.length === 0) {
            set({ isLoading: false, error: null });
            return;
          }

          set({ isLoading: true, error: null });

          try {
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
          return;
        }

        // Regular case - fetch only active wallet
        const addresses = {
          solana:
            activeWallet === "listen"
              ? solanaAddress
              : activeWallet === "eoaSolana"
                ? eoaSolanaAddress
                : null,
          evm:
            activeWallet === "listen"
              ? evmAddress
              : activeWallet === "eoaEvm"
                ? eoaEvmAddress
                : activeWallet === "worldchain"
                  ? worldchainAddress
                  : null,
        };

        if (!addresses.solana && !addresses.evm) {
          set({ isLoading: false, error: null });
          return;
        }

        set({ isLoading: true, error: null });

        try {
          const fetchPromises: Promise<void>[] = [];
          if (addresses.solana) {
            fetchPromises.push(
              get().fetchSolanaPortfolio(addresses.solana, activeWallet)
            );
          }
          if (addresses.evm) {
            fetchPromises.push(
              get().fetchEvmPortfolio(addresses.evm, activeWallet)
            );
          }

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
      refreshPortfolio: async (fetchAll: boolean = false) => {
        // Reset data first to ensure UI shows loading state and indicate force refresh
        set({ isLoading: true, error: null });

        // Fetch portfolios with current wallet addresses
        await get().fetchAllPortfolios(fetchAll);
      },

      // Initialize visibility listener and other portfolio management
      initializePortfolioManager: () => {
        // Don't add multiple event listeners
        const visibilityListenerAlreadyAdded = Boolean(
          (window as any).__portfolioVisibilityListenerAdded
        );

        if (!visibilityListenerAlreadyAdded) {
          // Function to handle visibility change
          const handleVisibilityChange = () => {
            if (document.visibilityState === "visible") {
              // On becoming visible, check if data is fresh
              if (!get().isFresh()) {
                console.debug("Data is stale, refreshing");
                get().refreshPortfolio();
              } else {
                console.debug("Data is fresh, no refresh needed");
              }
            }
          };

          // Add visibility listener
          document.addEventListener("visibilitychange", handleVisibilityChange);

          // Mark that we've added the listener
          (window as any).__portfolioVisibilityListenerAdded = true;
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
              // TODO possibly refetch for evm assets here too
              (window as any).__updatingSolanaTokenPrices = false;
            }
          }
        });

        // Store the unsubscribe function to prevent memory leaks
        (window as any).__portfolioTokenStoreUnsubscribe = unsubscribe;
      },

      clearPortfolio: () => {
        set({
          solanaAssetsMap: new Map(),
          evmAssetsMap: new Map(),
          isLoading: false, // Also reset loading state
          error: null,
          lastUpdated: null,
        });
      },

      updateTokenBalance: (mint: string, amount: number) => {
        set((state) => {
          const existingAsset = state.solanaAssetsMap.get(mint);
          if (!existingAsset) return state; // No changes if token not in portfolio

          // Create new map for immutable update
          const updatedMap = new Map(state.solanaAssetsMap);
          updatedMap.set(mint, {
            ...existingAsset,
            amount: amount / Math.pow(10, existingAsset.decimals),
          });

          return {
            solanaAssetsMap: updatedMap,
            lastUpdated: Date.now(),
          };
        });
      },
    }),
    {
      name: "portfolio-storage",
      partialize: (state): PersistedPortfolioState => ({
        listenSolanaAssets: Array.from(state.listenSolanaAssetsMap.values()),
        listenEvmAssets: Array.from(state.listenEvmAssetsMap.values()),
        eoaSolanaAssets: Array.from(state.eoaSolanaAssetsMap.values()),
        eoaEvmAssets: Array.from(state.eoaEvmAssetsMap.values()),
        lastUpdated: state.lastUpdated,
      }),
    }
  )
);
