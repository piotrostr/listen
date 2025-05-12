import { useEffect, useState } from "react";

export function usePWAStatus() {
  const [isPWA, setIsPWA] = useState(
    () =>
      window.matchMedia("(display-mode: standalone)").matches ||
      (window.navigator as any).standalone
  );

  useEffect(() => {
    const mediaQuery = window.matchMedia("(display-mode: standalone)");
    const handler = (e: MediaQueryListEvent) => setIsPWA(e.matches);

    mediaQuery.addEventListener("change", handler);
    return () => mediaQuery.removeEventListener("change", handler);
  }, []);

  return isPWA;
}
