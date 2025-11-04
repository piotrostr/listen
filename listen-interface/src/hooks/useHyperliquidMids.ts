import { useQuery } from "@tanstack/react-query";

interface HyperliquidMidsResponse {
  [symbol: string]: string; // Symbol to price mapping
}

async function fetchHyperliquidMids(): Promise<Map<string, number>> {
  try {
    const response = await fetch("https://api.hyperliquid.xyz/info", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        type: "allMids",
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch Hyperliquid mids: ${response.statusText}`);
    }

    const data: HyperliquidMidsResponse = await response.json();
    
    // Convert to Map with symbol as key and price as number
    const midsMap = new Map<string, number>();
    Object.entries(data).forEach(([symbol, price]) => {
      midsMap.set(symbol, parseFloat(price));
    });
    
    return midsMap;
  } catch (error) {
    console.error("Error fetching Hyperliquid mids:", error);
    throw error;
  }
}

export function useHyperliquidMids() {
  return useQuery({
    queryKey: ["hyperliquid-mids"],
    queryFn: fetchHyperliquidMids,
    staleTime: 30_000, // 30 seconds - prices change frequently
    gcTime: 5 * 60_000, // 5 minutes cache
    refetchInterval: 30_000, // Refetch every 30 seconds
    refetchOnWindowFocus: false,
    refetchOnMount: false,
    refetchOnReconnect: false,
  });
}