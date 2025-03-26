import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ChatType = "solana" | "omni";
export type ModelType = "gemini" | "claude";

interface SettingsState {
  quickBuyAmount: number;
  agentMode: boolean;
  chatType: ChatType;
  debugMode: boolean;
  modelType: ModelType;
  setQuickBuyAmount: (amount: number) => void;
  setAgentMode: (enabled: boolean) => void;
  setChatType: (type: ChatType) => void;
  setDebugMode: (enabled: boolean) => void;
  setModelType: (type: ModelType) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      quickBuyAmount: 0.1,
      agentMode: false,
      chatType: "solana" as ChatType,
      debugMode: false,
      modelType: "claude" as ModelType,

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

      setModelType: (type: ModelType) => {
        set({ modelType: type });
      },
    }),
    {
      name: "settings-storage",
    }
  )
);
