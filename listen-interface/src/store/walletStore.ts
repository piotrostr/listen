import { z } from "zod";
import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

const ActiveWalletSchema = z.enum(["listen", "eoaSolana", "eoaEvm"]);

type ActiveWallet = z.infer<typeof ActiveWalletSchema>;

interface WalletState {
  // Wallet addresses
  solanaAddress: string | null;
  evmAddress: string | null;

  // EOA - Externally Owned Account
  eoaSolanaAddress: string | null;
  eoaEvmAddress: string | null;

  activeWallet: ActiveWallet;

  // icon URLs from privy
  eoaEvmIcon: string | null;
  eoaSolanaIcon: string | null;

  // Actions
  setWalletAddresses: (
    solanaAddress: string | null,
    evmAddress: string | null
  ) => void;

  setEoaSolanaAddress: (solanaAddress: string | null) => void;
  setEoaEvmAddress: (evmAddress: string | null) => void;

  setEoaEvmIcon: (icon: string | null) => void;
  setEoaSolanaIcon: (icon: string | null) => void;

  setActiveWallet: (active: ActiveWallet) => void;

  clearWalletAddresses: () => void;
}

export const useWalletStore = create<WalletState>()(
  persist(
    (set) => ({
      // Initial state
      solanaAddress: null,
      evmAddress: null,

      eoaSolanaAddress: null,
      eoaEvmAddress: null,

      eoaEvmIcon: null,
      eoaSolanaIcon: null,

      activeWallet: "listen",

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

      clearWalletAddresses: () => {
        set({
          solanaAddress: null,
          evmAddress: null,
        });
      },

      setEoaSolanaAddress: (solanaAddress: string | null) => {
        set({ eoaSolanaAddress: solanaAddress });
      },

      setEoaEvmAddress: (evmAddress: string | null) => {
        set({ eoaEvmAddress: evmAddress });
      },

      setEoaEvmIcon: (icon: string | null) => {
        set({ eoaEvmIcon: icon });
      },

      setEoaSolanaIcon: (icon: string | null) => {
        set({ eoaSolanaIcon: icon });
      },

      setActiveWallet: (active: ActiveWallet) => {
        set({ activeWallet: active });
      },
    }),
    {
      name: "wallet-storage",
      // Only persist the activeWallet state
      partialize: (state) => ({ activeWallet: state.activeWallet }),
      storage: createJSONStorage(() => localStorage),
    }
  )
);
