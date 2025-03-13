import { create } from "zustand";
import { persist } from "zustand/middleware";
import { PortfolioItem } from "../hooks/types";
import { getTokenHoldings as fetchEvmPortfolio } from "../hooks/useEvmPortfolioAlchemy";
import { fetchPortfolio as fetchSolanaPortfolio } from "../hooks/useSolanaPortfolio";
import { useSettingsStore } from "./settingsStore";
import { useWalletStore } from "./walletStore";

export function getPortfolioTotalValue(assets: PortfolioItem[]): number {
  return assets.reduce((total, asset) => total + asset.price * asset.amount, 0);
}

// Stale time in milliseconds (data considered fresh for 30 seconds)
const STALE_TIME = 30 * 1000;

interface PortfolioState {
  // Data
  solanaAssets: PortfolioItem[];
  evmAssets: PortfolioItem[];
  combinedPortfolio: PortfolioItem[];
  portfolioValue: number;

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
  updateCombinedPortfolio: () => void;
  initializePortfolioManager: () => void;
}

export const usePortfolioStore = create<PortfolioState>()(
  persist(
    (set, get) => ({
      // Initial data state
      solanaAssets: [],
      evmAssets: [],
      combinedPortfolio: [],
      portfolioValue: 0,

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

      // Helper to update combined portfolio based on current settings
      updateCombinedPortfolio: () => {
        // Get chatType directly from settings store
        const chatType = useSettingsStore.getState().chatType;

        set((state) => {
          // Only include Solana assets if chatType is "solana"
          const combinedPortfolio =
            chatType === "solana"
              ? [...state.solanaAssets]
              : [...state.solanaAssets, ...state.evmAssets];

          // Calculate portfolio value
          const portfolioValue = getPortfolioTotalValue(combinedPortfolio);

          return {
            combinedPortfolio,
            portfolioValue,
          };
        });
      },

      // Actions
      fetchSolanaPortfolio: async (address: string) => {
        if (!address) return;

        // Don't set isLoading if we already have data
        set((state) => ({
          isLoading: state.solanaAssets.length === 0,
          error: null,
        }));

        try {
          const solanaAssets = await fetchSolanaPortfolio(address);

          // Normalize assets to ensure logoURI is always a string
          const normalizedAssets = solanaAssets.map((asset) => ({
            ...asset,
            logoURI: asset.logoURI || "",
          }));

          set(() => ({
            solanaAssets: normalizedAssets,
            isLoading: false,
            lastUpdated: Date.now(),
          }));

          // Update combined portfolio
          get().updateCombinedPortfolio();
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
            state.evmAssets.length === 0 && state.solanaAssets.length === 0,
          error: null,
        }));

        try {
          const evmAssets = await fetchEvmPortfolio(address);

          set(() => ({
            evmAssets,
            isLoading: false,
            lastUpdated: Date.now(),
          }));

          // Update combined portfolio
          get().updateCombinedPortfolio();
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
        if (get().solanaAssets.length === 0 && !get().isLoading) {
          console.log("Initial portfolio load");
          get().refreshPortfolio();
        }
      },
    }),
    {
      name: "portfolio-storage",
      partialize: (state) => ({
        lastUpdated: state.lastUpdated,
      }),
    }
  )
);
