import { create } from "zustand";
import { persist } from "zustand/middleware";
import { PortfolioItem } from "../hooks/types";
import { getTokenHoldings as fetchEvmPortfolio } from "../hooks/useEvmPortfolioAlchemy";
import { fetchPortfolio as fetchSolanaPortfolio } from "../hooks/useSolanaPortfolio";

export function getPortfolioTotalValue(assets: PortfolioItem[]): number {
  return assets.reduce((total, asset) => total + asset.price * asset.amount, 0);
}

interface PortfolioState {
  // Data
  solanaAssets: PortfolioItem[];
  evmAssets: PortfolioItem[];
  combinedPortfolio: PortfolioItem[];
  portfolioValue: number;
  chatType: "solana" | "all";

  // Status
  isLoading: boolean;
  error: Error | null;
  lastUpdated: Date | null;

  // Actions
  fetchSolanaPortfolio: (address: string) => Promise<void>;
  fetchEvmPortfolio: (address: string) => Promise<void>;
  fetchAllPortfolios: (
    solanaAddress: string,
    evmAddress: string
  ) => Promise<void>;
  setChatType: (type: "solana" | "all") => void;
  refreshPortfolio: (
    solanaAddress: string,
    evmAddress: string
  ) => Promise<void>;
}

export const usePortfolioStore = create<PortfolioState>()(
  persist(
    (set, get) => ({
      // Initial data state
      solanaAssets: [],
      evmAssets: [],
      combinedPortfolio: [],
      portfolioValue: 0,
      chatType: "all",

      // Initial status
      isLoading: false,
      error: null,
      lastUpdated: null,

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

          set((state) => {
            // Recalculate the combined portfolio
            const combinedPortfolio =
              state.chatType === "solana"
                ? normalizedAssets
                : [...normalizedAssets, ...state.evmAssets];

            // Calculate portfolio value
            const portfolioValue = getPortfolioTotalValue(combinedPortfolio);

            return {
              solanaAssets: normalizedAssets,
              combinedPortfolio,
              portfolioValue,
              isLoading: false,
              lastUpdated: new Date(),
            };
          });
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

        set((state) => ({
          isLoading: !state.solanaAssets.length, // Only show loading if we have no data
          error: null,
        }));

        try {
          const evmAssets = await fetchEvmPortfolio(address);

          set((state) => {
            // Recalculate the combined portfolio if not in Solana-only mode
            const combinedPortfolio =
              state.chatType === "solana"
                ? state.solanaAssets
                : [...state.solanaAssets, ...evmAssets];

            // Calculate portfolio value
            const portfolioValue = getPortfolioTotalValue(combinedPortfolio);

            return {
              evmAssets,
              combinedPortfolio,
              portfolioValue,
              isLoading: false,
              lastUpdated: new Date(),
            };
          });
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
          // Start both fetches in parallel
          const fetchPromises: Promise<any>[] = [];

          if (solanaAddress) {
            fetchPromises.push(fetchSolanaPortfolio(solanaAddress));
          }

          if (evmAddress && get().chatType !== "solana") {
            fetchPromises.push(fetchEvmPortfolio(evmAddress));
          }

          const [solanaAssetsResult, evmAssetsResult] =
            await Promise.allSettled(fetchPromises);

          // Handle the results
          let solanaAssets: PortfolioItem[] = get().solanaAssets;
          let evmAssets: PortfolioItem[] = get().evmAssets;

          if (solanaAssetsResult.status === "fulfilled" && solanaAddress) {
            // Normalize assets
            solanaAssets = solanaAssetsResult.value.map(
              (asset: PortfolioItem) => ({
                ...asset,
                logoURI: asset.logoURI || "",
              })
            );
          }

          if (
            evmAssetsResult?.status === "fulfilled" &&
            evmAddress &&
            get().chatType !== "solana"
          ) {
            evmAssets = evmAssetsResult.value;
          }

          // Determine combined portfolio based on chat type
          const combinedPortfolio =
            get().chatType === "solana"
              ? solanaAssets
              : [...solanaAssets, ...evmAssets];

          // Calculate portfolio value
          const portfolioValue = getPortfolioTotalValue(combinedPortfolio);

          set({
            solanaAssets,
            evmAssets,
            combinedPortfolio,
            portfolioValue,
            isLoading: false,
            lastUpdated: new Date(),
          });
        } catch (error) {
          set({
            error: error as Error,
            isLoading: false,
          });
          console.error("Error fetching portfolios:", error);
        }
      },

      setChatType: (type: "solana" | "all") => {
        set((state) => {
          // Recalculate combined portfolio based on new chat type
          const combinedPortfolio =
            type === "solana"
              ? state.solanaAssets
              : [...state.solanaAssets, ...state.evmAssets];

          // Recalculate portfolio value
          const portfolioValue = getPortfolioTotalValue(combinedPortfolio);

          return {
            chatType: type,
            combinedPortfolio,
            portfolioValue,
          };
        });
      },

      refreshPortfolio: async (solanaAddress: string, evmAddress: string) => {
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
