import { MiniKit } from "@worldcoin/minikit-js";
import { MiniKitProvider } from "@worldcoin/minikit-js/minikit-provider";
import {
  createContext,
  ReactNode,
  useContext,
  useEffect,
  useState,
} from "react";

interface WorldContextType {
  isWorldApp: boolean;
  isLoading: boolean;
}

const WorldContext = createContext<WorldContextType | undefined>(undefined);

export const WorldProvider = ({ children }: { children: ReactNode }) => {
  const [isWorldApp, setIsWorldApp] = useState<boolean>(false);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  useEffect(() => {
    const checkContext = () => {
      try {
        const isInstalled = MiniKit.isInstalled();
        setIsWorldApp(isInstalled);
      } catch (error) {
        // Silently handle the error - we're not in World App
        setIsWorldApp(false);
      } finally {
        setIsLoading(false);
      }
    };

    checkContext();
  }, []);

  return (
    <MiniKitProvider>
      <WorldContext.Provider value={{ isWorldApp, isLoading }}>
        {children}
      </WorldContext.Provider>
    </MiniKitProvider>
  );
};

export const useWorld = () => {
  const context = useContext(WorldContext);
  if (context === undefined) {
    throw new Error("useWorld must be used within a WorldProvider");
  }
  return context;
};
