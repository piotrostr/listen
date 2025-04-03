import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ChatType = "solana" | "omni";
export type ModelType = "gemini" | "claude" | "openai" | "deepseek";

interface SettingsState {
  quickBuyAmount: number;
  agentMode: boolean;
  chatType: ChatType;
  debugMode: boolean;
  modelType: ModelType;
  displaySuggestions: boolean;
  researchEnabled: boolean;
  tradingEnabled: boolean;
  setQuickBuyAmount: (amount: number) => void;
  setAgentMode: (enabled: boolean) => void;
  setChatType: (type: ChatType) => void;
  setDebugMode: (enabled: boolean) => void;
  setModelType: (type: ModelType) => void;
  setDisplaySuggestions: (enabled: boolean) => void;
  setResearchEnabled: (enabled: boolean) => void;
  setTradingEnabled: (enabled: boolean) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      quickBuyAmount: 0.1,
      agentMode: false,
      chatType: "solana" as ChatType,
      debugMode: false,
      modelType: "gemini" as ModelType,
      displaySuggestions: true,
      researchEnabled: false,
      tradingEnabled: false,

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

      setDisplaySuggestions: (enabled: boolean) => {
        set({ displaySuggestions: enabled });
      },

      setResearchEnabled: (enabled: boolean) => {
        set({ researchEnabled: enabled });
      },

      setTradingEnabled: (enabled: boolean) => {
        set({ tradingEnabled: enabled });
      },
    }),
    {
      name: "settings-storage",
    }
  )
);
