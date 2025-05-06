import { useState } from "react";

const STORAGE_KEY = "hasAddedToHomeScreen";

export const useHasAddedToHomeScreen = () => {
  const [hasAddedToHomeScreen, setHasAddedToHomeScreen] = useState(() => {
    return localStorage.getItem(STORAGE_KEY) === "true";
  });

  const updateHomeScreenStatus = (value: boolean) => {
    setHasAddedToHomeScreen(value);
    localStorage.setItem(STORAGE_KEY, String(value));
  };

  return [hasAddedToHomeScreen, updateHomeScreenStatus] as const;
};
