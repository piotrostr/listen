import {
  createContext,
  ReactNode,
  useContext,
  useEffect,
  useState,
} from "react";

interface SettingsContextType {
  quickBuyAmount: number;
  setQuickBuyAmount: (amount: number) => void;
  agentMode: boolean;
  setAgentMode: (enabled: boolean) => void;
}

const SettingsContext = createContext<SettingsContextType | null>(null);

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [quickBuyAmount, setQuickBuyAmount] = useState<number>(0.1);
  const [agentMode, setAgentMode] = useState<boolean>(false);

  // Load settings from localStorage on initial render
  useEffect(() => {
    // Load quick buy amount
    const savedAmount = localStorage.getItem("quickBuyAmount");
    if (savedAmount) {
      setQuickBuyAmount(parseFloat(savedAmount));
    }

    // Load agent mode setting
    const savedAgentMode = localStorage.getItem("agentMode");
    if (savedAgentMode) {
      setAgentMode(savedAgentMode === "true");
    }
  }, []);

  // Save quick buy amount to localStorage when it changes
  const updateQuickBuyAmount = (amount: number) => {
    if (!isNaN(amount) && amount > 0) {
      setQuickBuyAmount(amount);
      localStorage.setItem("quickBuyAmount", amount.toString());
    }
  };

  // Save agent mode to localStorage when it changes
  const updateAgentMode = (enabled: boolean) => {
    setAgentMode(enabled);
    localStorage.setItem("agentMode", enabled.toString());
  };

  return (
    <SettingsContext.Provider
      value={{
        quickBuyAmount,
        setQuickBuyAmount: updateQuickBuyAmount,
        agentMode,
        setAgentMode: updateAgentMode,
      }}
    >
      {children}
    </SettingsContext.Provider>
  );
}

export function useSettings() {
  const context = useContext(SettingsContext);
  if (!context) {
    throw new Error("useSettings must be used within a SettingsProvider");
  }
  return context;
}
