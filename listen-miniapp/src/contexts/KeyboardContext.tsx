import { createContext, useContext, useState } from "react";

interface KeyboardContextType {
  isKeyboardOpen: boolean;
  setIsKeyboardOpen: (isOpen: boolean) => void;
}

const KeyboardContext = createContext<KeyboardContextType | undefined>(
  undefined
);

export function KeyboardProvider({ children }: { children: React.ReactNode }) {
  const [isKeyboardOpen, setIsKeyboardOpen] = useState(false);

  return (
    <KeyboardContext.Provider value={{ isKeyboardOpen, setIsKeyboardOpen }}>
      {children}
    </KeyboardContext.Provider>
  );
}

export function useKeyboard() {
  const context = useContext(KeyboardContext);
  if (context === undefined) {
    throw new Error("useKeyboard must be used within a KeyboardProvider");
  }
  return context;
}
