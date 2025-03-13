import { create } from "zustand";
import { persist } from "zustand/middleware";

interface WalletState {
  // Wallet addresses
  solanaAddress: string | null;
  evmAddress: string | null;

  // Actions
  setWalletAddresses: (
    solanaAddress: string | null,
    evmAddress: string | null
  ) => void;
}

export const useWalletStore = create<WalletState>()(
  persist(
    (set) => ({
      // Initial state
      solanaAddress: null,
      evmAddress: null,

      // Set wallet addresses
      setWalletAddresses: (
        solanaAddress: string | null,
        evmAddress: string | null
      ) => {
        set({
          solanaAddress,
          evmAddress,
        });
      },
    }),
    {
      name: "wallet-storage",
    }
  )
);
