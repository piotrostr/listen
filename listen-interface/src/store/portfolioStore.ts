import { create } from "zustand";
import { persist } from "zustand/middleware";
import { ChatType } from "../contexts/SettingsContext";
import { PortfolioItem } from "../hooks/types";
import { getTokenHoldings as fetchEvmPortfolio } from "../hooks/useEvmPortfolioAlchemy";
import { fetchPortfolio as fetchSolanaPortfolio } from "../hooks/useSolanaPortfolio";

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
  chatType: ChatType;

  // Status
  isLoading: boolean;
  error: Error | null;
  lastUpdated: number | null; // timestamp in ms

  // Actions
  fetchSolanaPortfolio: (address: string) => Promise<void>;
  fetchEvmPortfolio: (address: string) => Promise<void>;
  fetchAllPortfolios: (
    solanaAddress: string,
    evmAddress: string
  ) => Promise<void>;
  setChatType: (type: ChatType) => void;
  refreshPortfolio: (
    solanaAddress: string,
    evmAddress: string,
    force?: boolean
  ) => Promise<void>;
  isFresh: () => boolean;
  updateCombinedPortfolio: () => void;
}

export const usePortfolioStore = create<PortfolioState>()(
  persist(
    (set, get) => ({
      // Initial data state
      solanaAssets: [],
      evmAssets: [],
      combinedPortfolio: [],
      portfolioValue: 0,
      chatType: "solana", // Default to solana-only mode

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

      // Helper to update combined portfolio based on current chatType
      updateCombinedPortfolio: () => {
        set((state) => {
          // Only include Solana assets if chatType is "solana"
          const combinedPortfolio =
            state.chatType === "solana"
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

        set({ isLoading: true, error: null });

        try {
          const solanaAssets = await fetchSolanaPortfolio(address);

          // Normalize assets to ensure logoURI is always a string
          const normalizedAssets = solanaAssets.map((asset) => ({
            ...asset,
            logoURI: asset.logoURI || "",
          }));

          set((_state) => {
            // Store the Solana assets
            const newState = {
              solanaAssets: normalizedAssets,
              isLoading: false,
              lastUpdated: Date.now(),
            };

            return newState;
          });

          // Update combined portfolio separately to ensure correct filtering
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

        // Skip fetching EVM assets if we're in Solana-only mode
        if (get().chatType === "solana") {
          console.log("Skipping EVM fetch in Solana-only mode");
          return;
        }

        set((state) => ({
          isLoading: !state.solanaAssets.length, // Only show loading if we have no data
          error: null,
        }));

        try {
          const evmAssets = await fetchEvmPortfolio(address);

          set({
            evmAssets,
            isLoading: false,
            lastUpdated: Date.now(),
          });

          // Update combined portfolio separately
          get().updateCombinedPortfolio();
        } catch (error) {
          set({
            error: error as Error,
            isLoading: false,
          });
          console.error("Error fetching EVM portfolio:", error);
        }
      },

      fetchAllPortfolios: async (solanaAddress: string, evmAddress: string) => {
        if (!solanaAddress && !evmAddress) return;

        set({ isLoading: true, error: null });

        try {
          // Always fetch Solana portfolio if solanaAddress is provided
          if (solanaAddress) {
            await get().fetchSolanaPortfolio(solanaAddress);
          }

          // Only fetch EVM portfolio if chatType is "omni" and evmAddress is provided
          if (evmAddress && get().chatType === "omni") {
            await get().fetchEvmPortfolio(evmAddress);
          }

          // Update combined portfolio after both fetches
          get().updateCombinedPortfolio();
        } catch (error) {
          set({
            error: error as Error,
            isLoading: false,
          });
          console.error("Error fetching portfolios:", error);
        }
      },

      setChatType: (type: ChatType) => {
        set({ chatType: type });

        // Update the combined portfolio whenever the chatType changes
        get().updateCombinedPortfolio();
      },

      refreshPortfolio: async (
        solanaAddress: string,
        evmAddress: string,
        force = false
      ) => {
        // Skip refresh if data is fresh and force is false
        if (!force && get().isFresh()) {
          console.log("Portfolio data is fresh, skipping refresh");
          return;
        }

        console.log("Refreshing portfolio, chatType:", get().chatType);

        // Reset data first to ensure UI shows loading state
        set((_state) => ({
          isLoading: true,
          error: null,
        }));

        // Reuse the fetchAllPortfolios to refresh
        await get().fetchAllPortfolios(solanaAddress, evmAddress);
      },
    }),
    {
      name: "portfolio-storage",
      // Only persist chat type preference
      partialize: (state) => ({
        chatType: state.chatType,
      }),
    }
  )
);
