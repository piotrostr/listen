import { useQuery } from "@tanstack/react-query";
import { usePortfolioStore } from "../store/portfolioStore";

interface OpenPriceResponse {
  price: number;
  timestamp: number;
}

export function use24hOpenPrice(mint: string) {
  const updateOpenPrice = usePortfolioStore((state) => state.updateOpenPrice);

  return useQuery<OpenPriceResponse>({
    queryKey: ["24h-open-price", mint],
    queryFn: async () => {
      const response = await fetch(
        `https://api.listen-rs.com/v1/adapter/24h-open?mint=${mint}`
      );
      if (!response.ok) {
        throw new Error("Failed to fetch 24h open price");
      }
      const data = await response.json();
      updateOpenPrice(mint, data.price);
      return data;
    },
    staleTime: 5 * 60 * 1000, // Consider data fresh for 5 minutes
    gcTime: 24 * 60 * 60 * 1000, // Cache for 24 hours
  });
}
