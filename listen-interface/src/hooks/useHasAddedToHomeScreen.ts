import { useState } from "react";

const STORAGE_KEY = "hasAddedToHomeScreen";

enum HomeScreenStatus {
  Added = "added",
  Later = "later",
  NotAdded = "not_added",
}

export const useHasAddedToHomeScreen = () => {
  const [hasAddedToHomeScreen, setHasAddedToHomeScreen] = useState(() => {
    return (
      localStorage.getItem(STORAGE_KEY) === HomeScreenStatus.Added ||
      localStorage.getItem(STORAGE_KEY) === HomeScreenStatus.Later
    );
  });

  const updateHomeScreenStatus = (value: boolean) => {
    setHasAddedToHomeScreen(value);
    localStorage.setItem(STORAGE_KEY, String(value));
  };

  return [hasAddedToHomeScreen, updateHomeScreenStatus] as const;
};
