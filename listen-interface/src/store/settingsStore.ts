import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ChatType = "solana" | "omni";

interface SettingsState {
  quickBuyAmount: number;
  agentMode: boolean;
  chatType: ChatType;
  setQuickBuyAmount: (amount: number) => void;
  setAgentMode: (enabled: boolean) => void;
  setChatType: (type: ChatType) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      quickBuyAmount: 0.1,
      agentMode: false,
      chatType: "solana" as ChatType,

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
    }),
    {
      name: "settings-storage",
    }
  )
);
