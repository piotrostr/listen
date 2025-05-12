import { useCallback, useEffect, useState } from "react";

const STORAGE_KEY = "hasAddedToHomeScreen";

export const useHasAddedToHomeScreen = () => {
  const [hasAddedToHomeScreen, setHasAddedToHomeScreen] = useState(() => {
    return localStorage.getItem(STORAGE_KEY) === "true";
  });
  const [isVisible, setIsVisible] = useState(false);

  // Delay the initial appearance
  useEffect(() => {
    if (!hasAddedToHomeScreen) {
      const timer = setTimeout(() => setIsVisible(true), 1200);
      return () => clearTimeout(timer);
    }
  }, [hasAddedToHomeScreen]);

  const hide = useCallback(() => {
    setIsVisible(false);
    // After animation completes, update the storage
    setTimeout(() => {
      setHasAddedToHomeScreen(true);
      localStorage.setItem(STORAGE_KEY, "true");
    }, 360);
  }, []);

  return {
    hasAddedToHomeScreen,
    isVisible,
    hide,
  } as const;
};
