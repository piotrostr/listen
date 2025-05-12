import { createContext, useContext, useState } from "react";

interface PanelContextType {
  activePanel: string | null;
  setActivePanel: (panel: string | null) => void;
}

const PanelContext = createContext<PanelContextType | undefined>(undefined);

export function PanelProvider({ children }: { children: React.ReactNode }) {
  const [activePanel, setActivePanel] = useState<string | null>(() => {
    // Initialize from localStorage if available
    const stored = localStorage.getItem("activePanel");
    return stored || null;
  });

  const handleSetActivePanel = (panel: string | null) => {
    setActivePanel(panel);
    if (panel) {
      localStorage.setItem("activePanel", panel);
    } else {
      localStorage.removeItem("activePanel");
    }
  };

  return (
    <PanelContext.Provider
      value={{ activePanel, setActivePanel: handleSetActivePanel }}
    >
      {children}
    </PanelContext.Provider>
  );
}

export function usePanel() {
  const context = useContext(PanelContext);
  if (context === undefined) {
    throw new Error("usePanel must be used within a PanelProvider");
  }
  return context;
}
