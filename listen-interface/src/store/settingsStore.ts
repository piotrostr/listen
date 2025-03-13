import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ChatType = "solana" | "omni";

interface SettingsState {
  quickBuyAmount: number;
  agentMode: boolean;
  chatType: ChatType;
  debugMode: boolean;
  setQuickBuyAmount: (amount: number) => void;
  setAgentMode: (enabled: boolean) => void;
  setChatType: (type: ChatType) => void;
  setDebugMode: (enabled: boolean) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      quickBuyAmount: 0.1,
      agentMode: false,
      chatType: "solana" as ChatType,
      debugMode: false,
      setQuickBuyAmount: (amount: number) => {
        if (!isNaN(amount) && amount > 0) {
          set({ quickBuyAmount: amount });
        }
      },

      setAgentMode: (enabled: boolean) => {
        set({ agentMode: enabled });
      },

      setChatType: (type: ChatType) => {
        set({ chatType: type });
      },

      setDebugMode: (enabled: boolean) => {
        set({ debugMode: enabled });
      },
    }),
    {
      name: "settings-storage",
    }
  )
);
